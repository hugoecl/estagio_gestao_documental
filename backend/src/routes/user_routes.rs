use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route(
                "/register",
                web::post().to(crate::handlers::user_handlers::register),
            )
            .route(
                "/login",
                web::post().to(crate::handlers::user_handlers::login),
            )
            .route(
                "/protected",
                web::get().to(crate::handlers::user_handlers::protected),
            )
            .route(
                "/check",
                web::post().to(crate::handlers::user_handlers::check),
            )
            .route(
                "/logout",
                web::post().to(crate::handlers::user_handlers::logout),
            )
            .route(
                "/analytics",
                web::get().to(crate::handlers::user_handlers::get_user_analytics),
            ),
    );
}
