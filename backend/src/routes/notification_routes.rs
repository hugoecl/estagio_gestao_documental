use crate::handlers::notification_handlers;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/notifications")
            .route(
                "/unread",
                web::get().to(notification_handlers::get_unread_notifications),
            )
            .route(
                "/unread/count",
                web::get().to(notification_handlers::get_unread_count),
            )
            .route(
                "/read",
                web::post().to(notification_handlers::mark_notifications_read),
            ) // Mark specific as read
            .route(
                "/read/all",
                web::post().to(notification_handlers::mark_all_notifications_read),
            ), // Mark all as read
               // Add more notification-related routes here if needed in the future
    );
}
