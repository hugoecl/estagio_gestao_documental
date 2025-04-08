use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;

use crate::{
    State,
    auth::{user_can_create_record, user_can_view_page, validate_session},
    models::{
        field::PageField,
        page_record::{CreatePageRecordRequest, PageRecord},
    },
    utils::json_utils::{Json, json_response},
};

#[derive(Deserialize)]
pub struct RecordSearchQuery {
    search: Option<String>,
}

pub async fn get_page_records(
    state: web::Data<State>,
    path: web::Path<u32>,
    query: web::Query<RecordSearchQuery>,
    session: Session,
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
        Ok(records) => json_response(&records),
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

    json_response(&record_with_files)
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

    let Json(mut create_record_req): Json<CreatePageRecordRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    create_record_req.page_id = page_id;

    match PageRecord::create(&state.db.pool, &create_record_req, user_id as u32).await {
        Ok(id) => HttpResponse::Created().body(id.to_string()),
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
}
