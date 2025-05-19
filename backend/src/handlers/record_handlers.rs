use actix_multipart::form::MultipartForm;
use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use serde::Deserialize;

use crate::{
    State,
    auth::{
        get_user_ids_with_view_permission, user_can_create_record, user_can_delete_record,
        user_can_edit_record, user_can_view_page, validate_session,
    },
    models::{
        custom_page::CustomPage, // Added for fetching page details
        field::PageField,
        notification::Notification, // Added for creating notifications
        page_record::{CreatePageRecordRequest, PageRecord, UpdatePageRecordRequest},
    },
    utils::{
        forms::FilesFormRequest,
        json_utils::{Json, json_response_with_etag},
    },
};

#[derive(Deserialize)]
pub struct RecordSearchQuery {
    search: Option<String>,
}

const NOTIFICATION_TYPE_NEW_RECORD: &str = "NEW_RECORD";

pub async fn get_page_records(
    state: web::Data<State>,
    path: web::Path<u32>,
    query: web::Query<RecordSearchQuery>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(user_id) => user_id,
        Err(resp) => return resp,
    };

    let page_id = path.into_inner();

    match user_can_view_page(&state.db.pool, user_id, page_id).await {
        Ok(can_view) => {
            if !can_view {
                return HttpResponse::Unauthorized().finish();
            }
        }
        Err(e) => {
            log::error!("Error checking user permissions: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let records = if let Some(search_term) = &query.search {
        PageRecord::search_records(&state.db.pool, page_id, search_term).await
    } else {
        PageRecord::get_by_page_id(&state.db.pool, page_id).await
    };

    match records {
        Ok(records) => json_response_with_etag(&records, &req),
        Err(e) => {
            log::error!("Error fetching page records: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_record(
    state: web::Data<State>,
    path: web::Path<u32>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(user_id) => user_id,
        Err(resp) => return resp,
    };

    let record_id = path.into_inner();

    let record_with_files = match PageRecord::get_by_id(&state.db.pool, record_id).await {
        Ok(record) => record,
        Err(e) => {
            log::error!("Error fetching page record: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match user_can_view_page(&state.db.pool, user_id, record_with_files.record.page_id).await {
        Ok(can_view) => {
            if !can_view {
                return HttpResponse::Unauthorized().finish();
            }
        }
        Err(e) => {
            log::error!("Error checking user permissions: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    json_response_with_etag(&record_with_files, &req)
}

pub async fn create_record(
    state: web::Data<State>,
    path: web::Path<u32>,
    data: web::Bytes,
    session: Session,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(user_id) => user_id,
        Err(resp) => return resp,
    };
    let page_id = path.into_inner();

    match user_can_create_record(&state.db.pool, user_id, page_id).await {
        Ok(can_create) => {
            if !can_create {
                return HttpResponse::Forbidden().finish();
            }
        }
        Err(e) => {
            log::error!("Error checking user permissions: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let Json(create_record_req): Json<CreatePageRecordRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match PageRecord::create(&state.db.pool, &create_record_req, page_id, user_id as u32).await {
        Ok(new_record_id) => {
            // Attempt to send notifications if configured for the page
            // We'll clone necessary data for the async block
            let pool_clone = state.db.pool.clone();
            actix_web::rt::spawn(async move {
                match CustomPage::get_by_id(&pool_clone, page_id, user_id).await {
                    // user_id used for permission check in get_by_id
                    Ok(page_details) => {
                        if page_details.page.notify_on_new_record {
                            log::info!(
                                "Page {} is configured to notify on new records. Attempting to notify for record {}.",
                                page_id,
                                new_record_id
                            );
                            match get_user_ids_with_view_permission(&pool_clone, page_id).await {
                                Ok(user_ids_to_notify) => {
                                    if user_ids_to_notify.is_empty() {
                                        log::info!(
                                            "No users found with view permission for page {} to notify.",
                                            page_id
                                        );
                                    }
                                    for notified_user_id in user_ids_to_notify {
                                        // Avoid notifying the user who created the record about their own action
                                        if notified_user_id == user_id as u32 {
                                            log::trace!(
                                                "Skipping notification for user {} (creator) for record {}",
                                                notified_user_id,
                                                new_record_id
                                            );
                                            continue;
                                        }

                                        // It's a new record, so a specific check for existing "NEW_RECORD" notifications
                                        // for this exact record ID is usually not needed unless the process could somehow run twice.
                                        // For simplicity here, we assume it won't run twice for the same creation event.
                                        let message = format!(
                                            "Novo registo #{} adicionado a '{}'.",
                                            new_record_id, page_details.page.name
                                        );
                                        match Notification::create(
                                            &pool_clone,
                                            notified_user_id,
                                            Some(new_record_id),
                                            None,               // No vacation_request_id
                                            Some(page_id),
                                            None, // No specific field for a general "new record" notification
                                            NOTIFICATION_TYPE_NEW_RECORD,
                                            &message,
                                            None, // No specific due date for this type
                                        )
                                        .await
                                        {
                                            Ok(_) => log::info!(
                                                "Created NEW_RECORD notification for user {} for record {} on page {}.",
                                                notified_user_id,
                                                new_record_id,
                                                page_id
                                            ),
                                            Err(e) => log::error!(
                                                "Failed to create NEW_RECORD notification for user {} for record {}: {}",
                                                notified_user_id,
                                                new_record_id,
                                                e
                                            ),
                                        }
                                    }
                                }
                                Err(e) => log::error!(
                                    "Failed to get users with view permission for page {}: {}",
                                    page_id,
                                    e
                                ),
                            }
                        } else {
                            log::trace!(
                                "Page {} is not configured to notify on new records.",
                                page_id
                            );
                        }
                    }
                    Err(e) => log::error!(
                        "Failed to fetch page details for page_id {} during new record notification: {}",
                        page_id,
                        e
                    ),
                }
            });
            HttpResponse::Created().body(new_record_id.to_string())
        }
        Err(e) => {
            log::error!("Error creating page record: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
pub async fn update_record(
    state: web::Data<State>,
    path: web::Path<u32>,
    data: web::Bytes,
    session: Session,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(user_id) => user_id,
        Err(resp) => return resp,
    };
    let record_id = path.into_inner();

    let record_with_files = match PageRecord::get_by_id(&state.db.pool, record_id).await {
        Ok(record) => record,
        Err(e) => {
            log::error!("Error fetching page record: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let page_id = record_with_files.record.page_id;

    match user_can_edit_record(&state.db.pool, user_id, page_id).await {
        Ok(can_edit) => {
            if !can_edit {
                return HttpResponse::Forbidden().finish();
            }
        }
        Err(e) => {
            log::error!("Error checking user permissions: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let Json(data): Json<UpdatePageRecordRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    let page_fields = match PageField::get_by_page_id(&state.db.pool, page_id).await {
        Ok(fields) => fields,
        Err(e) => {
            log::error!("Error fetching page fields: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if let serde_json::Value::Object(data_map) = &data.data {
        for field in &page_fields {
            if field.required {
                if !data_map.contains_key(&field.name) || data_map[&field.name].is_null() {
                    return HttpResponse::BadRequest()
                        .body(format!("Field {} is required", field.name));
                }
            }
        }
    }

    match PageRecord::update(&state.db.pool, record_id, &data, user_id as u32).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error updating page record: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_record(
    state: web::Data<State>,
    path: web::Path<u32>,
    session: Session,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(user_id) => user_id,
        Err(resp) => return resp,
    };

    let record_id = path.into_inner();

    let record_with_files = match PageRecord::get_by_id(&state.db.pool, record_id).await {
        Ok(record) => record,
        Err(e) => {
            log::error!("Error fetching page record: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match user_can_delete_record(&state.db.pool, user_id, record_with_files.record.page_id).await {
        Ok(can_delete) => {
            if !can_delete {
                return HttpResponse::Forbidden().finish();
            }
        }
        Err(e) => {
            log::error!("Error checking user permissions: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    for file in record_with_files.files {
        tokio::task::spawn_blocking(move || {
            std::fs::remove_file(&file.file_path).unwrap();
        });
    }

    match PageRecord::delete(&state.db.pool, record_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            log::error!("Error deleting page record: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn upload_record_files(
    state: web::Data<State>,
    path: web::Path<u32>,
    MultipartForm(form): MultipartForm<FilesFormRequest>,
    session: Session,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(user_id) => user_id,
        Err(resp) => return resp,
    };

    let record_id = path.into_inner();

    let record_with_files = match PageRecord::get_by_id(&state.db.pool, record_id).await {
        Ok(record) => record,
        Err(e) => {
            log::error!("Error fetching page record: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match user_can_edit_record(&state.db.pool, user_id, record_with_files.record.page_id).await {
        Ok(can_edit) => {
            if !can_edit {
                return HttpResponse::Forbidden().finish();
            }
        }
        Err(e) => {
            log::error!("Error checking user permissions: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let base_path = format!("media/page_records/{}/files", record_id);
    let base_path_clone = base_path.clone();

    tokio::task::spawn_blocking(move || {
        std::fs::create_dir_all(base_path_clone).unwrap_or_else(|e| {
            log::error!("Error creating directory: {}", e);
        });
    })
    .await
    .unwrap();

    let mut first_file_id: Option<u32> = None;

    for file in form.files {
        let file_name = file.file_name.clone();
        let file_path = format!("{}/{}", base_path, file_name);

        let file_path_clone = file_path.clone();
        let file_data = file.data.clone();

        tokio::task::spawn_blocking(move || {
            std::fs::write(&file_path_clone, &file_data).unwrap_or_else(|e| {
                log::error!("Error writing file: {}", e);
            });
        });

        match PageRecord::add_file(
            &state.db.pool,
            record_id,
            &file_name,
            &file_path,
            user_id as u32,
        )
        .await
        {
            Ok(file_id) => {
                if first_file_id.is_none() {
                    first_file_id = Some(file_id);
                }
            }
            Err(e) => {
                log::error!("Error adding file to record: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        }
    }

    if let Some(id) = first_file_id {
        HttpResponse::Created().body(id.to_string())
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub async fn delete_record_file(
    state: web::Data<State>,
    path: web::Path<(u32, u32)>,
    session: Session,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(user_id) => user_id,
        Err(resp) => return resp,
    };

    let (record_id, file_id) = path.into_inner();

    let record_with_files = match PageRecord::get_by_id(&state.db.pool, record_id).await {
        Ok(record) => record,
        Err(e) => {
            log::error!("Error fetching page record: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match user_can_edit_record(&state.db.pool, user_id, record_with_files.record.page_id).await {
        Ok(can_edit) => {
            if !can_edit {
                return HttpResponse::Forbidden().finish();
            }
        }
        Err(e) => {
            log::error!("Error checking user permissions: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let file_to_delete = record_with_files
        .files
        .into_iter()
        .find(|f| f.id == file_id);

    if let Some(file) = file_to_delete {
        let file_path = file.file_path;

        tokio::task::spawn_blocking(move || {
            std::fs::remove_file(&file_path).unwrap_or_else(|e| {
                log::error!("Error deleting file: {}", e);
            });
        });

        match PageRecord::delete_file(&state.db.pool, file_id).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(e) => {
                log::error!("Error deleting file from record: {}", e);
                HttpResponse::InternalServerError().finish()
            }
        }
    } else {
        HttpResponse::NotFound().finish()
    }
}
