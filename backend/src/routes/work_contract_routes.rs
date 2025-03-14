use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/work_contracts").route(
        "",
        web::get().to(crate::handlers::work_contract_handlers::get),
    ));
}
