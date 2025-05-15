use actix_session::Session;
use actix_web::{Error, HttpRequest, error::ErrorUnauthorized, get, web};

use crate::auth::validate_session;

pub mod admin_vacation_routes;
pub mod custom_page_routes;
pub mod field_routes;
pub mod notification_routes;
pub mod record_routes;
pub mod role_routes;
pub mod user_routes;
pub mod vacation_routes;

#[get("/{filename:.*}")]
async fn serve_files(req: HttpRequest, session: Session) -> Result<actix_files::NamedFile, Error> {
    if validate_session(&session).is_err() {
        return Err(ErrorUnauthorized("Não autorizado"));
    }

    let path = req.match_info().query("filename");

    if !path.starts_with("media") {
        return Err(ErrorUnauthorized("Não autorizado"));
    }

    let file = actix_files::NamedFile::open(path)?;
    Ok(file)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    user_routes::init(cfg);
    custom_page_routes::init(cfg);
    field_routes::init(cfg);
    role_routes::init(cfg);
    record_routes::init(cfg);
    notification_routes::init(cfg);
    vacation_routes::init(cfg); // Added vacation routes
    admin_vacation_routes::init(cfg);

    cfg.service(serve_files);
}
