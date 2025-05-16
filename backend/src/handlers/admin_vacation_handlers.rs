use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::{
    State,
    auth::is_admin,
    models::{notification::Notification, role::Role, vacation_request::{VacationRequest, VacationRequestStatus}},
    utils::json_utils::json_response_with_etag,
};

// Use notification constants from the Notification module
use crate::models::notification::{
    NOTIFICATION_TYPE_VACATION_APPROVED,
    NOTIFICATION_TYPE_VACATION_REJECTED,
};

// Handler to get all roles marked as "holiday roles"
pub async fn get_holiday_roles(
    state: web::Data<State>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    if let Err(admin_check_response) = is_admin(&session) {
        return admin_check_response;
    }

    match Role::get_holiday_roles(&state.db.pool).await {
        Ok(roles) => json_response_with_etag(&roles, &req),
        Err(e) => {
            log::error!("Error fetching holiday roles: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Handler to get pending vacation requests for users within a specific role
pub async fn get_pending_requests_for_role(
    state: web::Data<State>,
    session: Session,
    req: HttpRequest,
    role_id_path: web::Path<u32>,
) -> impl Responder {
    if let Err(admin_check_response) = is_admin(&session) {
        return admin_check_response;
    }

    let role_id = role_id_path.into_inner();

    // 1. Get user IDs for the given role_id
    let user_ids = match Role::get_user_ids_by_role_id(&state.db.pool, role_id).await {
        Ok(ids) => {
            if ids.is_empty() {
                // No users in this role, so no pending requests to fetch
                return json_response_with_etag(&Vec::<VacationRequest>::new(), &req);
            }
            ids
        }
        Err(e) => {
            log::error!("Error fetching user IDs for role {}: {}", role_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // 2. Fetch pending vacation requests for these user IDs, including user details
    match VacationRequest::get_pending_requests_with_users(&state.db.pool, Some(&user_ids)).await {
        Ok(requests_with_users) => json_response_with_etag(&requests_with_users, &req),
        Err(e) => {
            log::error!(
                "Error fetching pending requests for role {}: {}",
                role_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Placeholder for future admin handlers related to vacations:
// - Get all vacation requests for a user (admin view)

// Handler for an admin to action a vacation request (approve or reject)
pub async fn action_vacation_request_admin(
    state: web::Data<State>,
    session: Session,
    request_id_path: web::Path<u32>,
    data: web::Json<crate::models::vacation_request::ActionVacationRequest>,
) -> impl Responder {
    let admin_user_id = match is_admin(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    let request_id = request_id_path.into_inner();
    let action_data = data.into_inner();

    if action_data.status == crate::models::vacation_request::VacationRequestStatus::Pending {
        return HttpResponse::BadRequest()
            .body("Admin action cannot set status to PENDING. Use APPROVE or REJECT.");
    }

    // First, fetch the vacation request to get the user_id and other details
    let request_details = match VacationRequest::get_by_id(&state.db.pool, request_id).await {
        Ok(Some(request)) => request,
        Ok(None) => return HttpResponse::NotFound().body("Vacation request not found"),
        Err(e) => {
            log::error!("Error fetching vacation request {}: {}", request_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Action the request
    match crate::models::vacation_request::VacationRequest::action_request_with_days_deduction(
        &state.db.pool,
        request_id,
        admin_user_id,
        action_data.status,
        action_data.admin_notes,
    )
    .await
    {
        Ok(true) => {
            // Create notification for the user
            let notification_type = match action_data.status {
                VacationRequestStatus::Approved => NOTIFICATION_TYPE_VACATION_APPROVED,
                VacationRequestStatus::Rejected => NOTIFICATION_TYPE_VACATION_REJECTED,
                _ => unreachable!(), // We already checked this is not PENDING
            };

            // Format dates for the message
            let start_date_fmt = request_details.start_date.format("%d/%m/%Y").to_string();
            let end_date_fmt = request_details.end_date.format("%d/%m/%Y").to_string();

            // Prepare notification message based on status
            let message = match action_data.status {
                VacationRequestStatus::Approved => {
                    format!("O seu pedido de férias ({} a {}) foi aprovado.", start_date_fmt, end_date_fmt)
                },
                VacationRequestStatus::Rejected => {
                    format!("O seu pedido de férias ({} a {}) foi recusado.", start_date_fmt, end_date_fmt)
                },
                _ => unreachable!(),
            };

            // Create the notification
            match Notification::create(
                &state.db.pool, 
                request_details.user_id, 
                None,               // record_id - Not used for vacation requests
                Some(request_id),   // vacation_request_id - Using request_id
                None,               // page_id - Not applicable
                None,               // field_id - Not applicable 
                notification_type,
                &message,
                Some(request_details.end_date), // Use end_date as due_date
            ).await {
                Ok(_) => {
                    log::info!(
                        "Created vacation notification for user {}, vacation request {}",
                        request_details.user_id,
                        request_id
                    );
                },
                Err(e) => {
                    log::error!(
                        "Failed to create vacation notification for user {}, vacation request {}: {}",
                        request_details.user_id,
                        request_id,
                        e
                    );
                    // Continue processing even if notification fails
                }
            }

            // Success response
            HttpResponse::Ok().body("Vacation request actioned successfully")
        }
        Ok(false) => HttpResponse::BadRequest().body(
            "Vacation request was already actioned or not found when trying to update status.",
        ),
        Err(e) => {
            log::error!("Error actioning vacation request {}: {}", request_id, e);
            HttpResponse::InternalServerError().body(format!("Error: {}", e))
        }
    }
}
