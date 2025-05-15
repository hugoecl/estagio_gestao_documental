use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;
use std::collections::HashSet; // For unique user IDs

use crate::{
    State,
    auth::{is_admin, validate_session}, // Added is_admin
    models::{notification::Notification, role::Role}, // Added Role
    utils::json_utils::json_response,
};

const NOTIFICATION_TYPE_ADMIN_BROADCAST: &str = "ADMIN_BROADCAST";

// Handler to get the list of unread notifications for the current user
pub async fn get_unread_notifications(state: web::Data<State>, session: Session) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    match Notification::get_unread_by_user(&state.db.pool, user_id).await {
        Ok(notifications) => json_response(&notifications),
        Err(e) => {
            log::error!(
                "Error fetching unread notifications for user {}: {}",
                user_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

// New struct for the broadcast request
#[derive(Deserialize)]
pub struct BroadcastNotificationRequest {
    role_ids: Vec<u32>,
    message: String,
}

// New handler for broadcasting notifications
pub async fn broadcast_notification_to_roles(
    state: web::Data<State>,
    session: Session,
    body: web::Json<BroadcastNotificationRequest>,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        log::warn!("broadcast_notification_to_roles: Non-admin user attempted to broadcast.");
        return resp;
    }

    let req_data = body.into_inner();

    if req_data.role_ids.is_empty() {
        return HttpResponse::BadRequest().body("No role IDs provided for broadcast.");
    }
    if req_data.message.trim().is_empty() {
        return HttpResponse::BadRequest().body("Broadcast message cannot be empty.");
    }

    let mut notified_user_count = 0;
    let mut unique_user_ids_to_notify = HashSet::new();

    for role_id in req_data.role_ids {
        match Role::get_user_ids_by_role_id(&state.db.pool, role_id).await {
            Ok(user_ids) => {
                for user_id in user_ids {
                    unique_user_ids_to_notify.insert(user_id);
                }
            }
            Err(e) => {
                log::error!(
                    "Error fetching user IDs for role_id {}: {}. Skipping this role.",
                    role_id,
                    e
                );
                continue;
            }
        }
    }

    if unique_user_ids_to_notify.is_empty() {
        log::info!("No users found for the selected roles to send broadcast.");
        return HttpResponse::Ok().body("No users found for the selected roles to send broadcast.");
    }

    for user_id in unique_user_ids_to_notify {
        match Notification::create(
            &state.db.pool,
            user_id,
            None, // No specific record_id for a general broadcast
            None, // No specific page_id
            None, // No specific field_id
            NOTIFICATION_TYPE_ADMIN_BROADCAST,
            &req_data.message,
            None, // No due_date
        )
        .await
        {
            Ok(_) => {
                notified_user_count += 1;
                log::info!(
                    "Created ADMIN_BROADCAST notification for user_id {} with message: '{}'",
                    user_id,
                    req_data.message
                );
            }
            Err(e) => {
                log::error!(
                    "Failed to create ADMIN_BROADCAST notification for user_id {}: {}",
                    user_id,
                    e
                );
            }
        }
    }

    HttpResponse::Ok().body(format!(
        "Notificação enviada a {} utilizadores.",
        notified_user_count
    ))
}

// Handler to get the count of unread notifications for the current user
pub async fn get_unread_count(state: web::Data<State>, session: Session) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    match Notification::count_unread_by_user(&state.db.pool, user_id).await {
        Ok(count) => json_response(&serde_json::json!({ "count": count })),
        Err(e) => {
            log::error!(
                "Error counting unread notifications for user {}: {}",
                user_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
pub struct MarkReadRequest {
    ids: Vec<u32>,
}

// Handler to mark specific notifications as read
pub async fn mark_notifications_read(
    state: web::Data<State>,
    session: Session,
    body: web::Json<MarkReadRequest>,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    if body.ids.is_empty() {
        return HttpResponse::BadRequest().body("No notification IDs provided");
    }

    match Notification::mark_as_read(&state.db.pool, user_id, &body.ids).await {
        Ok(_rows_affected) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!(
                "Error marking notifications as read for user {}: {}",
                user_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Handler to mark ALL notifications as read for the current user
pub async fn mark_all_notifications_read(
    state: web::Data<State>,
    session: Session,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    match Notification::mark_all_as_read(&state.db.pool, user_id).await {
        Ok(_rows_affected) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!(
                "Error marking all notifications as read for user {}: {}",
                user_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
