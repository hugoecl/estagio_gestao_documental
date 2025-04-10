use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use log::error;
use serde::Deserialize;

use crate::{
    State,
    auth::validate_session,
    models::role::UserRoleAssignment,
    utils::{
        hashing_utils::{hash, verify},
        json_utils::{Json, json_response},
    },
};

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

pub async fn register(state: web::Data<State>, request_data: web::Bytes) -> impl Responder {
    let Json(user): Json<RegisterRequest> = match Json::from_bytes(&request_data) {
        Ok(data) => data,
        Err(e) => {
            error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().body("Invalid JSON");
        }
    };

    // Check if user already exists
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE username = ? OR email = ?",
        user.username,
        user.email
    )
    .fetch_optional(&state.db.pool)
    .await;

    match existing_user {
        Ok(Some(_)) => return HttpResponse::Conflict().body("Username or email already exists"),
        Ok(None) => {} // Continue with registration
        Err(e) => {
            error!("Database error checking existing user: {}", e);
            return HttpResponse::InternalServerError().body("Error checking existing user");
        }
    }

    let password_bytes = hash(&user.password);

    let result = sqlx::query!(
        "INSERT INTO users (username, email, password) VALUES (?, ?, ?)",
        user.username,
        user.email,
        &password_bytes[..]
    )
    .execute(&state.db.pool)
    .await;

    match result {
        Ok(r) => {
            let user_id = r.last_insert_id() as u32;
            // Assign default role if needed
            HttpResponse::Ok().body(format!("User registered successfully with ID {}", user_id))
        }
        Err(e) => {
            error!("Database error during user registration: {}", e);
            HttpResponse::InternalServerError().body("Error registering user")
        }
    }
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

pub async fn login(
    state: web::Data<State>,
    request_data: web::Bytes,
    session: Session,
) -> impl Responder {
    let Json(req): Json<LoginRequest> = match Json::from_bytes(&request_data) {
        Ok(data) => data,
        Err(e) => {
            error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().body("Invalid JSON");
        }
    };

    // Find user by email
    let user = sqlx::query!("SELECT id, password FROM users WHERE email = ?", req.email)
        .fetch_optional(&state.db.pool)
        .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => return HttpResponse::Unauthorized().finish(),
        Err(e) => {
            error!("Database error during login: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    // Verify password
    if !verify(&req.password, &&user.password[..]) {
        return HttpResponse::Unauthorized().body("Invalid credentials");
    }

    // Check if user is admin (has admin role)
    let is_admin = match sqlx::query!(
        r#"
        SELECT COUNT(*) as count
        FROM user_roles ur
        JOIN roles r ON ur.role_id = r.id
        WHERE ur.user_id = ? AND r.is_admin = 1
        "#,
        user.id
    )
    .fetch_one(&state.db.pool)
    .await
    {
        Ok(result) => result.count > 0,
        Err(e) => {
            error!("Database error checking admin status: {}", e);
            return HttpResponse::InternalServerError().body("Error checking permissions");
        }
    };

    // Set session data
    session.insert("user_id", user.id as i32).unwrap();
    session.insert("is_admin", is_admin).unwrap();
    session
        .insert("last_renewal", chrono::Utc::now().timestamp())
        .unwrap();

    HttpResponse::Ok().finish()
}

pub async fn check(session: Session, state: web::Data<State>, data: web::Bytes) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(response) => return response,
    };

    let Ok(page_path) = String::from_utf8(data.to_vec()) else {
        return HttpResponse::BadRequest().finish();
    };

    let now = chrono::Utc::now();

    // Query current visit count
    let analytics = sqlx::query!(
        r#"
        SELECT visit_count FROM user_page_analytics
        WHERE user_id = ? AND page_path = ?
        "#,
        user_id,
        &page_path
    )
    .fetch_optional(&state.db.pool)
    .await;

    match analytics {
        Ok(Some(record)) => {
            // Record exists, update with increment
            let new_count = record.visit_count + 1;

            // Run update in background task
            actix_web::rt::spawn(async move {
                sqlx::query!(
                    r#"
                    UPDATE user_page_analytics
                    SET visit_count = ?, last_visited_at = ?
                    WHERE user_id = ? AND page_path = ?
                    "#,
                    new_count,
                    now,
                    user_id,
                    page_path
                )
                .execute(&state.db.pool)
                .await
                .ok();
            });
        }
        Ok(None) => {
            // No record exists, create new one
            actix_web::rt::spawn(async move {
                sqlx::query!(
                    r#"
                    INSERT INTO user_page_analytics (user_id, page_path, visit_count, last_visited_at)
                    VALUES (?, ?, ?, ?)
                    "#,
                    user_id,
                    page_path,
                    1,
                    now
                )
                .execute(&state.db.pool)
                .await
                .ok();
            });
        }
        Err(e) => {
            error!("Database error checking analytics: {}", e);
        }
    }

    HttpResponse::Ok().finish()
}

pub async fn logout(session: Session) -> impl Responder {
    session.purge();
    HttpResponse::Ok().finish()
}

pub async fn protected(session: Session) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);

    if !is_admin {
        return HttpResponse::Forbidden().body("Admin privileges required");
    }

    HttpResponse::Ok().body("Protected Route")
}

pub async fn get_user_analytics(state: web::Data<State>, session: Session) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(response) => return response,
    };

    // Get analytics directly from database
    let analytics = sqlx::query!(
        r#"
        SELECT page_path, visit_count
        FROM user_page_analytics
        WHERE user_id = ?
        ORDER BY visit_count DESC
        "#,
        user_id
    )
    .fetch_all(&state.db.pool)
    .await;

    match analytics {
        Ok(records) => {
            let analytics_data: Vec<(String, i32)> = records
                .into_iter()
                .map(|r| (r.page_path, r.visit_count as i32))
                .collect();
            json_response(&analytics_data)
        }
        Err(e) => {
            error!("Database error fetching analytics: {}", e);
            HttpResponse::InternalServerError().body("Error fetching analytics")
        }
    }
}

// New endpoint for assigning roles to users
#[derive(Deserialize)]
struct AssignRoleRequest {
    user_id: u32,
    role_ids: Vec<u32>,
}

pub async fn assign_roles(
    state: web::Data<State>,
    request_data: web::Bytes,
    session: Session,
) -> impl Responder {
    // Only admins can assign roles
    let is_admin = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);

    if !is_admin {
        return HttpResponse::Forbidden().body("Admin privileges required");
    }

    let Json(req): Json<AssignRoleRequest> = match Json::from_bytes(&request_data) {
        Ok(data) => data,
        Err(e) => {
            error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().body("Invalid JSON");
        }
    };

    let assignment = UserRoleAssignment {
        user_id: req.user_id,
        role_ids: req.role_ids,
    };

    match crate::models::role::Role::assign_roles_to_user(&state.db.pool, &assignment).await {
        Ok(_) => HttpResponse::Ok().body("Roles assigned successfully"),
        Err(e) => {
            error!("Database error assigning roles: {}", e);
            HttpResponse::InternalServerError().body("Error assigning roles")
        }
    }
}
