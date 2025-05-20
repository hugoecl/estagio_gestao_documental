use actix_session::Session;
use actix_web::{HttpRequest, HttpResponse, Responder, web, FromRequest};
use actix_multipart::Multipart;
use futures_util::{StreamExt, TryStreamExt};
use std::fs;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

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
            icon, icon_type, icon_image_path, notify_on_new_record as "notify_on_new_record: bool", 
            requires_acknowledgment as "requires_acknowledgment: bool", display_order, 
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
    mut payload: Multipart,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    // Check if this is a multipart form (with file) or a JSON request
    let mut fields = std::collections::HashMap::new();
    let mut icon_image: Option<(Vec<u8>, String)> = None;
    
    // Process multipart form
    while let Ok(Some(mut field)) = payload.try_next().await {
        // Clone the content disposition to avoid borrow issues
        let cd = field.content_disposition().unwrap().clone();
        
        let name = cd.get_name();
        if let Some(name) = name {
            // Handle file upload
            if name == "icon_image" {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk.unwrap());
                }
                
                // Get filename from the cloned content disposition
                let filename = cd.get_filename();
                if let Some(filename) = filename {
                    icon_image = Some((data, filename.to_string()));
                } else {
                    log::warn!("Icon image field found but no filename provided");
                }
            } else {
                // Handle form fields
                let mut field_data = Vec::new();
                while let Some(chunk) = field.next().await {
                    field_data.extend_from_slice(&chunk.unwrap());
                }
                if let Ok(field_value) = String::from_utf8(field_data) {
                    log::debug!("Form field: {} = {}", name, field_value);
                    fields.insert(name.to_string(), field_value);
                }
            }
        }
    }
    
    // If we have form fields, build a request from them
    if !fields.is_empty() {
        let mut custom_page_req = CreateCustomPageRequest {
            name: fields.get("name").unwrap_or(&String::new()).clone(),
            path: fields.get("path").unwrap_or(&String::new()).clone(),
            parent_path: fields.get("parent_path").map(|v| if v.is_empty() { None } else { Some(v.clone()) }).unwrap_or(None),
            is_group: fields.get("is_group").map(|v| v == "true").unwrap_or(false),
            description: fields.get("description").map(|v| if v.is_empty() { None } else { Some(v.clone()) }).unwrap_or(None),
            icon: fields.get("icon").map(|v| if v.is_empty() { None } else { Some(v.clone()) }).unwrap_or(None),
            icon_type: fields.get("icon_type").map(|v| if v.is_empty() { None } else { Some(v.clone()) }).unwrap_or(None),
            notify_on_new_record: fields.get("notify_on_new_record").map(|v| v == "true").unwrap_or(false),
            requires_acknowledgment: fields.get("requires_acknowledgment").map(|v| v == "true").unwrap_or(false),
            fields: Vec::new(),
            permissions: Vec::new(),
        };
        
        // Parse JSON fields
        if let Some(permissions_json) = fields.get("permissions") {
            if let Ok(permissions) = serde_json::from_str::<Vec<RolePermissionRequest>>(permissions_json) {
                custom_page_req.permissions = permissions;
            }
        }
        
        if let Some(fields_json) = fields.get("fields") {
            if let Ok(fields) = serde_json::from_str(fields_json) {
                custom_page_req.fields = fields;
            }
        }
        
        // Create the page first to get its ID
        let page_id = match CustomPage::create(&state.db.pool, &custom_page_req).await {
            Ok(id) => id,
            Err(e) => {
                log::error!("Error creating custom page: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };
        
        log::info!("Successfully created page with ID: {}", page_id);
        
        // Handle icon image if present
        if let Some((image_data, filename)) = icon_image {
            
            // Force icon_type to 'image' in the database
            let set_icon_type_result = sqlx::query!(
                "UPDATE custom_pages SET icon_type = 'image' WHERE id = ?",
                page_id
            )
            .execute(&state.db.pool)
            .await;
            
            if let Err(e) = set_icon_type_result {
                log::error!("Error setting icon_type to 'image' for page {}: {}", page_id, e);
            } 
            
            let icon_path = handle_icon_upload(page_id, image_data, filename).await;
            
            if let Some(icon_path) = icon_path {
                // Update the page with the icon path and ensure icon_type is set to "image"
                let update_result = sqlx::query!(
                    "UPDATE custom_pages SET icon_image_path = ?, icon_type = 'image' WHERE id = ?",
                    icon_path, page_id
                )
                .execute(&state.db.pool)
                .await;
                
                if let Err(e) = update_result {
                    log::error!("Error updating icon path: {}", e);
                    // We've created the page, so return success anyway
                }
            } else {
                log::error!("Failed to get icon path for page {}", page_id);
            }
        } else {
            log::info!("No icon image provided for page {}", page_id);
        }
        
        return HttpResponse::Created().body(page_id.to_string());
    } else {
        // If no form fields, suggest using the JSON endpoint
        log::warn!("No form fields detected - client should use the JSON endpoint instead");
        return HttpResponse::BadRequest().body("No form fields detected. For JSON requests, use the /custom_pages/json endpoint");
    }
}

// Add a new handler for JSON requests
pub async fn create_custom_page_json(
    state: web::Data<State>,
    req: HttpRequest,
    session: Session,
) -> impl Responder {
    if let Err(resp) = is_admin(&session) {
        return resp;
    }

    // Parse request body
    let body = match web::Json::<CreateCustomPageRequest>::extract(&req).await {
        Ok(json_data) => json_data.into_inner(),
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    if !body.is_group && (body.path.is_empty() || body.path == "/")
    {
        log::error!(
            "Attempted to create a page with an invalid path: {}",
            body.path
        );
        return HttpResponse::BadRequest().finish();
    }

    match CustomPage::create(&state.db.pool, &body).await {
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
    mut payload: Multipart,
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

    // Check if this is a multipart form (with file) or a JSON request
    let mut fields = std::collections::HashMap::new();
    let mut icon_image: Option<(Vec<u8>, String)> = None;
    
    // Process multipart form
    while let Ok(Some(mut field)) = payload.try_next().await {
        // Clone the content disposition to avoid borrow issues
        let cd = field.content_disposition().unwrap().clone();
        
        let name = cd.get_name();
        if let Some(name) = name {
            // Handle file upload
            if name == "icon_image" {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    data.extend_from_slice(&chunk.unwrap());
                }
                
                // Get filename from the cloned content disposition
                let filename = cd.get_filename();
                if let Some(filename) = filename {
                    icon_image = Some((data, filename.to_string()));
                }
            } else {
                // Handle form fields
                let mut field_data = Vec::new();
                while let Some(chunk) = field.next().await {
                    field_data.extend_from_slice(&chunk.unwrap());
                }
                if let Ok(field_value) = String::from_utf8(field_data) {
                    fields.insert(name.to_string(), field_value);
                }
            }
        }
    }
    
    // If we have form fields, build a request from them
    if !fields.is_empty() {
        let update_req = UpdateCustomPageRequest {
            name: fields.get("name").unwrap_or(&String::new()).clone(),
            parent_path: fields.get("parent_path").map(|v| if v.is_empty() { None } else { Some(v.clone()) }).unwrap_or(None),
            description: fields.get("description").map(|v| if v.is_empty() { None } else { Some(v.clone()) }).unwrap_or(None),
            icon: fields.get("icon").map(|v| if v.is_empty() { None } else { Some(v.clone()) }).unwrap_or(None),
            icon_type: fields.get("icon_type").map(|v| if v.is_empty() { None } else { Some(v.clone()) }).unwrap_or(None),
            notify_on_new_record: fields.get("notify_on_new_record").map(|v| Some(v == "true")).unwrap_or(None),
            requires_acknowledgment: fields.get("requires_acknowledgment").map(|v| Some(v == "true")).unwrap_or(None),
        };
        
        // Update the page
        match CustomPage::update(&state.db.pool, page_id, &update_req).await {
            Ok(_) => (),
            Err(e) => {
                log::error!("Error updating custom page: {}", e);
                return HttpResponse::InternalServerError().finish();
            }
        }
        
        // Check if we should clear the icon image
        if fields.get("clear_icon_image").map(|v| v == "true").unwrap_or(false) {
            let clear_image_result = sqlx::query!(
                "UPDATE custom_pages SET icon_image_path = NULL, icon_type = 'fontawesome' WHERE id = ?", 
                page_id
            )
            .execute(&state.db.pool)
            .await;
            
            if let Err(e) = clear_image_result {
                log::error!("Error clearing icon image path: {}", e);
                // Continue anyway, as the main update succeeded
            }
        }
        
        // Handle icon image if present
        if let Some((image_data, filename)) = icon_image {
            let icon_path = handle_icon_upload(page_id, image_data, filename).await;
            
            if let Some(icon_path) = icon_path {
                // Update the page with the icon path and ensure icon_type is set to "image"
                let update_result = sqlx::query!(
                    "UPDATE custom_pages SET icon_image_path = ?, icon_type = 'image' WHERE id = ?",
                    icon_path, page_id
                )
                .execute(&state.db.pool)
                .await;
                
                if let Err(e) = update_result {
                    log::error!("Error updating icon path: {}", e);
                    // We've created the page, so return success anyway
                }
            }
        }
        
        return HttpResponse::Ok().finish();
    } else {
        // If no form fields, suggest using the JSON endpoint
        log::warn!("No form fields detected - client should use the JSON endpoint instead");
        return HttpResponse::BadRequest().body(format!("No form fields detected. For JSON requests, use the /custom_pages/{}/json endpoint", page_id));
    }
}

// Add a new handler for JSON requests
pub async fn update_custom_page_json(
    state: web::Data<State>,
    path: web::Path<u32>,
    req: HttpRequest,
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

    // Parse request body
    let body = match web::Json::<UpdateCustomPageRequest>::extract(&req).await {
        Ok(json_data) => json_data.into_inner(),
        Err(e) => {
            log::error!("Error parsing JSON: {}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    match CustomPage::update(&state.db.pool, page_id, &body).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Error updating custom page: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Helper function to handle icon uploads
async fn handle_icon_upload(page_id: u32, image_data: Vec<u8>, filename: String) -> Option<String> {
    // Create directory for icons if it doesn't exist
    let icons_dir = "media/page_icons";
    
    if let Err(e) = fs::create_dir_all(icons_dir) {
        log::error!("Error creating page icons directory: {}", e);
        return None;
    }
    
    // Generate a unique filename
    let extension = Path::new(&filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("png");
    
    let unique_filename = format!("{}-{}.{}", page_id, Uuid::new_v4(), extension);
    let file_path = format!("{}/{}", icons_dir, unique_filename);
    
    // Write the file
    let mut file = match fs::File::create(&file_path) {
        Ok(file) => file,
        Err(e) => {
            log::error!("Error creating icon file: {}", e);
            return None;
        }
    };
    
    if let Err(e) = file.write_all(&image_data) {
        log::error!("Error writing icon file: {}", e);
        return None;
    }
    
    Some(file_path)
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
            icon, icon_type, icon_image_path, notify_on_new_record, requires_acknowledgment
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
        parent_path: original_page_data.parent_path.clone(), // Clone to avoid move
        is_group: original_page_data.is_group != 0, // Convert i8 to bool
        description: original_page_data.description.clone(), // Clone to avoid move
        icon: original_page_data.icon.clone(), // Clone to avoid move
        icon_type: original_page_data.icon_type.clone(), // Clone to avoid move
        notify_on_new_record: original_page_data.notify_on_new_record != 0, // Convert i8 to bool
        requires_acknowledgment: original_page_data.requires_acknowledgment != 0, // Convert i8 to bool
        fields: Vec::new(),
        permissions: Vec::new(),
    };
    
    // We'll copy the icon image after the page is created, as we need its ID
    
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
    
    // Copy the icon image if it exists
    if let Some(icon_image_path) = &original_page_data.icon_image_path {
        if let Some(icon_type) = &original_page_data.icon_type {
            if icon_type == "image" {
                // Copy the image file to a new location
                if let Ok(image_data) = std::fs::read(icon_image_path) {
                    if let Some(file_path) = handle_icon_upload(new_page_id, image_data, icon_image_path.clone()).await {
                        // Update the icon_image_path in the database
                        if let Err(e) = sqlx::query!(
                            "UPDATE custom_pages SET icon_image_path = ? WHERE id = ?",
                            file_path, new_page_id
                        )
                        .execute(&state.db.pool)
                        .await 
                        {
                            log::error!("Error updating icon image path: {}", e);
                            // Continue anyway as this is not critical
                        }
                    }
                }
            }
        }
    }

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
