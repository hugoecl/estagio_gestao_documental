use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::{
    State, auth::is_admin, models::role::Role, utils::json_utils::json_response_with_etag,
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

// Placeholder for future admin handlers related to vacations:
// - Get pending requests for a specific holiday role (or all)
// - Action a specific vacation request (approve/reject)
// - Get all vacation requests for a user (admin view)
