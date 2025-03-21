use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ers")
        .route(
        "/radiological-protection/licenses",
        web::get().to(crate::handlers::radiological_protection_handlers::get_radiological_protection_licenses))
        .route(
            "/radiological-protection/licenses",
            web::post().to(crate::handlers::radiological_protection_handlers::upload_license),
        ).route(
            "/radiological-protection/licenses/{id}",
            web::put().to(crate::handlers::radiological_protection_handlers::update_license),
        )
    );
}
