use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use log::error;
use serde::Deserialize;

use crate::{
    State,
    cache::{AnalyticsKey, PageVisit, UserCache},
    utils::{
        hashing_utils::{hash, verify},
        json_utils::{Json, json_response},
        session_utils::{admin_only, validate_session},
    },
};

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

pub async fn register(state: web::Data<State>, request_data: web::Bytes) -> impl Responder {
    let Json(user): Json<RegisterRequest> = Json::from_bytes(request_data).unwrap();
    let pinned_users_cache = state.cache.users.pin();

    if pinned_users_cache
        .values()
        .any(|u| u.username == user.username || u.email == user.email)
    {
        return HttpResponse::Conflict().finish();
    }

    let password_bytes = hash(&user.password);

    let result = sqlx::query!(
        "INSERT INTO users (username, email, password, is_admin) VALUES (?, ?, ?, ?)",
        user.username,
        user.email,
        &password_bytes[..],
        false
    )
    .execute(&state.db.pool)
    .await;

    let result = match result {
        Ok(r) => r,
        Err(e) => {
            error!("Database error during user registration: {e}");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let new_user_id = result.last_insert_id() as u32;

    let user_cache = UserCache {
        username: user.username,
        email: user.email,
        password: password_bytes,
        is_admin: false,
    };

    pinned_users_cache.insert(new_user_id, user_cache);

    HttpResponse::Ok().body("User registered successfully")
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

pub async fn login(
    state: web::Data<State>,
    request_date: web::Bytes,
    session: Session,
) -> impl Responder {
    let Json(req): Json<LoginRequest> = Json::from_bytes(request_date).unwrap();
    let pinned_users_cache = state.cache.users.pin();

    for (i, u) in &pinned_users_cache {
        if u.email == req.email && verify(&req.password, &u.password) {
            session.insert("user_id", i).unwrap();
            session.insert("is_admin", u.is_admin).unwrap();
            return HttpResponse::Ok().finish();
        }
    }

    HttpResponse::Unauthorized().finish()
}

pub async fn check(session: Session, state: web::Data<State>, data: web::Bytes) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(response) => return response,
    };

    let page_path = match String::from_utf8(data.to_vec()) {
        Ok(p) => p,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let key = AnalyticsKey {
        user_id,
        page_path: page_path.clone(),
    };

    let now = chrono::Utc::now();
    let pinned_analytics_cache = state.cache.analytics.pin();

    let updated_value = pinned_analytics_cache.update_or_insert(
        key,
        |data| {
            const ONE_MINUTE: chrono::TimeDelta = chrono::Duration::minutes(1);
            if now.signed_duration_since(data.last_visited_at) > ONE_MINUTE {
                PageVisit {
                    visit_count: data.visit_count + 1,
                    last_visited_at: now,
                }
            } else {
                PageVisit {
                    visit_count: data.visit_count,
                    last_visited_at: now,
                }
            }
        },
        PageVisit {
            visit_count: 1,
            last_visited_at: now,
        },
    );

    let visit_count = updated_value.visit_count;
    let incremented = updated_value.last_visited_at == now; // Only true if this is a counted visit
    drop(pinned_analytics_cache);

    actix_web::rt::spawn(async move {
        if incremented {
            sqlx::query!(
                r#"
                INSERT INTO user_page_analytics (user_id, page_path, visit_count, last_visited_at) 
                VALUES (?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE 
                    visit_count = ?,
                    last_visited_at = ?
                "#,
                user_id,
                page_path,
                visit_count,
                now,
                visit_count,
                now
            )
            .execute(&state.db.pool)
            .await
            .ok();
        } else {
            sqlx::query!(
                r#"
                UPDATE user_page_analytics 
                SET last_visited_at = ? 
                WHERE user_id = ? AND page_path = ?
                "#,
                now,
                user_id,
                page_path
            )
            .execute(&state.db.pool)
            .await
            .ok();
        }
    });

    HttpResponse::Ok().finish()
}

pub async fn logout(session: Session) -> impl Responder {
    session.clear();
    HttpResponse::Ok().finish()
}

pub async fn protected(session: Session) -> impl Responder {
    if let Err(response) = admin_only(&session) {
        return response;
    }

    HttpResponse::Ok().body("Protected Route")
}

pub async fn get_user_analytics(state: web::Data<State>, session: Session) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(response) => return response,
    };

    let pinned_analytics_cache = state.cache.analytics.pin();
    let user_analytics = pinned_analytics_cache
        .iter()
        .filter(|(k, _)| k.user_id == user_id)
        .map(|(k, v)| (k.page_path.clone(), v.visit_count))
        .collect::<Vec<_>>();

    json_response(&user_analytics)
}
