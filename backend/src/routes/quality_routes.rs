use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/quality").route(
        "/models",
        web::get().to(crate::handlers::quality::model_handlers::get_models),
    ));
}
