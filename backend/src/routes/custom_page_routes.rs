use actix_web::web;

use crate::handlers::{custom_page_handlers, field_handlers};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/custom_pages")
            .route(
                "/by-path/{path:.*}",
                web::get().to(custom_page_handlers::get_custom_page_by_path),
            )
            .route(
                "/menu",
                web::get().to(custom_page_handlers::get_navigation_menu),
            )
            .route("", web::get().to(custom_page_handlers::get_custom_pages))
            .route("", web::post().to(custom_page_handlers::create_custom_page))
            .route(
                "/{page_id}/fields",
                web::get().to(field_handlers::get_page_fields),
            )
            .route(
                "/{page_id}/fields",
                web::post().to(field_handlers::add_page_field),
            )
            .route(
                "/{page_id}/permissions",
                web::put().to(custom_page_handlers::update_page_permissions),
            )
            .route(
                "/{id}",
                web::get().to(custom_page_handlers::get_custom_page),
            )
            .route(
                "/{id}",
                web::put().to(custom_page_handlers::update_custom_page),
            )
            .route(
                "/{id}",
                web::delete().to(custom_page_handlers::delete_custom_page),
            ),
    );
}
