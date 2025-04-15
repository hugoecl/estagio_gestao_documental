use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::{
    State,
    auth::{is_admin, user_can_manage_page, validate_session},
    models::{
        custom_page::{
            CreateCustomPageRequest, CustomPage, RolePermissionRequest, UpdateCustomPageRequest,
        },
        role::Role,
    },
    utils::json_utils::{Json, json_response_with_etag},
};

pub async fn get_custom_pages(
    state: web::Data<State>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    match CustomPage::get_all(&state.db.pool).await {
        Ok(pages) => json_response_with_etag(&pages, &req),
        Err(e) => {
            log::error!("Error fetching custom pages: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_custom_page(
    state: web::Data<State>,
    path: web::Path<u32>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    match CustomPage::get_by_id(&state.db.pool, path.into_inner()).await {
        Ok(page) => json_response_with_etag(&page, &req),
        Err(e) => {
            log::error!("Error fetching custom page: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_custom_page(
    state: web::Data<State>,
    session: Session,
    data: web::Bytes,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    let Json(custom_page_req): Json<CreateCustomPageRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match CustomPage::create(&state.db.pool, &custom_page_req).await {
        Ok(page_id) => HttpResponse::Created().body(page_id.to_string()),
        Err(e) => {
            log::error!("Error creating custom page: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_custom_page(
    state: web::Data<State>,
    path: web::Path<u32>,
    data: web::Bytes,
    session: Session,
) -> impl Responder {
    let page_id = path.into_inner();
    let user_id = match validate_session(&session) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match user_can_manage_page(&state.db.pool, user_id, page_id).await {
        Ok(can) => {
            if !can {
                return HttpResponse::Forbidden().finish();
            }
        }
        Err(e) => {
            log::error!("Error checking permissions: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let Json(data): Json<UpdateCustomPageRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match CustomPage::update(&state.db.pool, page_id, &data).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error updating custom page: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_custom_page(
    state: web::Data<State>,
    path: web::Path<u32>,
    session: Session,
) -> impl Responder {
    let page_id = path.into_inner();

    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    match CustomPage::delete(&state.db.pool, page_id).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error deleting custom page: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_navigation_menu(
    state: web::Data<State>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    match CustomPage::get_navigation_menu(&state.db.pool).await {
        Ok(menu) => json_response_with_etag(&menu, &req),
        Err(e) => {
            log::error!("Error fetching navigation menu: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_page_permissions(
    state: web::Data<State>,
    path: web::Path<u32>,
    data: web::Bytes,
    session: Session,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    let Json(permissions): Json<Vec<RolePermissionRequest>> = match Json::from_bytes(&data) {
        Ok(json) => json,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match Role::update_page_permissions(&state.db.pool, *path, &permissions).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
