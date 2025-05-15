use crate::handlers::admin_vacation_handlers;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/vacations") // Base path for admin vacation management
            .route(
                "/roles", // Endpoint to get holiday roles
                web::get().to(admin_vacation_handlers::get_holiday_roles),
            ), // Future routes for admin vacation management:
               // .route("/requests", web::get().to(admin_vacation_handlers::get_all_pending_requests))
               // .route("/requests/role/{role_id}", web::get().to(admin_vacation_handlers::get_pending_requests_for_role))
               // .route("/request/{request_id}/action", web::put().to(admin_vacation_handlers::action_vacation_request))
    );
}
