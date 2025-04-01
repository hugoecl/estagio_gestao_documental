use actix_session::Session;
use actix_web::{HttpRequest, Responder, web};

use crate::{
    State,
    utils::{json_utils::json_response_with_etag, session_utils::validate_session},
};

pub async fn get_models(
    session: Session,
    state: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    json_response_with_etag(&state.cache.models, &req)
}
