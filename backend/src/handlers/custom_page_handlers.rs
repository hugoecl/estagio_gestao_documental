use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::{
    State,
    auth::{is_admin, user_can_manage_page, validate_session},
    models::{
        auth::calculate_user_page_permissions,
        custom_page::{
            CreateCustomPageRequest, CustomPage, RolePermissionRequest, UpdateCustomPageRequest,
        },
        role::Role,
    },
    utils::json_utils::{Json, json_response_with_etag},
};

pub async fn get_group_pages(
    state: web::Data<State>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    // In a more structured approach, this query would be in `CustomPage::get_all_groups(&state.db.pool).await`
    match sqlx::query_as!(
        CustomPage,
        r#"
        SELECT 
            id, name, path, parent_path, is_group as "is_group: bool", description, 
            icon, notify_on_new_record as "notify_on_new_record: bool", requires_acknowledgment as "requires_acknowledgment: bool", created_at as "created_at!", updated_at as "updated_at!"
        FROM custom_pages 
        WHERE is_group = true 
        ORDER BY name
        "#
    )
    .fetch_all(&state.db.pool)
    .await
    {
        Ok(pages) => json_response_with_etag(&pages, &req),
        Err(e) => {
            log::error!("Error fetching group pages: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_custom_pages(
    state: web::Data<State>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    if let Err(resp) = validate_session(&session) {
        return resp;
    }

    match CustomPage::get_all(&state.db.pool).await {
        Ok(pages) => json_response_with_etag(&pages, &req),
        Err(e) => {
            log::error!("Error fetching custom pages: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_custom_page(
    state: web::Data<State>,
    path: web::Path<u32>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let page_id = path.into_inner();

    // Pass user_id to check permissions within get_by_id
    match CustomPage::get_by_id(&state.db.pool, page_id, user_id).await {
        Ok(page_with_details) => json_response_with_etag(&page_with_details, &req),
        Err(sqlx::Error::RowNotFound) => {
            // This error now also means "permission denied" or "not found"
            log::warn!(
                "User {} attempted to access non-existent or forbidden page {}",
                user_id,
                page_id
            );
            HttpResponse::NotFound().finish()
        }
        Err(e) => {
            log::error!("Error fetching custom page {}: {}", page_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_custom_page(
    state: web::Data<State>,
    session: Session,
    data: web::Bytes,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    let Json(custom_page_req): Json<CreateCustomPageRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    if !custom_page_req.is_group && (custom_page_req.path.is_empty() || custom_page_req.path == "/")
    {
        log::error!(
            "Attempted to create a page with an invalid path: {}",
            custom_page_req.path
        );
        return HttpResponse::BadRequest().finish();
    }

    match CustomPage::create(&state.db.pool, &custom_page_req).await {
        Ok(page_id) => HttpResponse::Created().body(page_id.to_string()),
        Err(e) => {
            log::error!("Error creating custom page: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_custom_page(
    state: web::Data<State>,
    path: web::Path<u32>,
    data: web::Bytes,
    session: Session,
) -> impl Responder {
    let page_id = path.into_inner();
    let user_id = match validate_session(&session) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    match user_can_manage_page(&state.db.pool, user_id, page_id).await {
        Ok(can) => {
            if !can {
                return HttpResponse::Forbidden().finish();
            }
        }
        Err(e) => {
            log::error!("Error checking permissions: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let Json(data): Json<UpdateCustomPageRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match CustomPage::update(&state.db.pool, page_id, &data).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error updating custom page: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn delete_custom_page(
    state: web::Data<State>,
    path: web::Path<u32>,
    session: Session,
) -> impl Responder {
    let page_id = path.into_inner();

    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    match CustomPage::delete(&state.db.pool, page_id).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error deleting custom page: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_navigation_menu(
    state: web::Data<State>,
    session: Session,
    req: HttpRequest,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match CustomPage::get_navigation_menu(&state.db.pool, user_id).await {
        Ok(menu) => json_response_with_etag(&menu, &req),
        Err(e) => {
            log::error!("Error fetching navigation menu: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn update_page_permissions(
    state: web::Data<State>,
    path: web::Path<u32>,
    data: web::Bytes,
    session: Session,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    let Json(permissions): Json<Vec<RolePermissionRequest>> = match Json::from_bytes(&data) {
        Ok(json) => json,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match Role::update_page_permissions(&state.db.pool, *path, &permissions).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_custom_page_by_path(
    state: web::Data<State>,
    path_param: web::Path<String>,
    session: Session,
    req: HttpRequest, // Keep req for potential ETag later if needed
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let requested_path = path_param.into_inner();
    // Ensure path starts and ends with '/' for consistent DB lookup, if that's how they are stored
    let formatted_path = format!("/{}", requested_path.trim_matches('/'));
    // Check if the original path ended with a slash, if so, add it back.
    // This handles cases like /contratos vs /contratos/
    let final_path = if requested_path.ends_with('/') && !formatted_path.ends_with('/') {
        format!("{}/", formatted_path)
    } else if !requested_path.ends_with('/') && formatted_path.ends_with('/') {
        // If DB path has trailing slash but request didn't, maybe remove it? Or ensure DB is consistent.
        // Let's assume DB paths might have trailing slashes.
        formatted_path
    } else {
        formatted_path
    };

    // 1. Find page ID by path
    let page_info = match sqlx::query!("SELECT id FROM custom_pages WHERE path = ?", final_path)
        .fetch_optional(&state.db.pool)
        .await
    {
        Ok(Some(record)) => record,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(e) => {
            log::error!("Error fetching page ID by path '{}': {}", final_path, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let page_id = page_info.id;

    // 2. Get full page details (including all role permissions)
    let mut page_with_fields = match CustomPage::get_by_id(&state.db.pool, page_id, user_id).await {
        Ok(page_data) => page_data,
        Err(sqlx::Error::RowNotFound) => return HttpResponse::NotFound().finish(), // Should not happen if ID was found, but good practice
        Err(e) => {
            log::error!("Error fetching page details for ID {}: {}", page_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // 3. Calculate current user's specific permissions for this page
    match calculate_user_page_permissions(&state.db.pool, user_id, &page_with_fields.permissions)
        .await
    {
        Ok(user_perms) => {
            // 4. Check if user can view this page
            if !user_perms.can_view {
                log::warn!(
                    "User {} attempted to view page {} ({}) without permission.",
                    user_id,
                    page_id,
                    final_path
                );
                return HttpResponse::Forbidden().finish(); // Or NotFound() to obscure existence
            }
            // Attach calculated permissions to the response
            page_with_fields.current_user_permissions = Some(user_perms);
        }
        Err(e) => {
            log::error!(
                "Error calculating user permissions for user {} on page {}: {}",
                user_id,
                page_id,
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    }

    // 5. Return the data
    json_response_with_etag(&page_with_fields, &req)
}
