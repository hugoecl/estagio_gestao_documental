use actix_web::web;

use crate::handlers::{record_handlers, acknowledgment_handlers}; // Add acknowledgment_handlers

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/records")
            // Standard Record Routes
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
            )
            .route(
                "/pages/{page_id}/records",
                web::get().to(record_handlers::get_page_records),
            )
            .route(
                "/pages/{page_id}/records",
                web::post().to(record_handlers::create_record),
            )
            // Acknowledgment Routes (now under the same /records scope)
            .route(
                "/{record_id}/acknowledge",
                web::post().to(acknowledgment_handlers::acknowledge_record),
            )
            .route(
                "/{record_id}/acknowledgment-status",
                web::get().to(acknowledgment_handlers::check_acknowledgment_status),
            )
            .route(
                "/{record_id}/acknowledgments",
                web::get().to(acknowledgment_handlers::get_acknowledgments_for_record),
            ),
    );
}
