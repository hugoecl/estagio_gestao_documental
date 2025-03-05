use actix_session::Session;
use actix_web::{Error, HttpRequest, error::ErrorUnauthorized, get, web};

use crate::utils::session_utils::validate_session;

pub mod contract_routes;
pub mod user_routes;

#[get("/media/{filename:.*}")]
async fn serve_files(req: HttpRequest, session: Session) -> Result<actix_files::NamedFile, Error> {
    if let Err(_) = validate_session(&session) {
        return Err(ErrorUnauthorized("Não autorizado"));
    }

    let path = req.match_info().query("filename");
    let file = actix_files::NamedFile::open(path)?;
    Ok(file)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    user_routes::init(cfg);
    contract_routes::init(cfg);
    cfg.service(serve_files);
}
