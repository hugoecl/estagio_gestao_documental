use actix_session::Session;
use actix_web::{Error, HttpRequest, error::ErrorUnauthorized, get, web};

use crate::utils::session_utils::validate_session;

pub mod contract_routes;
pub mod ers_routes;
pub mod quality_routes;
pub mod user_routes;
pub mod work_contract_routes;

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
    contract_routes::init(cfg);
    work_contract_routes::init(cfg);
    quality_routes::init(cfg);
    ers_routes::init(cfg);

    cfg.service(serve_files);
}
