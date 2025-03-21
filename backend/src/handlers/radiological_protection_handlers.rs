use actix_multipart::form::{MultipartForm, text::Text};
use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use ahash::RandomState;
use chrono::NaiveDate;
use papaya::HashMap;
use serde::Deserialize;

use crate::{
    State,
    cache::{RadiologicalProtectionLicenseCache, RadiologicalProtectionLicenseFileCache},
    models::location::Location,
    utils::{
        json_utils::{Json, json_response},
        memory_file::MemoryFile,
        session_utils::validate_session,
    },
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

#[derive(MultipartForm)]
pub struct UploadeLicenseRequest {
    scope: Text<String>,
    license_number: Text<u32>,
    start_date: Text<String>,
    end_date: Text<String>,
    location: Text<i8>,
    description: Option<Text<String>>,
    files: Vec<MemoryFile>,
}

pub async fn upload_license(
    session: Session,
    state: web::Data<State>,
    MultipartForm(form): MultipartForm<UploadeLicenseRequest>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let scope = form.scope.into_inner();
    let license_number = form.license_number.into_inner();
    let start_date = NaiveDate::parse_from_str(&form.start_date, "%d/%m/%Y").unwrap();
    let end_date = NaiveDate::parse_from_str(&form.end_date, "%d/%m/%Y").unwrap();
    let location = form.location.into_inner();
    let description = form.description.map(|d| d.into_inner());
    let files = form.files;

    let now = chrono::Utc::now();

    let result = match sqlx::query!(
        "INSERT INTO radiological_protection_licenses (scope, license_number, start_date, end_date, description, location, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        scope,
        license_number,
        start_date,
        end_date,
        description,
        location,
        now,
        now,
    ).execute(&state.db.pool).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Database error during license upload: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let license_id = result.last_insert_id() as u32;

    let base_path = format!("media/radiological_protection/licenses/{}", license_id);
    let base_path_clone = base_path.clone();
    tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(base_path_clone).unwrap();
    })
    .await
    .unwrap();

    let files_length = files.len();
    let pinned_license_cache = state.cache.radiological_protection_licenses.pin();
    let files_cache = HashMap::builder()
        .capacity(files_length)
        .hasher(RandomState::new())
        .build();
    let pinned_license_files_cache = files_cache.pin();

    let mut file_paths = Vec::with_capacity(files_length);

    for file in files.into_iter() {
        let file_path = format!("{}/{}", base_path, file.file_name);
        file_paths.push(file_path.clone());

        tokio::task::spawn_blocking(move || {
            std::fs::write(&file_path, &file.data).unwrap();
        });
    }

    let mut query_builder = sqlx::QueryBuilder::new(
        "INSERT INTO radiological_protection_license_files (license_id, file_path, uploaded_at)",
    );

    query_builder.push_values(file_paths.iter(), |mut b, file_path| {
        b.push_bind(license_id).push_bind(file_path).push_bind(now);
    });

    let file_result = match query_builder.build().execute(&state.db.pool).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Database error during license file upload: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let first_file_id = file_result.last_insert_id() as u32;

    for (i, file_path) in file_paths.into_iter().enumerate() {
        let file_id = first_file_id + i as u32;

        pinned_license_files_cache.insert(
            file_id,
            RadiologicalProtectionLicenseFileCache {
                path: file_path,
                uploaded_at: now,
            },
        );
    }

    drop(pinned_license_files_cache);

    pinned_license_cache.insert(
        license_id,
        RadiologicalProtectionLicenseCache {
            scope,
            license_number,
            start_date,
            end_date,
            description,
            location: Location::from(location),
            created_at: now,
            updated_at: now,
            files: files_cache,
        },
    );

    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
pub struct UpdateLicenseRequest {
    scope: String,
    license_number: u32,
    start_date: String,
    end_date: String,
    location: i8,
    description: Option<String>,
}

pub async fn update_license(
    session: Session,
    state: web::Data<State>,
    data: web::Bytes,
    license_id: web::Path<u32>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let license_id = license_id.into_inner();

    let pinned_license_cache = state.cache.radiological_protection_licenses.pin();
    let license = match pinned_license_cache.get(&license_id) {
        Some(l) => l,
        None => return HttpResponse::NotFound().finish(),
    };

    let Json(data): Json<UpdateLicenseRequest> = Json::from_bytes(data).unwrap();

    let now = chrono::Utc::now();

    let start_date = NaiveDate::parse_from_str(&data.start_date, "%d/%m/%Y").unwrap();
    let end_date = NaiveDate::parse_from_str(&data.end_date, "%d/%m/%Y").unwrap();

    let old_files = license.files.clone();

    let license = RadiologicalProtectionLicenseCache {
        scope: data.scope.clone(),
        license_number: data.license_number,
        start_date,
        end_date,
        location: Location::from(data.location),
        description: data.description.clone(),
        created_at: license.created_at,
        updated_at: now,
        files: old_files,
    };

    pinned_license_cache.insert(license_id, license);

    drop(pinned_license_cache);

    actix_web::rt::spawn(async move {
        sqlx::query!(
            "UPDATE radiological_protection_licenses SET scope = ?, license_number = ?, start_date = ?, end_date = ?, location = ?, description = ?, updated_at = ? WHERE id = ?",
            data.scope,
            data.license_number,
            start_date,
            end_date,
            data.location,
            data.description,
            now,
            license_id,
        ).execute(&state.db.pool).await.unwrap();
    });

    HttpResponse::Ok().finish()
}

pub async fn delete_license(
    session: Session,
    state: web::Data<State>,
    license_id: web::Path<u32>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let license_id = license_id.into_inner();

    let pinned_license_cache = state.cache.radiological_protection_licenses.pin();

    if let None = pinned_license_cache.remove(&license_id) {
        return HttpResponse::NotFound().finish();
    }

    tokio::task::spawn_blocking(move || {
        std::fs::remove_dir_all(format!(
            "media/radiological_protection/licenses/{}",
            license_id
        ))
        .unwrap();
    });

    drop(pinned_license_cache);

    tokio::spawn(async move {
        sqlx::query!(
            "DELETE FROM radiological_protection_licenses WHERE id = ?",
            license_id
        )
        .execute(&state.db.pool)
        .await
        .unwrap();
    });

    HttpResponse::Ok().finish()
}
