use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::{
    State,
    cache::WorkContractCategoryCache,
    utils::{
        json_utils::{Json, json_response_with_etag},
        session_utils::validate_session,
    },
};

pub async fn get_work_contract_categories(
    session: Session,
    state: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }
    json_response_with_etag(&state.cache.work_contract_categories, &req)
}

pub async fn add_work_contract_category(
    session: Session,
    state: web::Data<State>,
    data: web::Bytes,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }
    let Json(category): Json<WorkContractCategoryCache> = Json::from_bytes(data).unwrap();

    let result = sqlx::query!(
        "INSERT INTO work_contract_categories (name, description) VALUES (?, ?)",
        category.name,
        category.description
    )
    .execute(&state.db.pool)
    .await;

    match result {
        Ok(result) => {
            state
                .cache
                .work_contract_categories
                .pin()
                .insert(result.last_insert_id() as u32, category);
            HttpResponse::Created().finish()
        }
        Err(e) => {
            eprintln!(
                "Database error during work contract category creation: {}",
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_work_contract_category(
    session: Session,
    state: web::Data<State>,
    data: web::Bytes,
    id: web::Path<u32>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }
    let Json(category): Json<WorkContractCategoryCache> = Json::from_bytes(data).unwrap();
    let id = id.into_inner();

    let result = sqlx::query!(
        "UPDATE work_contract_categories SET name = ?, description = ? WHERE id = ?",
        category.name,
        category.description,
        id
    )
    .execute(&state.db.pool)
    .await;

    match result {
        Ok(_) => {
            let pinned_cache = state.cache.work_contract_categories.pin();
            if let None = pinned_cache.get(&id) {
                return HttpResponse::NotFound().finish();
            }
            pinned_cache.insert(id, category);
            HttpResponse::NoContent().finish()
        }
        Err(e) => {
            eprintln!("Database error during work contract category update: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_work_contract_category(
    session: Session,
    state: web::Data<State>,
    id: web::Path<u32>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }
    let id = id.into_inner();

    let result = sqlx::query!("DELETE FROM work_contract_categories WHERE id = ?", id)
        .execute(&state.db.pool)
        .await;

    match result {
        Ok(_) => {
            if let None = state.cache.work_contract_categories.pin().remove(&id) {
                return HttpResponse::NotFound().finish();
            }
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            eprintln!(
                "Database error during work contract category deletion: {}",
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_work_contracts(
    session: Session,
    state: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    json_response_with_etag(&state.cache.work_contracts, &req)
}
