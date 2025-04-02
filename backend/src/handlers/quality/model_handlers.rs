use actix_multipart::form::{MultipartForm, text::Text};
use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use ahash::RandomState;
use papaya::HashMap;

use crate::{
    State,
    utils::{
        json_utils::json_response_with_etag, memory_file::MemoryFile,
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

pub async fn upload_modal(
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
    let description = form.description.map(actix_multipart::form::text::Text::into_inner);

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
