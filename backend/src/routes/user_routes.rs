use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/users").route(
        "/register",
        web::post().to(crate::handlers::user_handlers::register),
    ));
}
