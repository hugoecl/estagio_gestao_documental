use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::{
    State,
    auth::{is_admin, validate_session},
    models::role::{CreateRoleRequest, Role, UpdateRoleRequest},
    utils::json_utils::{Json, json_response, json_response_with_etag},
};

pub async fn get_roles(
    state: web::Data<State>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    match Role::get_all(&state.db.pool).await {
        Ok(roles) => json_response_with_etag(&roles, &req),
        Err(e) => {
            log::error!("Error fetching roles: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_role(
    state: web::Data<State>,
    path: web::Path<u32>,
    session: Session,
) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    match Role::get_by_id(&state.db.pool, path.into_inner()).await {
        Ok(role) => json_response(&role),
        Err(e) => {
            log::error!("Error fetching role: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_role(
    state: web::Data<State>,
    data: web::Bytes,
    session: Session,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    let Json(data): Json<CreateRoleRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match Role::create(&state.db.pool, &data).await {
        Ok(role) => json_response(&role),
        Err(e) => {
            log::error!("Error creating role: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_role(
    state: web::Data<State>,
    data: web::Bytes,
    path: web::Path<u32>,
    session: Session,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    let Json(data): Json<UpdateRoleRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match Role::update(&state.db.pool, path.into_inner(), &data).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error updating role: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_role(
    state: web::Data<State>,
    path: web::Path<u32>,
    session: Session,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    match Role::delete(&state.db.pool, path.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error deleting role: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
