use actix_web::web::{self};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/contracts")
            .route(
                "",
                web::get().to(crate::handlers::contract_handlers::get_contracts),
            )
            .route(
                "",
                web::post().to(crate::handlers::contract_handlers::upload_contract),
            ),
    );
}
