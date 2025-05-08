use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use ahash::HashMap;
use log::error;
use serde::{Deserialize, Serialize};

use crate::{
    State,
    auth::{is_admin, user_can_manage_page, validate_session},
    models::{
        role::{Role, UserRoleAssignment},
        user::{UserRoleRow, UserWithRoles},
    },
    utils::{
        hashing_utils::{hash, verify},
        json_utils::{Json, json_response, json_response_with_etag},
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

    // --- Transaction Start ---
    let mut tx = match state.db.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Database error starting transaction: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    // Check if user already exists within the transaction
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE username = ? OR email = ?",
        user.username,
        user.email
    )
    .fetch_optional(&mut *tx) // Use transaction
    .await;

    match existing_user {
        Ok(Some(_)) => {
            // No need to rollback explicitly here, transaction will drop if we return
            return HttpResponse::Conflict().body("Username or email already exists");
        }
        Ok(None) => {} // Continue with registration
        Err(e) => {
            error!("Database error checking existing user: {}", e);
            // No need to rollback explicitly here
            return HttpResponse::InternalServerError().body("Error checking existing user");
        }
    }

    // Hash password
    let password_bytes = hash(&user.password);

    // Insert user within the transaction
    let insert_result = sqlx::query!(
        "INSERT INTO users (username, email, password) VALUES (?, ?, ?)",
        user.username,
        user.email,
        &password_bytes[..]
    )
    .execute(&mut *tx) // Use transaction
    .await;

    let user_id = match insert_result {
        Ok(r) => r.last_insert_id() as u32,
        Err(e) => {
            error!("Database error during user registration: {}", e);
            // No need to rollback explicitly here
            return HttpResponse::InternalServerError().body("Error registering user");
        }
    };

    // Fetch the default "Colaborador" role ID within the transaction
    let default_role = sqlx::query!("SELECT id FROM roles WHERE name = 'Colaborador' LIMIT 1")
        .fetch_one(&mut *tx) // Use transaction
        .await;

    let default_role_id = match default_role {
        Ok(role) => role.id,
        Err(e) => {
            error!("Database error fetching default role: {}", e);
            // No need to rollback explicitly here
            return HttpResponse::InternalServerError().body("Error assigning default role");
        }
    };

    // Assign the default role to the new user within the transaction
    let assign_result = sqlx::query!(
        "INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)",
        user_id,
        default_role_id
    )
    .execute(&mut *tx) // Use transaction
    .await;

    if let Err(e) = assign_result {
        error!("Database error assigning default role: {}", e);
        // No need to rollback explicitly here
        return HttpResponse::InternalServerError().body("Error assigning default role");
    }

    // --- Commit Transaction ---
    if let Err(e) = tx.commit().await {
        error!("Database error committing transaction: {}", e);
        return HttpResponse::InternalServerError().body("Error finalizing registration");
    }

    HttpResponse::Ok().body(format!("User registered successfully with ID {}", user_id))
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

#[derive(Serialize)]
pub struct CheckResponse {
    #[serde(rename = "isAdmin")]
    is_admin: bool,
    #[serde(rename = "canManageThisPage", skip_serializing_if = "Option::is_none")]
    // Only include if relevant
    can_manage_this_page: Option<bool>,
}

pub async fn check(state: web::Data<State>, session: Session, data: web::Bytes) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let Ok(page_path) = String::from_utf8(data.to_vec()) else {
        return HttpResponse::BadRequest().finish();
    };

    let is_admin_session = session
        .get::<bool>("is_admin")
        .unwrap_or(Some(false))
        .unwrap_or(false);

    let mut can_manage_this_page_result: Option<bool> = None;

    // Check if the path matches the admin edit page pattern
    const EDIT_PREFIX: &str = "/admin/pages/edit/";
    if page_path.starts_with(EDIT_PREFIX) {
        if let Some(id_str) = page_path.strip_prefix(EDIT_PREFIX) {
            // Remove trailing slash if present before parsing
            let id_str_cleaned = id_str.trim_end_matches('/');
            if let Ok(page_id) = id_str_cleaned.parse::<u32>() {
                // Check specific permission for this page
                match user_can_manage_page(&state.db.pool, user_id, page_id).await {
                    Ok(can_manage) => {
                        can_manage_this_page_result = Some(can_manage);
                    }
                    Err(e) => {
                        error!(
                            "Error checking page management permission for user {} on page {}: {}",
                            user_id, page_id, e
                        );
                        // Default to false on error, or return server error? Let's default to false for security.
                        can_manage_this_page_result = Some(false);
                    }
                }
            } else {
                log::warn!("Could not parse page ID from path: {}", page_path);
                can_manage_this_page_result = Some(false); // Invalid ID, cannot manage
            }
        }
    }

    // --- Analytics Update (remains the same) ---
    let now = chrono::Utc::now();
    let user_id_u32 = user_id as u32; // Use appropriate type for query
    let page_path_clone = page_path.clone(); // Clone for the async block
    let pool_clone = state.db.pool.clone(); // Clone pool for the async block

    actix_web::rt::spawn(async move {
        let analytics = sqlx::query!(
            r#"SELECT visit_count FROM user_page_analytics WHERE user_id = ? AND page_path = ?"#,
            user_id_u32,
            &page_path_clone // Use cloned path
        )
        .fetch_optional(&pool_clone) // Use cloned pool
        .await;

        match analytics {
            Ok(Some(record)) => {
                let new_count = record.visit_count + 1;
                sqlx::query!(
                    r#"UPDATE user_page_analytics SET visit_count = ?, last_visited_at = ? WHERE user_id = ? AND page_path = ?"#,
                    new_count, now, user_id_u32, page_path_clone
                )
                .execute(&pool_clone).await.ok(); // Use cloned pool
            }
            Ok(None) => {
                sqlx::query!(
                    r#"INSERT INTO user_page_analytics (user_id, page_path, visit_count, last_visited_at) VALUES (?, ?, ?, ?)"#,
                    user_id_u32, page_path_clone, 1, now
                )
                .execute(&pool_clone).await.ok(); // Use cloned pool
            }
            Err(e) => {
                error!("Database error checking analytics: {}", e);
            }
        }
    });
    // --- End Analytics Update ---

    // Return the potentially richer response
    json_response(&CheckResponse {
        is_admin: is_admin_session,
        can_manage_this_page: can_manage_this_page_result,
    })
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

pub async fn get_users_with_roles(
    state: web::Data<State>,
    session: Session,
    req: HttpRequest, // Add HttpRequest for ETag
) -> impl Responder {
    // Ensure only admins can access this
    if is_admin(&session).is_err() {
        return HttpResponse::Forbidden().finish();
    }

    // Fetch all users and their roles using a LEFT JOIN
    let rows = sqlx::query_as!(
        UserRoleRow,
        r#"
        SELECT
            u.id as user_id,
            u.username,
            u.email,
            r.id as role_id,
            r.name as role_name,
            r.description as role_description,
            r.is_admin as "role_is_admin: bool",
            r.created_at as "role_created_at?: chrono::DateTime<chrono::Utc>",
            r.updated_at as "role_updated_at?: chrono::DateTime<chrono::Utc>"
        FROM users u
        LEFT JOIN user_roles ur ON u.id = ur.user_id
        LEFT JOIN roles r ON ur.role_id = r.id
        ORDER BY u.username, r.name
        "#
    )
    .fetch_all(&state.db.pool)
    .await;

    match rows {
        Ok(rows) => {
            // Aggregate results: Group roles by user
            let mut users_map: HashMap<u32, UserWithRoles> = HashMap::default();

            for row in rows {
                let user = users_map
                    .entry(row.user_id)
                    .or_insert_with(|| UserWithRoles {
                        id: row.user_id,
                        username: row.username.clone(),
                        email: row.email.clone(),
                        roles: Vec::new(),
                    });

                // Add role if it exists (due to LEFT JOIN)
                if let (
                    Some(role_id),
                    Some(role_name),
                    Some(role_created_at),
                    Some(role_updated_at),
                    Some(role_is_admin),
                ) = (
                    row.role_id,
                    row.role_name,
                    row.role_created_at,
                    row.role_updated_at,
                    row.role_is_admin,
                ) {
                    user.roles.push(Role {
                        id: role_id,
                        name: role_name,
                        description: row.role_description,
                        is_admin: role_is_admin,
                        created_at: role_created_at,
                        updated_at: role_updated_at,
                    });
                }
            }

            // Convert map values to a Vec for the final response
            let users: Vec<UserWithRoles> = users_map.into_values().collect();
            json_response_with_etag(&users, &req) // Use ETag response
        }
        Err(e) => {
            error!("Database error fetching users with roles: {}", e);
            HttpResponse::InternalServerError().body("Error fetching users")
        }
    }
}
