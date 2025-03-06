use actix_multipart::form::{MultipartForm, text::Text};
use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use ahash::RandomState;
use chrono::NaiveDate;
use papaya::HashMap;

use crate::{
    State,
    models::contract,
    utils::{json_utils::json_response, memory_file::MemoryFile, session_utils::validate_session},
};

pub async fn get_contracts(session: Session, state: web::Data<State>) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    json_response(&state.cache.contracts)
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

    let new_contract_id = state
        .cache
        .last_contract_id
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        + 1;

    let _ = tokio::task::spawn_blocking(move || {
        std::fs::create_dir(format!("media/contracts/{}", new_contract_id))
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
    let now = chrono::Utc::now();

    let new_contract_file_id = state
        .cache
        .last_contract_file_id
        .fetch_add(files_length as u32, std::sync::atomic::Ordering::SeqCst)
        + 1;

    let mut file_names = Vec::with_capacity(files_length);

    for (i, file) in form.files.into_iter().enumerate() {
        let file_id = new_contract_file_id + i as u32;
        pinned_contract_files_cache.insert(
            file_id,
            crate::db::ContractFilesCache {
                path: format!("/media/contracts/{}/{}", new_contract_id, file.file_name),
                uploaded_at: now,
            },
        );
        file_names.push(file.file_name.clone());

        tokio::task::spawn_blocking(move || {
            std::fs::write(
                format!("media/contracts/{}/{}", new_contract_id, file.file_name),
                &file.data,
            )
        });
    }
    drop(pinned_contract_files_cache);
    let contract_number = form.contract_number.into_inner();
    let date = NaiveDate::parse_from_str(&form.date.into_inner(), "%d/%m/%Y").unwrap();
    let (date_start, date_end) = form.date_range.split_once(" - ").unwrap();
    let date_start = NaiveDate::parse_from_str(date_start, "%d/%m/%Y").unwrap();
    let date_end = NaiveDate::parse_from_str(date_end, "%d/%m/%Y").unwrap();
    let description = form.description.into_inner();
    let location_value = form.location.into_inner();
    let location = contract::Location::from(location_value);
    let service_value = form.service.into_inner();
    let service = contract::Service::from(service_value);
    let status_value = form.status.into_inner();
    let status = contract::Status::from(status_value);
    let supplier = form.supplier.into_inner();
    let type_value = form.type_of_contract.into_inner();
    let type_of_contract = contract::Type::from(type_value);

    pinned_contracts_cache.insert(
        new_contract_id,
        crate::db::ContractCache {
            contract_number,
            date,
            date_start,
            date_end,
            description: description.clone(),
            location,
            service,
            status,
            supplier: supplier.clone(),
            type_of_contract,
            created_at: now,
            updated_at: now,
            files: contract_files_cache,
        },
    );
    drop(pinned_contracts_cache);

    tokio::spawn(async move {
        sqlx::query!(
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
        ).execute(&state.db.pool).await.unwrap();

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO contract_files (contract_id, file_path, uploaded_at)",
        );

        query_builder.push_values(file_names, |mut b, file_name| {
            b.push_bind(new_contract_id)
                .push_bind(format!(
                    "/media/contracts/{}/{}",
                    new_contract_id, file_name
                ))
                .push_bind(now);
        });

        query_builder.build().execute(&state.db.pool).await.unwrap();
    });

    HttpResponse::Ok().finish()
}
