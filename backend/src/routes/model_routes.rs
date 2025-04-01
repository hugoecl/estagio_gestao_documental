use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/models").route(
        "/",
        web::get().to(crate::handlers::model_handlers::get_models),
    ));
}
