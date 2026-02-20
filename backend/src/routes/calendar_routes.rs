// Routes for calendar events (fixed holidays).

use actix_web::web;

use crate::handlers::calendar_handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/calendar-events")
            .route("", web::get().to(calendar_handlers::get_calendar_events)),
    );
}
