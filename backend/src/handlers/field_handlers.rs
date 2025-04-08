use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::{
    State,
    auth::{user_can_manage_page, validate_session},
    models::{
        custom_page::CreatePageFieldRequest,
        field::{FieldType, PageField, UpdatePageFieldRequest},
        validation,
    },
    utils::json_utils::{Json, json_response, json_response_with_etag},
};

pub async fn get_field_types(state: web::Data<State>, session: Session) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    match FieldType::get_all(&state.db.pool).await {
        Ok(field_types) => json_response(&field_types),
        Err(e) => {
            log::error!("Error fetching field types: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_page_fields(
    state: web::Data<State>,
    session: Session,
    path: web::Path<u32>,
) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    match PageField::get_by_page_id(&state.db.pool, path.into_inner()).await {
        Ok(fields) => json_response(&fields),
        Err(e) => {
            log::error!("Error fetching page fields: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn add_page_field(
    state: web::Data<State>,
    session: Session,
    path: web::Path<u32>,
    data: web::Bytes,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(user_id) => user_id,
        Err(resp) => return resp,
    };

    let page_id = path.into_inner();

    match user_can_manage_page(&state.db.pool, user_id, page_id).await {
        Ok(can) => {
            if !can {
                return HttpResponse::Forbidden().finish();
            }
        }
        Err(resp) => {
            log::error!("Error checking page permissions: {}", resp);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let Json(field_req): Json<CreatePageFieldRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match PageField::create(&state.db.pool, page_id, &field_req).await {
        Ok(field_id) => json_response(&field_id),
        Err(e) => {
            log::error!("Error adding field to page {}: {}", page_id, e);

            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_field(
    state: web::Data<State>,
    session: Session,
    path: web::Path<u32>,
    data: web::Bytes,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(user_id) => user_id,
        Err(resp) => return resp,
    };

    let field_id = path.into_inner();
    let page_id = match sqlx::query!(
        r#"
        SELECT page_id FROM page_fields WHERE id = ?
        "#,
        field_id
    )
    .fetch_one(&state.db.pool)
    .await
    {
        Ok(record) => record.page_id,
        Err(e) => {
            log::error!("Error fetching page ID for field {}: {}", field_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match user_can_manage_page(&state.db.pool, user_id, page_id).await {
        Ok(can) => {
            if !can {
                return HttpResponse::Forbidden().finish();
            }
        }
        Err(resp) => {
            log::error!("Error checking page permissions: {}", resp);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let Json(field_req): Json<UpdatePageFieldRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match PageField::update(&state.db.pool, field_id, &field_req).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error updating field {}: {}", field_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_field(
    state: web::Data<State>,
    session: Session,
    path: web::Path<u32>,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let field_id = path.into_inner();
    let page_id = match sqlx::query!(
        r#"
        SELECT page_id FROM page_fields WHERE id = ?
        "#,
        field_id
    )
    .fetch_one(&state.db.pool)
    .await
    {
        Ok(record) => record.page_id,
        Err(e) => {
            log::error!("Error fetching page ID for field {}: {}", field_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match user_can_manage_page(&state.db.pool, user_id, page_id).await {
        Ok(can) => {
            if !can {
                return HttpResponse::Forbidden().finish();
            }
        }
        Err(resp) => {
            log::error!("Error checking page permissions: {}", resp);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match PageField::delete(&state.db.pool, field_id).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error deleting field {}: {}", field_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_validations(session: Session, req: HttpRequest) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    let validations = validation::get_available_validations();
    json_response_with_etag(&validations, &req)
}
