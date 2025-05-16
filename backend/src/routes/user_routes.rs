use actix_web::web;

use crate::handlers::{user_handlers, vacation_handlers}; // Import vacation_handlers

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/register", web::post().to(user_handlers::register))
            .route("/login", web::post().to(user_handlers::login))
            .route("/protected", web::get().to(user_handlers::protected))
            .route("/check", web::post().to(user_handlers::check))
            .route("/logout", web::post().to(user_handlers::logout))
            .route(
                "/analytics",
                web::get().to(user_handlers::get_user_analytics),
            )
            .route("/all", web::get().to(user_handlers::get_users_with_roles))
            .route("/roles", web::post().to(user_handlers::assign_roles))
            // New routes for user settings
            .route("/me", web::get().to(user_handlers::get_current_user_details))
            .route(
                "/me/details",
                web::put().to(user_handlers::update_user_details),
            )
            .route(
                "/me/password",
                web::put().to(user_handlers::change_user_password),
            )
            // Admin routes for specific user modification
            .route(
                "/admin/{user_id}/details",
                web::put().to(user_handlers::admin_update_user_details),
            )
            .route(
                "/admin/{user_id}/password",
                web::put().to(user_handlers::admin_set_user_password),
            )
            .route(
                "/admin/{user_id}",
                web::delete().to(user_handlers::admin_delete_user),
            )
            // Route for fetching user's own remaining vacation days
            .route(
                "/me/vacation-days",
                web::get().to(vacation_handlers::get_my_remaining_vacation_days),
            ),
    );
}
