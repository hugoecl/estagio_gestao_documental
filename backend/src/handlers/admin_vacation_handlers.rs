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
            // Format dates for the message
            let start_date_fmt = request_details.start_date.format("%d/%m/%Y").to_string();
            let end_date_fmt = request_details.end_date.format("%d/%m/%Y").to_string();
            
            // Get user name for the notification messages
            let user_name = match sqlx::query!("SELECT username FROM users WHERE id = ?", request_details.user_id)
                .fetch_optional(&state.db.pool)
                .await {
                    Ok(Some(user_row)) => user_row.username,
                    _ => "Um utilizador".to_string(), // Fallback if we can't get the username
                };

            // 1. Create notification for the requesting user
            let notification_type = match action_data.status {
                VacationRequestStatus::Approved => NOTIFICATION_TYPE_VACATION_APPROVED,
                VacationRequestStatus::Rejected => NOTIFICATION_TYPE_VACATION_REJECTED,
                _ => unreachable!(), // We already checked this is not PENDING
            };

            // Prepare notification message based on status
            let user_message = match action_data.status {
                VacationRequestStatus::Approved => {
                    format!("O seu pedido de férias ({} a {}) foi aprovado.", start_date_fmt, end_date_fmt)
                },
                VacationRequestStatus::Rejected => {
                    format!("O seu pedido de férias ({} a {}) foi recusado.", start_date_fmt, end_date_fmt)
                },
                _ => unreachable!(),
            };

            // Create the notification for the user who made the request
            match Notification::create(
                &state.db.pool, 
                request_details.user_id, 
                None,               // record_id - Not used for vacation requests
                Some(request_id),   // vacation_request_id - Using request_id
                None,               // page_id - Not applicable
                None,               // field_id - Not applicable 
                notification_type,
                &user_message,
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
            
            // 2. Only send notifications to colleagues if the request was APPROVED
            // (we don't need to bother colleagues about rejected requests)
            if action_data.status == VacationRequestStatus::Approved {
                // Prepare the message for colleagues
                let colleague_message = format!(
                    "O pedido de férias do seu colega {} ({} a {}) foi aprovado.",
                    user_name, start_date_fmt, end_date_fmt
                );
                
                // Get colleagues in the same vacation role
                match Role::get_colleague_user_ids_in_shared_holiday_roles(&state.db.pool, request_details.user_id).await {
                    Ok(colleague_ids) => {
                        for colleague_id in colleague_ids {
                            // Skip the user who made the request - they already got their own notification
                            if colleague_id == request_details.user_id {
                                continue;
                            }
                            
                            // Send notification to each colleague
                            match Notification::create(
                                &state.db.pool,
                                colleague_id,
                                None,               // record_id - Not used for vacation requests
                                Some(request_id),   // vacation_request_id - Using request_id
                                None,               // page_id - Not applicable
                                None,               // field_id - Not applicable 
                                NOTIFICATION_TYPE_VACATION_APPROVED,
                                &colleague_message,
                                Some(request_details.end_date), // Use end_date as due_date
                            ).await {
                                Ok(_) => {
                                    log::info!(
                                        "Created vacation approval notification for colleague {}, vacation request {}",
                                        colleague_id,
                                        request_id
                                    );
                                },
                                Err(e) => {
                                    log::error!(
                                        "Failed to create vacation approval notification for colleague {}, vacation request {}: {}",
                                        colleague_id,
                                        request_id,
                                        e
                                    );
                                }
                            }
                        }
                    },
                    Err(e) => {
                        log::error!(
                            "Error fetching colleague users for vacation approval notifications: {}",
                            e
                        );
                        // Continue processing even if we can't notify colleagues
                    }
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
