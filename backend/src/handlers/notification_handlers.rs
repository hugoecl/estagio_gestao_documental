use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;

use crate::{
    State, auth::validate_session, models::notification::Notification,
    utils::json_utils::json_response,
};

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
        Ok(rows_affected) => {
            log::debug!(
                "Marked {} notifications as read for user {}",
                rows_affected,
                user_id
            );
            HttpResponse::Ok().finish()
        }
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
        Ok(rows_affected) => {
            log::debug!(
                "Marked all ({}) notifications as read for user {}",
                rows_affected,
                user_id
            );
            HttpResponse::Ok().finish()
        }
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
