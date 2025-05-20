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
            icon, notify_on_new_record as "notify_on_new_record: bool", requires_acknowledgment as "requires_acknowledgment: bool", display_order, 
            created_at as "created_at!", updated_at as "updated_at!"
        FROM custom_pages 
        WHERE is_group = true 
        ORDER BY display_order, name
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

pub async fn duplicate_custom_page(
    state: web::Data<State>,
    path: web::Path<u32>,
    session: Session,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    let page_id = path.into_inner();
    // Get the user_id from the session (unused but needed for validation)
    let _user_id = match validate_session(&session) {
        Ok(id) => id,
        Err(_) => return HttpResponse::Unauthorized().finish(),
    };
    
    // Fetch the original page using a direct query instead of the get_by_id method
    // to bypass permission checks that might filter it out
    let original_page_data = match sqlx::query!(
        r#"
        SELECT 
            id, name, path, parent_path, is_group, description, 
            icon, notify_on_new_record, requires_acknowledgment
        FROM custom_pages 
        WHERE id = ?
        "#,
        page_id
    )
    .fetch_optional(&state.db.pool)
    .await
    {
        Ok(Some(page)) => page,
        Ok(None) => {
            log::error!("Page with ID {} not found", page_id);
            return HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "Página não encontrada"
            }));
        },
        Err(e) => {
            log::error!("Database error fetching page {}: {}", page_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Start transaction for field and permission operations
    let mut tx = match state.db.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            log::error!("Error starting transaction: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Create new page request
    let mut request = crate::models::custom_page::CreateCustomPageRequest {
        name: format!("{} (Cópia)", original_page_data.name),
        path: format!("{}-copy", original_page_data.path),
        parent_path: original_page_data.parent_path,
        is_group: original_page_data.is_group != 0, // Convert i8 to bool
        description: original_page_data.description,
        icon: original_page_data.icon,
        notify_on_new_record: original_page_data.notify_on_new_record != 0, // Convert i8 to bool
        requires_acknowledgment: original_page_data.requires_acknowledgment != 0, // Convert i8 to bool
        fields: Vec::new(),
        permissions: Vec::new(),
    };
    
    // If not a group, get fields and permissions
    if original_page_data.is_group == 0 { // Check if is_group is 0 (false)
        // Get fields
        let fields = match sqlx::query!(
            r#"
            SELECT 
                id, name, display_name, field_type_id, required,
                options, validation_name, is_searchable, is_displayed_in_table,
                order_index, notification_enabled, notification_days_before,
                notification_target_date_part
            FROM page_fields 
            WHERE page_id = ?
            "#,
            page_id
        )
        .fetch_all(&mut *tx)
        .await
        {
            Ok(fields) => fields,
            Err(e) => {
                log::error!("Error fetching fields for page {}: {}", page_id, e);
                return HttpResponse::InternalServerError().finish();
            }
        };
        
        // Convert fields to CreatePageFieldRequest
        for field in fields {
            // Parse the options from binary to JSON if present
            let options = if let Some(opt_bytes) = field.options {
                match serde_json::from_slice::<serde_json::Value>(&opt_bytes) {
                    Ok(json) => Some(json),
                    Err(e) => {
                        log::error!("Error parsing options JSON for field {}: {}", field.id, e);
                        None
                    }
                }
            } else {
                None
            };
            
            request.fields.push(crate::models::custom_page::CreatePageFieldRequest {
                name: field.name,
                display_name: field.display_name,
                field_type_id: field.field_type_id,
                required: field.required != 0, // Convert i8 to bool
                options, // Use the parsed JSON value
                validation_name: field.validation_name,
                is_searchable: field.is_searchable != 0, // Convert i8 to bool
                is_displayed_in_table: field.is_displayed_in_table != 0, // Convert i8 to bool
                order_index: field.order_index,
                notification_enabled: Some(field.notification_enabled != 0), // Convert i8 to bool inside Some
                notification_days_before: field.notification_days_before,
                notification_target_date_part: field.notification_target_date_part,
            });
        }
        
        // Get permissions
        let permissions = match sqlx::query!(
            r#"
            SELECT 
                role_id, can_view, can_create, can_edit, can_delete,
                can_manage_fields, can_view_acknowledgments, can_add
            FROM page_permissions
            WHERE page_id = ?
            "#,
            page_id
        )
        .fetch_all(&mut *tx)
        .await
        {
            Ok(perms) => perms,
            Err(e) => {
                log::error!("Error fetching permissions for page {}: {}", page_id, e);
                return HttpResponse::InternalServerError().finish();
            }
        };
        
        // Convert permissions to RolePermissionRequest
        for perm in permissions {
            request.permissions.push(crate::models::custom_page::RolePermissionRequest {
                role_id: perm.role_id,
                can_view: perm.can_view != 0, // Convert i8 to bool
                can_create: perm.can_create != 0, // Convert i8 to bool
                can_edit: perm.can_edit != 0, // Convert i8 to bool
                can_delete: perm.can_delete != 0, // Convert i8 to bool
                can_manage_fields: perm.can_manage_fields != 0, // Convert i8 to bool
                can_view_acknowledgments: perm.can_view_acknowledgments != 0, // Convert i8 to bool
                can_add: perm.can_add != 0, // Convert i8 to bool
            });
        }
    }
    
    // Create the new page using the model function
    let new_page_id = match crate::models::custom_page::CustomPage::create(&state.db.pool, &request).await {
        Ok(id) => id,
        Err(e) => {
            log::error!("Error creating duplicate page: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Commit transaction
    if let Err(e) = tx.commit().await {
        log::error!("Error committing transaction: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "page_id": new_page_id,
        "message": "Página duplicada com sucesso"
    }))
}

// Add new handler for reordering pages
pub async fn reorder_pages(
    state: web::Data<State>,
    session: Session,
    data: web::Bytes,
) -> impl Responder {
    // Check if user is admin
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    // Parse the request body
    #[derive(serde::Deserialize)]
    struct PageOrder {
        id: u32,
        display_order: u32,
    }

    #[derive(serde::Deserialize)]
    struct ReorderPagesRequest {
        orders: Vec<PageOrder>,
    }

    let Json(req): Json<ReorderPagesRequest> = match Json::from_bytes(&data) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };
    
    // Convert the orders to the format expected by update_multiple_display_orders
    let orders: Vec<(u32, u32)> = req.orders
        .iter()
        .map(|order| (order.id, order.display_order))
        .collect();
        
    // Update the display orders
    match CustomPage::update_multiple_display_orders(&state.db.pool, &orders).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => {
            log::error!("Error updating page orders: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to update page orders"}))
        }
    }
}
