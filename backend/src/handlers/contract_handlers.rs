use actix_multipart::form::{MultipartForm, text::Text};
use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use ahash::RandomState;
use chrono::NaiveDate;
use papaya::HashMap;
use serde::Deserialize;

use crate::{
    State,
    cache::ContractCache,
    models::{contract, location},
    utils::{
        forms::FilesFormRequest,
        json_utils::{Json, json_response_with_etag},
        memory_file::MemoryFile,
        session_utils::validate_session,
    },
};

pub async fn get_contracts(
    session: Session,
    state: web::Data<State>,
    req: HttpRequest,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    json_response_with_etag(&state.cache.contracts, &req)
}

#[derive(MultipartForm)]
pub struct ContractFormRequest {
    #[multipart(rename = "contract-number")]
    contract_number: Text<u32>,
    date: Text<String>,
    #[multipart(rename = "date-range")]
    date_range: Text<String>,
    description: Text<String>,
    files: Vec<MemoryFile>,
    location: Text<i8>,
    service: Text<i8>,
    status: Text<i8>,
    supplier: Text<String>,
    #[multipart(rename = "type")]
    type_of_contract: Text<i8>,
}

pub async fn upload_contract(
    session: Session,
    state: web::Data<State>,
    MultipartForm(form): MultipartForm<ContractFormRequest>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let contract_number = form.contract_number.into_inner();
    let date = NaiveDate::parse_from_str(&form.date.into_inner(), "%d/%m/%Y").unwrap();
    let date_start = &form.date_range[0..10];
    let date_end = &form.date_range[13..23];
    let date_start = NaiveDate::parse_from_str(date_start, "%d/%m/%Y").unwrap();
    let date_end = NaiveDate::parse_from_str(date_end, "%d/%m/%Y").unwrap();
    let description = form.description.into_inner();
    let location_value = form.location.into_inner();
    let location = location::Location::from(location_value);
    let service_value = form.service.into_inner();
    let service = contract::Service::from(service_value);
    let status_value = form.status.into_inner();
    let status = contract::Status::from(status_value);
    let supplier = form.supplier.into_inner();
    let type_value = form.type_of_contract.into_inner();
    let type_of_contract = contract::Type::from(type_value);
    let now = chrono::Utc::now();

    let result = sqlx::query!(
        "INSERT INTO contracts (contract_number, date, date_start, date_end, description, location, service, status, supplier, type, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        contract_number,
        date,
        date_start,
        date_end,
        description,
        location_value,
        service_value,
        status_value,
        supplier,
        type_value,
        now,
        now
    )
    .execute(&state.db.pool)
    .await
    .unwrap();

    let new_contract_id = result.last_insert_id() as u32;

    let base_path = format!("media/contracts/{}", new_contract_id);
    let base_path_clone = base_path.clone();
    let _ = tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(base_path_clone).unwrap();
    })
    .await
    .unwrap();

    let files_length = form.files.len();
    let pinned_contracts_cache = state.cache.contracts.pin();
    let contract_files_cache = HashMap::builder()
        .capacity(files_length)
        .hasher(RandomState::new())
        .build();
    let pinned_contract_files_cache = contract_files_cache.pin();

    let mut file_paths = Vec::with_capacity(files_length);

    for file in form.files.into_iter() {
        let file_path = format!("{}/{}", base_path, file.file_name);
        file_paths.push(file_path.clone());

        tokio::task::spawn_blocking(move || {
            std::fs::write(file_path, file.data).unwrap();
        });
    }

    let mut query_builder =
        sqlx::QueryBuilder::new("INSERT INTO contract_files (contract_id, file_path, uploaded_at)");

    query_builder.push_values(file_paths.iter(), |mut b, file_path| {
        b.push_bind(new_contract_id)
            .push_bind(file_path)
            .push_bind(now);
    });

    let file_result = query_builder.build().execute(&state.db.pool).await.unwrap();
    let first_file_id = file_result.last_insert_id() as u32;

    for (i, file_path) in file_paths.into_iter().enumerate() {
        let file_id = first_file_id + i as u32;

        pinned_contract_files_cache.insert(
            file_id,
            crate::cache::ContractFilesCache {
                path: file_path,
                uploaded_at: now,
            },
        );
    }

    drop(pinned_contract_files_cache);

    pinned_contracts_cache.insert(
        new_contract_id,
        crate::cache::ContractCache {
            contract_number,
            date,
            date_start,
            date_end,
            description,
            location,
            service,
            status,
            supplier,
            type_of_contract,
            created_at: now,
            updated_at: now,
            files: contract_files_cache,
        },
    );

    HttpResponse::Created().body(format!("{},{}", new_contract_id, first_file_id))
}

#[derive(Deserialize)]
pub struct UpdateContractRequest {
    contract_number: u32,
    date: String,
    date_start: String,
    date_end: String,
    description: String,
    location: i8,
    service: i8,
    status: i8,
    supplier: String,
    type_of_contract: i8,
}

