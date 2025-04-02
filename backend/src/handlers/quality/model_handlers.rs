use actix_multipart::form::{MultipartForm, text::Text};
use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use ahash::RandomState;
use log::error;
use papaya::HashMap;
use serde::Deserialize;

use crate::{
    State,
    utils::{
        forms::FilesFormRequest,
        json_utils::{Json, json_response_with_etag},
        memory_file::MemoryFile,
        session_utils::validate_session,
    },
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

#[derive(MultipartForm)]
pub struct ModelFormRequest {
    name: Text<String>,
    version: Text<String>,
    model: Text<String>,
    description: Option<Text<String>>,
    files: Vec<MemoryFile>,
}

pub async fn upload_model(
    session: Session,
    state: web::Data<State>,
    MultipartForm(form): MultipartForm<ModelFormRequest>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let name = form.name.into_inner();
    let version = form.version.into_inner();
    let model = form.model.into_inner();
    let description = form
        .description
        .map(actix_multipart::form::text::Text::into_inner);

    let now = chrono::Utc::now();

    let result = sqlx::query!(
        "INSERT INTO models (name, version, model, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        name,
        version,
        model,
        description,
        now,
        now
    ).execute(&state.db.pool).await.unwrap();

    let new_model_id = result.last_insert_id() as u32;

    let base_path = format!("media/quality/models/{new_model_id}");
    let base_path_clone = base_path.clone();
    tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(base_path_clone).unwrap();
    })
    .await
    .unwrap();

    let files_length = form.files.len();
    let pinned_models_cache = state.cache.models.pin();
    let model_files_cache = HashMap::builder()
        .capacity(files_length)
        .hasher(RandomState::new())
        .build();
    let pinned_model_files_cache = model_files_cache.pin();

    let mut file_paths = Vec::with_capacity(files_length);

    for file in form.files {
        let file_path = format!("{}/{}", base_path, file.file_name);
        file_paths.push(file_path.clone());

        tokio::task::spawn_blocking(move || {
            std::fs::write(file_path, file.data).unwrap();
        });
    }

    let mut query_builder =
        sqlx::QueryBuilder::new("INSERT INTO model_files (model_id, file_path, uploaded_at)");
    query_builder.push_values(file_paths.iter(), |mut b, file_path| {
        b.push_bind(new_model_id)
            .push_bind(file_path)
            .push_bind(now);
    });

    let file_result = query_builder.build().execute(&state.db.pool).await.unwrap();
    let first_file_id = file_result.last_insert_id() as u32;

    for (i, file_path) in file_paths.into_iter().enumerate() {
        let file_id = first_file_id + i as u32;

        pinned_model_files_cache.insert(
            file_id,
            crate::cache::ModelFileCache {
                path: file_path,
                uploaded_at: now,
            },
        );
    }

    drop(pinned_model_files_cache);

    pinned_models_cache.insert(
        new_model_id,
        crate::cache::ModelCache {
            name,
            description,
            version,
            model,
            created_at: now,
            updated_at: now,
            files: model_files_cache,
        },
    );

    HttpResponse::Created().body(format!("{new_model_id},{first_file_id}"))
}

#[derive(Deserialize)]
pub struct UpdateModelRequest {
    name: String,
    version: String,
    model: String,
    description: Option<String>,
}

pub async fn update_model(
    session: Session,
    state: web::Data<State>,
    data: web::Bytes,
    model_id: web::Path<u32>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let model_id = model_id.into_inner();

    let pinned_models_cache = state.cache.models.pin();

    let Some(old_model) = pinned_models_cache.get(&model_id) else {
        return HttpResponse::NotFound().finish();
    };

    let Json(req): Json<UpdateModelRequest> = match Json::from_bytes(&data) {
        Ok(json) => json,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let now = chrono::Utc::now();

    let old_files = old_model.files.clone();

    let updated_model = crate::cache::ModelCache {
        name: req.name.clone(),
        version: req.version.clone(),
        model: req.model.clone(),
        description: req.description.clone(),
        created_at: old_model.created_at,
        updated_at: now,
        files: old_files,
    };

    pinned_models_cache.insert(model_id, updated_model);
    drop(pinned_models_cache);

    tokio::spawn(async move {
        let result = sqlx::query!(
            "UPDATE models SET name = ?, version = ?, model = ?, description = ?, updated_at = ? WHERE id = ?",
            req.name,
            req.version,
            req.model,
            req.description,
            now,
            model_id
        )
        .execute(&state.db.pool)
        .await;

        if let Err(e) = result {
            error!("Error updating model in database: {e}");
        }
    });

    HttpResponse::Ok().finish()
}

pub async fn upload_model_files(
    session: Session,
    state: web::Data<State>,
    model_id: web::Path<u32>,
    MultipartForm(form): MultipartForm<FilesFormRequest>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    let model_id = model_id.into_inner();
    let files_length = form.files.len();

    let pinned_models_cache = state.cache.models.pin();

    let Some(model) = pinned_models_cache.get(&model_id) else {
        return HttpResponse::NotFound().finish();
    };

    let base_path = format!("media/quality/models/{model_id}");
    let now = chrono::Utc::now();

    let mut file_paths = Vec::with_capacity(files_length);

    for file in form.files {
        let file_path = format!("{}/{}", base_path, file.file_name);
        file_paths.push(file_path.clone());

        tokio::task::spawn_blocking(move || {
            std::fs::write(file_path, file.data).unwrap();
        });
    }

    let mut query_builder =
        sqlx::QueryBuilder::new("INSERT INTO model_files (model_id, file_path, uploaded_at)");

    query_builder.push_values(file_paths.iter(), |mut b, file_path| {
        b.push_bind(model_id).push_bind(file_path).push_bind(now);
    });

    let result = query_builder.build().execute(&state.db.pool).await.unwrap();
    let first_file_id = result.last_insert_id() as u32;

    let pinned_model_files_cache = model.files.pin();

    for (i, file_path) in file_paths.into_iter().enumerate() {
        let file_id = first_file_id + i as u32;

        pinned_model_files_cache.insert(
            file_id,
            crate::cache::ModelFileCache {
                path: file_path,
                uploaded_at: now,
            },
        );
    }

    drop(pinned_model_files_cache);
    drop(pinned_models_cache);

    HttpResponse::Created().body(first_file_id.to_string())
}
