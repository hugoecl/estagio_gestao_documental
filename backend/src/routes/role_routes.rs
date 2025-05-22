use actix_web::web;

use crate::handlers::role_handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/roles")
            .route("", web::get().to(role_handlers::get_roles))
            .route("/with-interfering", web::get().to(role_handlers::get_roles_with_interfering_roles))
            .route("/{id}", web::get().to(role_handlers::get_role))
            .route("/{id}/with-interfering", web::get().to(role_handlers::get_role_with_interfering_roles))
            .route("", web::post().to(role_handlers::create_role))
            .route("/{id}", web::put().to(role_handlers::update_role))
            .route("/{id}", web::delete().to(role_handlers::delete_role)),
    );
}