pub async fn update_contract(
    session: Session,
    state: web::Data<State>,
    data: web::Bytes,
    contract_id: web::Path<u32>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let contract_id = contract_id.into_inner();

    let pinned_contracts_cache = state.cache.contracts.pin();
    let old_contect = pinned_contracts_cache.get(&contract_id).unwrap();
    if let None = pinned_contracts_cache.get(&contract_id) {
        return HttpResponse::NotFound().finish();
    }

    let Json(req): Json<UpdateContractRequest> = Json::from_bytes(data).unwrap();

    let now = chrono::Utc::now();

    let date = NaiveDate::parse_from_str(&req.date, "%d/%m/%Y").unwrap();
    let date_start = NaiveDate::parse_from_str(&req.date_start, "%d/%m/%Y").unwrap();
    let date_end = NaiveDate::parse_from_str(&req.date_end, "%d/%m/%Y").unwrap();

    let old_files = old_contect.files.clone();

    let contract = ContractCache {
        contract_number: req.contract_number,
        date,
        date_start,
        date_end,
        description: req.description.clone(),
        location: location::Location::from(req.location),
        service: contract::Service::from(req.service),
        status: contract::Status::from(req.status),
        supplier: req.supplier.clone(),
        type_of_contract: contract::Type::from(req.type_of_contract),
        created_at: old_contect.created_at,
        updated_at: now,
        files: old_files,
    };

    pinned_contracts_cache.insert(contract_id, contract);

    drop(pinned_contracts_cache);

    tokio::spawn(async move {
        sqlx::query!(
            "UPDATE contracts SET contract_number = ?, date = ?, date_start = ?, date_end = ?, description = ?, location = ?, service = ?, status = ?, supplier = ?, type = ?, updated_at = ? WHERE id = ?",
            req.contract_number,
            date,
            date_start,
            date_end,
            req.description,
            req.location,
            req.service,
            req.status,
            req.supplier,
            req.type_of_contract,
            now,
            contract_id
        ).execute(&state.db.pool).await.unwrap();
    });

    HttpResponse::Ok().finish()
}

pub async fn delete_contract(
    session: Session,
    state: web::Data<State>,
    contract_id: web::Path<u32>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let contract_id = contract_id.into_inner();

    let pinned_contracts_cache = state.cache.contracts.pin();

    let contract = pinned_contracts_cache.remove(&contract_id);
    if let None = contract {
        return HttpResponse::NotFound().finish();
    }

    tokio::task::spawn_blocking(move || {
        std::fs::remove_dir_all(format!("media/contracts/{}", contract_id))
    });

    drop(pinned_contracts_cache);

    tokio::spawn(async move {
        sqlx::query!("DELETE FROM contracts WHERE id = ?", contract_id)
            .execute(&state.db.pool)
            .await
            .unwrap();
    });

    HttpResponse::Ok().finish()
}

pub async fn upload_contract_files(
    session: Session,
    state: web::Data<State>,
    contract_id: web::Path<u32>,
    MultipartForm(form): MultipartForm<FilesFormRequest>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let contract_id = contract_id.into_inner();
    let files_length = form.files.len();

    let pinned_contracts_cache = state.cache.contracts.pin();
    let contract = match pinned_contracts_cache.get(&contract_id) {
        Some(contract) => contract,
        None => return HttpResponse::NotFound().finish(),
    };

    let base_path = format!("media/contracts/{}", contract_id);
    let now = chrono::Utc::now();

    let mut file_paths = Vec::with_capacity(files_length);

    for file in form.files.into_iter() {
        let file_path = format!("{}/{}", base_path, file.file_name);
        file_paths.push(file_path.clone());

        tokio::task::spawn_blocking(move || {
            std::fs::write(file_path, file.data).unwrap();
        });
    }

    let mut query_builder =
        sqlx::QueryBuilder::new("INSERT INTO contract_files (contract_id, file_path, uploaded_at)");

    query_builder.push_values(file_paths.iter(), |mut b, file_path| {
        b.push_bind(contract_id).push_bind(file_path).push_bind(now);
    });

    let result = query_builder.build().execute(&state.db.pool).await.unwrap();
    let first_file_id = result.last_insert_id() as u32;

    let pinned_contract_files_cache = contract.files.pin();

    for (i, file_path) in file_paths.into_iter().enumerate() {
        let file_id = first_file_id + i as u32;

        pinned_contract_files_cache.insert(
            file_id,
            crate::cache::ContractFilesCache {
                path: file_path,
                uploaded_at: now,
            },
        );
    }

    drop(pinned_contract_files_cache);
    drop(pinned_contracts_cache);

    HttpResponse::Created().body(first_file_id.to_string())
}

pub async fn delete_contract_file(
    session: Session,
    state: web::Data<State>,
    path: web::Path<(u32, u32)>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let (contract_id, file_id) = path.into_inner();

    let pinned_contracts_cache = state.cache.contracts.pin();

    let contract = pinned_contracts_cache.get(&contract_id);
    let contract = if let Some(contract) = contract {
        contract
    } else {
        return HttpResponse::NotFound().finish();
    };

    let pinned_contract_files_cache = contract.files.pin();

    let contract_file = pinned_contract_files_cache.remove(&file_id);
    if let None = contract_file {
        return HttpResponse::NotFound().finish();
    }
    let file_path = contract_file.unwrap().path.clone();
    tokio::task::spawn_blocking(move || std::fs::remove_file(format!("media{}", file_path)));

    drop(pinned_contract_files_cache);
    drop(pinned_contracts_cache);

    tokio::spawn(async move {
        sqlx::query!("DELETE FROM contract_files WHERE id = ?", file_id)
            .execute(&state.db.pool)
            .await
            .unwrap();
    });

    HttpResponse::Ok().finish()
}
