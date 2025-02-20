use std::sync::atomic;

use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use crate::{
    db::UserCache,
    utils::{
        hashing_utils::{hash, verify},
        json_utils::Json,
        session_utils::admin_only,
    },
    State,
};

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

pub async fn register(state: web::Data<State>, request_data: web::Bytes) -> impl Responder {
    let Json(user): Json<RegisterRequest> = Json::from_bytes(request_data).unwrap();

    if state
        .cache
        .users
        .pin()
        .values()
        .any(|u| u.username == user.username)
    {
        return HttpResponse::Conflict().finish();
    }

    let password_bytes = hash(&user.password);

    let user_cache = UserCache {
        username: user.username.clone(),
        email: user.email.clone(),
        password: password_bytes,
        is_admin: false,
    };

    let new_user_id = state
        .cache
        .last_user_id
        .fetch_add(1, atomic::Ordering::SeqCst);

    state.cache.users.pin().insert(new_user_id, user_cache);

    actix_web::rt::spawn(async move {
        let _ = sqlx::query!(
            "INSERT INTO users (username, email, password, is_admin) VALUES (?, ?, ?, ?)",
            user.username,
            user.email,
            &password_bytes[..],
            false
        )
        .execute(&state.db.pool)
        .await;
    });

    HttpResponse::Ok().body("Registering user")
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

    for (i, u) in state.cache.users.pin().iter() {
        if u.email == req.email {
            if verify(&req.password, &u.password) {
                session.insert("user_id", i).unwrap();
                session.insert("is_admin", u.is_admin).unwrap();
                return HttpResponse::Ok().finish();
            }
        }
    }

    HttpResponse::Unauthorized().finish()
}

pub async fn protected(session: Session) -> impl Responder {
    if let Err(response) = admin_only(&session) {
        return response;
    }

    HttpResponse::Ok().body("Protected Route")
}
