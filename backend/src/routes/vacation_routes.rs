// src/routes/vacation_routes.rs
use crate::handlers::vacation_handlers;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/vacation-requests") // Base path for user's own vacation requests
            .route(
                "",
                web::post().to(vacation_handlers::submit_vacation_request),
            )
            .route(
                "/me",
                web::get().to(vacation_handlers::get_my_vacation_requests),
            )
            // Note: /users/me/vacation-days is in user_routes.rs
            .route(
                // New route for shared calendar data
                "/shared-calendar",
                web::get().to(vacation_handlers::get_shared_calendar_vacations),
            )
            .route(
                "/{request_id}",
                web::delete().to(vacation_handlers::cancel_vacation_request),
            ),
    );
    // Admin vacation routes are in admin_vacation_routes.rs
}
