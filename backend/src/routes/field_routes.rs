use actix_web::web;

use crate::handlers::field_handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/fields")
            .route("/types", web::get().to(field_handlers::get_field_types))
            .route("/{field_id}", web::put().to(field_handlers::update_field))
            .route(
                "/{field_id}",
                web::delete().to(field_handlers::delete_field),
            )
            .route(
                "/validations",
                web::get().to(field_handlers::get_validations),
            ),
    );
}
