use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/ers").route(
        "/radiological-protection",
        web::get().to(
            crate::handlers::radiological_protection_handlers::get_radiological_protection_licenses,
        ),
    ));
}
