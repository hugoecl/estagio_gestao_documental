use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::{
    State,
    auth::is_admin,
    models::{role::Role, vacation_request::VacationRequest},
    utils::json_utils::json_response_with_etag,
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
            // true indicates days were successfully deducted if approved, or request actioned
            let action_verb = if action_data.status
                == crate::models::vacation_request::VacationRequestStatus::Approved
            {
                "aprovado"
            } else {
                "rejeitado"
            };
            HttpResponse::Ok().body(format!("Pedido de férias {} com sucesso.", action_verb))
        }
        Ok(false) => {
            // This case might occur if the request wasn't in PENDING state,
            // or if deducting days failed for an approved request.
            log::warn!(
                "action_request_with_days_deduction returned false for request_id: {}, action: {:?}",
                request_id,
                action_data.status
            );
            HttpResponse::Conflict().body(
                "Não foi possível processar o pedido. Pode já ter sido processado ou ocorreu um erro ao deduzir os dias.",
            )
        }
        Err(e) => {
            log::error!("Error actioning vacation request {}: {}", request_id, e);
            if e.to_string().contains("Not enough vacation days available") {
                return HttpResponse::BadRequest().body(e.to_string());
            }
            HttpResponse::InternalServerError().finish()
        }
    }
}
