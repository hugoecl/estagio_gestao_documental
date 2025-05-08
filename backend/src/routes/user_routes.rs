use actix_web::web;

use crate::handlers::user_handlers;

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
            .route("/roles", web::post().to(user_handlers::assign_roles)),
    );
}
