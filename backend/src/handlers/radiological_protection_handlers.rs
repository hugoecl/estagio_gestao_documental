use actix_session::Session;
use actix_web::{Responder, web};

use crate::{
    State,
    utils::{json_utils::json_response, session_utils::validate_session},
};

pub async fn get_radiological_protection_licenses(
    session: Session,
    state: web::Data<State>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    json_response(&state.cache.radiological_protection_licenses)
}
