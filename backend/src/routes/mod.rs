use actix_web::web;

pub mod contract_routes;
pub mod user_routes;

pub fn init(cfg: &mut web::ServiceConfig) {
    user_routes::init(cfg);
    contract_routes::init(cfg);
}
