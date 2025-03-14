use actix_multipart::form::{MultipartForm, text::Text};
use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use ahash::RandomState;
use chrono::NaiveDate;
use papaya::HashMap;
use serde::Deserialize;

use crate::{
    State,
    cache::WorkContractCategoryCache,
    models::{location::Location, work_contract},
    utils::{
        json_utils::{Json, json_response_with_etag},
        memory_file::MemoryFile,
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

#[derive(MultipartForm)]
pub struct WorkContractForm {
    employee_name: Text<String>,
    nif: Text<String>,
    start_date: Text<String>,
    end_date: Option<Text<String>>,
    type_of_contract: Text<i8>,
    location: Text<i8>,
    category_id: Text<u32>,
    description: Option<Text<String>>,
    files: Vec<MemoryFile>,
}

pub async fn upload_work_contract(
    session: Session,
    state: web::Data<State>,
    MultipartForm(form): MultipartForm<WorkContractForm>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let employee_name = form.employee_name.into_inner();
    let nif = form.nif.into_inner();
    let start_date = NaiveDate::parse_from_str(&form.start_date.into_inner(), "%d/%m/%Y").unwrap();
    let end_date = match form.end_date {
        Some(end_date) => Some(NaiveDate::parse_from_str(&end_date, "%d/%m/%Y").unwrap()),
        None => None,
    };

    let type_value = form.type_of_contract.into_inner();
    let location_value = form.location.into_inner();
    let category_id = form.category_id.into_inner();
    let description = form.description.map(|d| d.into_inner());
    let now = chrono::Utc::now();

    let result = sqlx::query!(
        "INSERT INTO work_contracts (employee_name, nif, start_date, end_date, type, location, category_id, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        employee_name,
        nif,
        start_date,
        end_date,
        type_value,
        location_value,
        category_id,
        description,
        now,
        now
    )
    .execute(&state.db.pool)
    .await;

    match result {
        Ok(result) => {
            let new_contract_id = result.last_insert_id() as u32;

            let base_path = format!("media/work_contracts/{}", new_contract_id);
            let base_path_clone = base_path.clone();
            let _ = tokio::task::spawn_blocking(move || {
                std::fs::create_dir_all(base_path_clone).unwrap();
            })
            .await
            .unwrap();

            let files_length = form.files.len();
            let work_contract_files_cache = HashMap::builder()
                .capacity(files_length)
                .hasher(RandomState::new())
                .build();
            let pinned_files_cache = work_contract_files_cache.pin();

            let mut file_paths = Vec::with_capacity(files_length);
            for file in form.files.into_iter() {
                let file_path = format!("{}/{}", base_path, file.file_name);
                file_paths.push(file_path.clone());

                tokio::task::spawn_blocking(move || {
                    std::fs::write(file_path, file.data).unwrap();
                });
            }

            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO work_contract_files (contract_id, file_path, uploaded_at)",
            );

            query_builder.push_values(file_paths.iter(), |mut b, file_path| {
                b.push_bind(new_contract_id)
                    .push_bind(file_path)
                    .push_bind(now);
            });

            let first_file_id = if !file_paths.is_empty() {
                let file_result = query_builder.build().execute(&state.db.pool).await.unwrap();
                let id = file_result.last_insert_id() as u32;

                for (i, file_path) in file_paths.into_iter().enumerate() {
                    let file_id = id + i as u32;
                    pinned_files_cache.insert(
                        file_id,
                        crate::cache::WorkContractFileCache {
                            path: file_path,
                            uploaded_at: now,
                        },
                    );
                }

                Some(id)
            } else {
                None
            };

            drop(pinned_files_cache);

            let contract_cache = crate::cache::WorkContractCache {
                employee_name,
                nif,
                start_date,
                end_date,
                type_of_contract: work_contract::Type::from(type_value),
                location: Location::from(location_value),
                category_id,
                description,
                created_at: now,
                updated_at: now,
                files: work_contract_files_cache,
            };

            state
                .cache
                .work_contracts
                .pin()
                .insert(new_contract_id, contract_cache);

            match first_file_id {
                Some(file_id) => {
                    HttpResponse::Created().body(format!("{},{}", new_contract_id, file_id))
                }
                None => HttpResponse::Created().body(format!("{}", new_contract_id)),
            }
        }
        Err(e) => {
            eprintln!("Database error during work contract creation: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateWorkContractRequest {
    employee_name: String,
    nif: String,
    start_date: String,
    end_date: Option<String>,
    type_of_contract: i8,
    location: i8,
    category_id: u32,
    description: Option<String>,
}

pub async fn update_work_contract(
    session: Session,
    state: web::Data<State>,
    data: web::Bytes,
    contract_id: web::Path<u32>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let contract_id = contract_id.into_inner();

    let pinned_work_contracts_cache = state.cache.work_contracts.pin();

    let old_contract = match pinned_work_contracts_cache.get(&contract_id) {
        Some(contract) => contract,
        None => return HttpResponse::NotFound().finish(),
    };

    let Json(req): Json<UpdateWorkContractRequest> = match Json::from_bytes(data) {
        Ok(json) => json,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let now = chrono::Utc::now();

    let start_date = match NaiveDate::parse_from_str(&req.start_date, "%d/%m/%Y") {
        Ok(date) => date,
        Err(_) => return HttpResponse::BadRequest().body("Invalid start_date format"),
    };

    let end_date = match req.end_date {
        Some(date_str) if !date_str.is_empty() => {
            match NaiveDate::parse_from_str(&date_str, "%d/%m/%Y") {
                Ok(date) => Some(date),
                Err(_) => return HttpResponse::BadRequest().body("Invalid end_date format"),
            }
        }
        _ => None,
    };

    let old_files = old_contract.files.clone();

    let updated_contract = crate::cache::WorkContractCache {
        employee_name: req.employee_name.clone(),
        nif: req.nif.clone(),
        start_date,
        end_date,
        type_of_contract: work_contract::Type::from(req.type_of_contract),
        location: Location::from(req.location),
        category_id: req.category_id,
        description: req.description.clone(),
        created_at: old_contract.created_at,
        updated_at: now,
        files: old_files,
    };

    pinned_work_contracts_cache.insert(contract_id, updated_contract);
    drop(pinned_work_contracts_cache);

    tokio::spawn(async move {
        let result = sqlx::query!(
            "UPDATE work_contracts SET employee_name = ?, nif = ?, start_date = ?, end_date = ?, 
             type = ?, location = ?, category_id = ?, description = ?, updated_at = ? WHERE id = ?",
            req.employee_name,
            req.nif,
            start_date,
            end_date,
            req.type_of_contract,
            req.location,
            req.category_id,
            req.description,
            now,
            contract_id
        )
        .execute(&state.db.pool)
        .await;

        if let Err(e) = result {
            eprintln!("Error updating work contract in database: {}", e);
        }
    });

    HttpResponse::Ok().finish()
}
