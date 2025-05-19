use actix_web::web;
use crate::handlers::admin_vacation_handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/vacations") // Base path for admin vacation management
            .route(
                "/roles", // Endpoint to get holiday roles
                web::get().to(admin_vacation_handlers::get_holiday_roles)
            )
            .route( // Route for pending requests for a specific role
                "/role/{role_id}/pending-requests",
                web::get().to(admin_vacation_handlers::get_pending_requests_for_role)
            )
            .route( // Route for an admin to action a specific request
                "/request/{request_id}/action",
                web::put().to(admin_vacation_handlers::action_vacation_request_admin)
            )
        // Future routes could be added here
    );
}