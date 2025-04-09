use actix_web::web;

use crate::handlers::record_handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/records")
            .route("/{record_id}", web::get().to(record_handlers::get_record))
            .route(
                "/{record_id}",
                web::put().to(record_handlers::update_record),
            )
            .route(
                "/{record_id}",
                web::delete().to(record_handlers::delete_record),
            )
            .route(
                "/{record_id}/files",
                web::post().to(record_handlers::upload_record_files),
            )
            .route(
                "/{record_id}/files/{file_id}",
                web::delete().to(record_handlers::delete_record_file),
            ),
    )
    .service(
        web::scope("/pages")
            .route(
                "/{page_id}/records",
                web::get().to(record_handlers::get_page_records),
            )
            .route(
                "/{page_id}/records",
                web::post().to(record_handlers::create_record),
            ),
    );
}
