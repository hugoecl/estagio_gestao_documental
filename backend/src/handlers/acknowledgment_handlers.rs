use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use serde::Serialize;

use crate::{
    State,
    auth::{user_can_view_page, validate_session},
    models::{
        custom_page::CustomPage, // Added for fetching user-specific page permissions
        page_record::PageRecord, // To get page_id from record_id
        record_acknowledgment::RecordAcknowledgment,
    },
    utils::json_utils::json_response,
};

// Handler for a user to acknowledge a record
pub async fn acknowledge_record(
    state: web::Data<State>,
    record_id_path: web::Path<u32>,
    session: Session,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };
    let record_id = record_id_path.into_inner();

    // First, check if the user can even view the page this record belongs to
    let page_id = match PageRecord::get_page_id_for_record(&state.db.pool, record_id).await {
        Ok(id) => id,
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().body("Record not found to determine page access.");
        }
        Err(e) => {
            log::error!("Failed to get page_id for record {}: {}", record_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match user_can_view_page(&state.db.pool, user_id as i32, page_id).await {
        Ok(can_view) => {
            if !can_view {
                log::warn!(
                    "User {} attempted to acknowledge record {} on page {} without view permission.",
                    user_id,
                    record_id,
                    page_id
                );
                return HttpResponse::Forbidden()
                    .body("You do not have permission to view this record.");
            }
        }
        Err(e) => {
            log::error!(
                "Error checking view permission for user {} on page {}: {}",
                user_id,
                page_id,
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    }

    // Attempt to create the acknowledgment
    match RecordAcknowledgment::create(&state.db.pool, user_id, record_id).await {
        Ok(true) => HttpResponse::Created().body("Record acknowledged successfully."),
        Ok(false) => HttpResponse::Ok().body("Record already acknowledged by this user."),
        Err(e) => {
            log::error!(
                "Error creating record acknowledgment for user {}, record {}: {}",
                user_id,
                record_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize)]
struct AcknowledgmentStatusResponse {
    acknowledged: bool,
}

// Handler to check if the current user has acknowledged a specific record
pub async fn check_acknowledgment_status(
    state: web::Data<State>,
    record_id_path: web::Path<u32>,
    session: Session,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };
    let record_id = record_id_path.into_inner();

    // Optional: Could also check if user can view the page here, for consistency.
    // However, if they are checking status, they likely expect to see it.

    match RecordAcknowledgment::has_user_acknowledged(&state.db.pool, user_id, record_id).await {
        Ok(has_acknowledged) => json_response(&AcknowledgmentStatusResponse {
            acknowledged: has_acknowledged,
        }),
        Err(e) => {
            log::error!(
                "Error checking acknowledgment status for user {}, record {}: {}",
                user_id,
                record_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Handler to get all acknowledgments for a specific record
pub async fn get_acknowledgments_for_record(
    state: web::Data<State>,
    record_id_path: web::Path<u32>,
    session: Session,
) -> impl Responder {
    let user_id_from_session = match validate_session(&session) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let record_id = record_id_path.into_inner();

    let page_id = match PageRecord::get_page_id_for_record(&state.db.pool, record_id).await {
        Ok(id) => id,
        Err(sqlx::Error::RowNotFound) => return HttpResponse::NotFound().body("Record not found."),
        Err(e) => {
            log::error!("Failed to get page_id for record {}: {}", record_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Check if the user has permission to view acknowledgments for this page
    match CustomPage::get_user_permissions_for_page(&state.db.pool, user_id_from_session, page_id)
        .await
    {
        Ok(permissions) => {
            if !permissions.is_admin && !permissions.can_view_acknowledgments {
                log::warn!(
                    "User {} attempted to view acknowledgments for record {} on page {} without can_view_acknowledgments permission.",
                    user_id_from_session,
                    record_id,
                    page_id
                );
                return HttpResponse::Forbidden()
                    .body("You do not have permission to view acknowledgments for this page.");
            }
            // User is admin or has specific permission, proceed.
        }
        Err(sqlx::Error::RowNotFound) => {
            // This might happen if page permissions themselves are not found for the user, treat as forbidden
            log::warn!(
                "No page permissions found for user {} on page {} when trying to view acknowledgments for record {}.",
                user_id_from_session,
                page_id,
                record_id
            );
            return HttpResponse::Forbidden().body("Permission data not found.");
        }
        Err(e) => {
            log::error!(
                "Error checking can_view_acknowledgments permission for user {} on page {}: {}",
                user_id_from_session,
                page_id,
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    }

    match RecordAcknowledgment::get_acknowledgments_for_record(&state.db.pool, record_id).await {
        Ok(acks) => json_response(&acks),
        Err(e) => {
            log::error!(
                "Error fetching acknowledgments for record {}: {}",
                record_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
