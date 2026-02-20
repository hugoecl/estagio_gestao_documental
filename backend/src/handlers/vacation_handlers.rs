use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use chrono::Datelike; // For year extraction
use serde::Serialize;
use sqlx::Row; // Added for Row::get method

use crate::{
    State,
    auth::validate_session,
    models::{
        role::Role, // Added for shared calendar logic
        user::User, // Assuming User model exists to fetch vacation_days_current_year
        vacation_request::{CreateVacationRequest, VacationRequest, VacationRequestStatus},
        notification::Notification,
    },
    utils::json_utils::{json_response, json_response_with_etag},
};
use actix_web::HttpRequest;
use serde::Deserialize;
use std::collections::HashMap;

// Use notification constants from the Notification module
use crate::models::notification::{
    NOTIFICATION_TYPE_VACATION_CANCELED,
    NOTIFICATION_TYPE_VACATION_REQUESTED,
};

// Struct for parsing query parameters for shared calendar
#[derive(Deserialize, Debug)]
pub struct SharedCalendarQuery {
    year: i32,
}

// Handler for a user to submit a new vacation request
pub async fn submit_vacation_request(
    state: web::Data<State>,
    session: Session,
    data: web::Json<CreateVacationRequest>,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    let request_data = data.into_inner();

    // --- Basic Validations ---
    if request_data.start_date > request_data.end_date {
        return HttpResponse::BadRequest()
            .body("A data de início não pode ser posterior à data de fim.");
    }
    // Optional: Check if dates are in the past (allow for some flexibility or be strict)
    let today = chrono::Utc::now().date_naive();
    if request_data.start_date < today {
        // return HttpResponse::BadRequest().body("Não pode solicitar férias para datas passadas.");
    }

    let requested_days_count =
        crate::utils::working_days::count_working_days(
            request_data.start_date,
            request_data.end_date,
            request_data.start_date.year(),
        );
    if requested_days_count <= 0 {
        return HttpResponse::BadRequest().body("Número de dias de férias inválido.");
    }

    // --- Check Remaining Vacation Days ---
    let user_details = match sqlx::query_as!(
        User, // Assuming User model has vacation_days_current_year or a similar field
        "SELECT id, username, email, vacation_days_current_year FROM users WHERE id = ?", // Adjust query as needed
        user_id
    )
    .fetch_one(&state.db.pool)
    .await
    {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().body("Utilizador não encontrado.");
        }
        Err(e) => {
            log::error!("Error fetching user details for vacation check: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let available_days = user_details.vacation_days_current_year.unwrap_or(0); // Default to 0 if null

    // Calculate already approved days for the current year
    let current_year = today.year();
    let approved_days_count = match VacationRequest::count_approved_vacation_days_for_year(
        &state.db.pool,
        user_id,
        current_year,
    )
    .await
    {
        Ok(count) => count,
        Err(e) => {
            log::error!(
                "Error counting approved vacation days for user {}: {}",
                user_id,
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    if approved_days_count + requested_days_count > available_days as i64 {
        return HttpResponse::BadRequest().body(format!(
            "Não tem dias de férias suficientes. Disponíveis: {}, Solicitados: {}, Já aprovados: {}.",
            available_days, requested_days_count, approved_days_count
        ));
    }

    // --- Initial Conflict Check (Same User) ---
    // Check for overlapping requests for the *same user*.
    let existing_requests = match VacationRequest::get_by_user_id(&state.db.pool, user_id).await {
        Ok(reqs) => reqs,
        Err(e) => {
            log::error!(
                "Error fetching existing requests for user {}: {}",
                user_id,
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    for req in existing_requests {
        // Skip rejected requests for conflict checking
        if req.status == VacationRequestStatus::Rejected {
            continue;
        }
        // Check for overlap: (StartA <= EndB) and (EndA >= StartB)
        if request_data.start_date <= req.end_date && request_data.end_date >= req.start_date {
            return HttpResponse::Conflict().body(format!(
                "Já tem um pedido de férias ({:?}) que entra em conflito com as datas solicitadas.",
                req.status
            ));
        }
    }
    
    // --- Advanced Conflict Check (Colleagues in Shared Holiday Role) ---
    // Get all users who share holiday roles with the current user
    let colleague_user_ids = match Role::get_colleague_user_ids_in_shared_holiday_roles(&state.db.pool, user_id).await {
        Ok(ids) => ids,
        Err(e) => {
            log::error!(
                "Error fetching colleague user IDs for vacation conflict check: {}",
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    };
    
    if !colleague_user_ids.is_empty() {
        // Get both pending and approved vacation requests from colleagues
        let requests_query = format!(
            r#"
            SELECT vr.start_date, vr.end_date, vr.status, u.username
            FROM vacation_requests vr
            JOIN users u ON vr.user_id = u.id
            WHERE vr.user_id IN ({})
              AND vr.status IN ('PENDING', 'APPROVED')
              AND (
                  (vr.start_date <= ? AND vr.end_date >= ?) OR 
                  (vr.start_date <= ? AND vr.end_date >= ?) OR 
                  (vr.start_date >= ? AND vr.end_date <= ?)    
              )
            "#,
            colleague_user_ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(",")
        );
        
        let mut query_builder = sqlx::query(&requests_query);
        for colleague_id in &colleague_user_ids {
            query_builder = query_builder.bind(colleague_id);
        }
        
        query_builder = query_builder
            .bind(&request_data.end_date)
            .bind(&request_data.start_date)
            .bind(&request_data.end_date)
            .bind(&request_data.start_date)
            .bind(&request_data.start_date)
            .bind(&request_data.end_date);
            
        let colleague_requests = match query_builder.fetch_all(&state.db.pool).await {
            Ok(rows) => rows,
            Err(e) => {
                log::error!(
                    "Error fetching colleague vacation requests for conflict check: {}",
                    e
                );
                return HttpResponse::InternalServerError().finish();
            }
        };
        
        if !colleague_requests.is_empty() {
            let first_conflict = colleague_requests.first().unwrap();
            let status: String = first_conflict.try_get("status").unwrap_or_default();
            let username: String = first_conflict.try_get("username").unwrap_or_default();
            
            let status_desc = match status.as_str() {
                "PENDING" => "pendente",
                "APPROVED" => "aprovado",
                _ => "existente",
            };
            
            return HttpResponse::Conflict().body(format!(
                "As datas solicitadas entram em conflito com um pedido {} do colega {}.",
                status_desc, username
            ));
        }
    }
    // --- End Conflict Check ---

    match VacationRequest::create(&state.db.pool, user_id, &request_data).await {
        Ok(request_id) => {
            // Get user name for the notification
            let user_name = match sqlx::query!("SELECT username FROM users WHERE id = ?", user_id)
                .fetch_optional(&state.db.pool)
                .await {
                    Ok(Some(user_row)) => user_row.username,
                    _ => "Um utilizador".to_string(), // Fallback if we can't get the username
                };

            // Format dates for the message
            let start_date_fmt = request_data.start_date.format("%d/%m/%Y").to_string();
            let end_date_fmt = request_data.end_date.format("%d/%m/%Y").to_string();
            
            // Step 1: Send notification to admins about the new request
            match Role::get_user_ids_by_role_id(&state.db.pool, 1).await { // Admin role ID is usually 1
                Ok(admin_ids) => {
                    if !admin_ids.is_empty() {
                        let admin_message = format!(
                            "{} solicitou férias ({} a {}).",
                            user_name, start_date_fmt, end_date_fmt
                        );

                        // Send notification to each admin
                        for admin_id in admin_ids {
                            match Notification::create(
                                &state.db.pool,
                                admin_id,
                                None,              // record_id - Not used for vacation requests
                                Some(request_id),  // vacation_request_id
                                None,              // page_id - Not applicable
                                None,              // field_id - Not applicable
                                NOTIFICATION_TYPE_VACATION_REQUESTED,
                                &admin_message,
                                Some(request_data.end_date), // End date as due date
                            ).await {
                                Ok(_) => {
                                    log::info!(
                                        "Created vacation request notification for admin {}, vacation request {}",
                                        admin_id,
                                        request_id
                                    );
                                },
                                Err(e) => {
                                    log::error!(
                                        "Failed to create vacation request notification for admin {}, vacation request {}: {}",
                                        admin_id,
                                        request_id,
                                        e
                                    );
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    log::error!("Error fetching admin users for notifications: {}", e);
                    // Continue even if we can't send notifications
                }
            }
            
            // Step 2: Send notifications to users in the same vacation role(s)
            match Role::get_colleague_user_ids_in_shared_holiday_roles(&state.db.pool, user_id).await {
                Ok(colleague_ids) => {
                    if !colleague_ids.is_empty() {
                        let colleague_message = format!(
                            "O seu colega {} solicitou férias ({} a {}).",
                            user_name, start_date_fmt, end_date_fmt
                        );
                        
                        // Send notification to each colleague (except the requesting user)
                        for colleague_id in colleague_ids {
                            // Skip the requesting user - they don't need a notification about their own request
                            if colleague_id == user_id {
                                continue;
                            }
                            
                            match Notification::create(
                                &state.db.pool,
                                colleague_id,
                                None,              // record_id - Not used for vacation requests
                                Some(request_id),  // vacation_request_id
                                None,              // page_id - Not applicable
                                None,              // field_id - Not applicable
                                NOTIFICATION_TYPE_VACATION_REQUESTED,
                                &colleague_message,
                                Some(request_data.end_date), // End date as due date
                            ).await {
                                Ok(_) => {
                                    log::info!(
                                        "Created vacation request notification for colleague {}, vacation request {}",
                                        colleague_id,
                                        request_id
                                    );
                                },
                                Err(e) => {
                                    log::error!(
                                        "Failed to create vacation request notification for colleague {}, vacation request {}: {}",
                                        colleague_id,
                                        request_id,
                                        e
                                    );
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    log::error!("Error fetching colleague users for notifications: {}", e);
                    // Continue even if we can't send colleague notifications
                }
            }

            HttpResponse::Created().json(serde_json::json!({ "id": request_id }))
        },
        Err(e) => {
            log::error!("Error creating vacation request: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Handler for a user to fetch their own vacation requests
pub async fn get_my_vacation_requests(
    state: web::Data<State>, 
    session: Session,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    // Get year from query parameters or use current year as default
    let year_filter = match query.get("year") {
        Some(year_str) => match year_str.parse::<i32>() {
            Ok(year) => Some(year),
            Err(_) => {
                log::error!("Invalid year parameter: {}", year_str);
                return HttpResponse::BadRequest().body("Invalid year parameter");
            }
        },
        None => None,
    };

    match VacationRequest::get_by_user_id(&state.db.pool, user_id).await {
        Ok(mut requests) => {
            // Filter requests by year if year parameter was provided
            if let Some(year) = year_filter {
                requests = requests
                    .into_iter()
                    .filter(|req| {
                        // Include request if it overlaps with the specified year
                        req.start_date.year() == year || req.end_date.year() == year
                    })
                    .collect();
            }
            json_response(&requests)
        },
        Err(e) => {
            log::error!(
                "Error fetching vacation requests for user {}: {}",
                user_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Serialize)]
struct RemainingVacationDaysResponse {
    total_allocated_days: u16,
    approved_days_taken: i64,
    pending_days_requested: i64, // Also good to show pending
    remaining_days: i64,
}

// Handler for a user to fetch their remaining vacation days
pub async fn get_my_remaining_vacation_days(
    state: web::Data<State>,
    session: Session,
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    // Get year from query parameters or use current year as default
    let year = match query.get("year") {
        Some(year_str) => match year_str.parse::<i32>() {
            Ok(year) => year,
            Err(_) => {
                log::error!("Invalid year parameter: {}", year_str);
                return HttpResponse::BadRequest().body("Invalid year parameter");
            }
        },
        None => chrono::Utc::now().year(),
    };

    let user_details = match sqlx::query!(
        "SELECT vacation_days_current_year FROM users WHERE id = ?",
        user_id
    )
    .fetch_one(&state.db.pool)
    .await
    {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::NotFound().body("Utilizador não encontrado.");
        }
        Err(e) => {
            log::error!("Error fetching user's allocated vacation days: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let total_allocated_days = user_details.vacation_days_current_year.unwrap_or(0);

    let approved_days_taken = match VacationRequest::count_approved_vacation_days_for_year(
        &state.db.pool,
        user_id,
        year, // Use requested year instead of current year
    )
    .await
    {
        Ok(count) => count,
        Err(e) => {
            log::error!(
                "Error counting approved vacation days for user {}: {}",
                user_id,
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Calculate pending days (optional, but good for user info)
    let all_user_requests = match VacationRequest::get_by_user_id(&state.db.pool, user_id).await {
        Ok(requests) => requests,
        Err(e) => {
            log::error!(
                "Error fetching all requests for pending calculation for user {}: {}",
                user_id,
                e
            );
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut pending_days_requested: i64 = 0;
    for req in all_user_requests {
        if req.status == VacationRequestStatus::Pending {
            if req.start_date.year() == year || req.end_date.year() == year {
                let start_date = std::cmp::max(
                    req.start_date,
                    chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
                );
                let end_date = std::cmp::min(
                    req.end_date,
                    chrono::NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
                );
                if start_date <= end_date {
                    pending_days_requested +=
                        crate::utils::working_days::count_working_days(start_date, end_date, year);
                }
            }
        }
    }

    let remaining_days = total_allocated_days as i64 - approved_days_taken - pending_days_requested;

    json_response(&RemainingVacationDaysResponse {
        total_allocated_days: total_allocated_days as u16,
        approved_days_taken,
        pending_days_requested,
        remaining_days,
    })
}

// Add handlers to the end of src/handlers/vacation_handlers.rs

// Handler for canceling a pending vacation request
pub async fn cancel_vacation_request(
    state: web::Data<State>,
    session: Session,
    request_id_path: web::Path<u32>
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    let request_id = request_id_path.into_inner();

    // First, fetch the request to verify it belongs to the user and is in PENDING state
    let request = match sqlx::query!(
        "SELECT user_id, status, start_date, end_date FROM vacation_requests WHERE id = ?",
        request_id
    )
    .fetch_optional(&state.db.pool)
    .await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return HttpResponse::NotFound().body(format!("Pedido de férias #{} não encontrado.", request_id));
        },
        Err(e) => {
            log::error!("Error fetching vacation request {}: {}", request_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Verify the request belongs to the current user
    if request.user_id != user_id {
        return HttpResponse::Forbidden().body("Não tem permissão para cancelar este pedido de férias.");
    }

    // Verify the request is in PENDING state
    if request.status != "PENDING" {
        return HttpResponse::BadRequest().body(
            "Apenas pedidos pendentes podem ser cancelados. Este pedido já foi processado."
        );
    }

    // Delete the request
    match sqlx::query!(
        "DELETE FROM vacation_requests WHERE id = ? AND user_id = ? AND status = 'PENDING'",
        request_id, user_id
    )
    .execute(&state.db.pool)
    .await {
        Ok(result) => {
            if result.rows_affected() > 0 {
                // Send notification to admins
                // First, get all users with admin role
                match Role::get_user_ids_by_role_id(&state.db.pool, 1).await { // Admin role ID is usually 1
                    Ok(admin_ids) => {
                        if !admin_ids.is_empty() {
                            // Get user name for the notification
                            let user_name = match sqlx::query!("SELECT username FROM users WHERE id = ?", user_id)
                                .fetch_optional(&state.db.pool)
                                .await {
                                    Ok(Some(user_row)) => user_row.username,
                                    _ => "Um utilizador".to_string(), // Fallback if we can't get the username
                                };

                            // Format dates for the message
                            let start_date_fmt = request.start_date.format("%d/%m/%Y").to_string();
                            let end_date_fmt = request.end_date.format("%d/%m/%Y").to_string();
                            
                            let admin_message = format!(
                                "{} cancelou um pedido de férias ({} a {}).",
                                user_name, start_date_fmt, end_date_fmt
                            );

                            // Send notification to each admin
                            for admin_id in admin_ids {
                                match Notification::create(
                                    &state.db.pool,
                                    admin_id,
                                    None,              // record_id - Not used for vacation requests
                                    Some(request_id),  // vacation_request_id
                                    None,              // page_id - Not applicable
                                    None,              // field_id - Not applicable
                                    NOTIFICATION_TYPE_VACATION_CANCELED,
                                    &admin_message,
                                    None,             // No due date for cancellations
                                ).await {
                                    Ok(_) => {
                                        log::info!(
                                            "Created vacation cancellation notification for admin {}, vacation request {}",
                                            admin_id,
                                            request_id
                                        );
                                    },
                                    Err(e) => {
                                        log::error!(
                                            "Failed to create vacation cancellation notification for admin {}, vacation request {}: {}",
                                            admin_id,
                                            request_id,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Error fetching admin users for notifications: {}", e);
                        // Continue even if we can't send notifications
                    }
                }
                
                // Also notify colleagues in the same vacation role
                match Role::get_colleague_user_ids_in_shared_holiday_roles(&state.db.pool, user_id).await {
                    Ok(colleague_ids) => {
                        if !colleague_ids.is_empty() {
                            // Get user name for colleague notifications (if not already fetched)
                            let user_name = match sqlx::query!("SELECT username FROM users WHERE id = ?", user_id)
                                .fetch_optional(&state.db.pool)
                                .await {
                                    Ok(Some(user_row)) => user_row.username,
                                    _ => "Um utilizador".to_string(), // Fallback if we can't get the username
                                };

                            // Format dates for the message (if not already formatted)
                            let start_date_fmt = request.start_date.format("%d/%m/%Y").to_string();
                            let end_date_fmt = request.end_date.format("%d/%m/%Y").to_string();
                            
                            let colleague_message = format!(
                                "O seu colega {} cancelou um pedido de férias ({} a {}).",
                                user_name, start_date_fmt, end_date_fmt
                            );
                            
                            // Send notification to each colleague (except the canceling user)
                            for colleague_id in colleague_ids {
                                // Skip the user who canceled - they don't need a notification about their own cancellation
                                if colleague_id == user_id {
                                    continue;
                                }
                                
                                match Notification::create(
                                    &state.db.pool,
                                    colleague_id,
                                    None,              // record_id - Not used for vacation requests
                                    Some(request_id),  // vacation_request_id
                                    None,              // page_id - Not applicable
                                    None,              // field_id - Not applicable
                                    NOTIFICATION_TYPE_VACATION_CANCELED,
                                    &colleague_message,
                                    None,             // No due date for cancellations
                                ).await {
                                    Ok(_) => {
                                        log::info!(
                                            "Created vacation cancellation notification for colleague {}, vacation request {}",
                                            colleague_id,
                                            request_id
                                        );
                                    },
                                    Err(e) => {
                                        log::error!(
                                            "Failed to create vacation cancellation notification for colleague {}, vacation request {}: {}",
                                            colleague_id,
                                            request_id,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Error fetching colleague users for cancellation notifications: {}", e);
                        // Continue even if we can't send notifications to colleagues
                    }
                }

                HttpResponse::Ok().body(format!("Pedido de férias #{} cancelado com sucesso.", request_id))
            } else {
                // This should not happen since we already checked the request exists and belongs to the user
                HttpResponse::InternalServerError().body("Falha ao cancelar o pedido de férias.")
            }
        },
        Err(e) => {
            log::error!("Error deleting vacation request {}: {}", request_id, e);
                HttpResponse::InternalServerError().finish()
        }
    }
}

/// Handler for requesting cancellation of an approved vacation.
/// Changes status from APPROVED to CANCELLATION_REQUESTED. Admin must approve.
pub async fn request_vacation_cancellation(
    state: web::Data<State>,
    session: Session,
    request_id_path: web::Path<u32>,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    let request_id = request_id_path.into_inner();

    let request = match sqlx::query!(
        "SELECT user_id, status, start_date, end_date FROM vacation_requests WHERE id = ?",
        request_id
    )
    .fetch_optional(&state.db.pool)
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return HttpResponse::NotFound()
                .body(format!("Pedido de férias #{} não encontrado.", request_id));
        }
        Err(e) => {
            log::error!("Error fetching vacation request {}: {}", request_id, e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if request.user_id != user_id {
        return HttpResponse::Forbidden()
            .body("Não tem permissão para pedir cancelamento deste pedido.");
    }

    if request.status != "APPROVED" {
        return HttpResponse::BadRequest().body(
            "Apenas férias aprovadas podem ter pedido de cancelamento. Este pedido não está aprovado.",
        );
    }

    match sqlx::query!(
        "UPDATE vacation_requests SET status = 'CANCELLATION_REQUESTED' WHERE id = ? AND user_id = ? AND status = 'APPROVED'",
        request_id,
        user_id
    )
    .execute(&state.db.pool)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                let user_name = match sqlx::query!("SELECT username FROM users WHERE id = ?", user_id)
                    .fetch_optional(&state.db.pool)
                    .await
                {
                    Ok(Some(r)) => r.username,
                    _ => "Um utilizador".to_string(),
                };
                let start_fmt = request.start_date.format("%d/%m/%Y").to_string();
                let end_fmt = request.end_date.format("%d/%m/%Y").to_string();
                let msg = format!(
                    "{} pediu o cancelamento das férias aprovadas ({} a {}).",
                    user_name, start_fmt, end_fmt
                );
                if let Ok(admin_ids) = Role::get_user_ids_by_role_id(&state.db.pool, 1).await {
                    for admin_id in admin_ids {
                        let _ = Notification::create(
                            &state.db.pool,
                            admin_id,
                            None,
                            Some(request_id),
                            None,
                            None,
                            NOTIFICATION_TYPE_VACATION_REQUESTED,
                            &msg,
                            None,
                        )
                        .await;
                    }
                }
                HttpResponse::Ok()
                    .body(format!("Pedido de cancelamento enviado. O administrador irá processar o pedido #{}", request_id))
            } else {
                HttpResponse::InternalServerError().body("Falha ao registar pedido de cancelamento.")
            }
        }
        Err(e) => {
            log::error!("Error updating vacation request {}: {}", request_id, e);
            let err_str = e.to_string();
            let msg = if err_str.contains("Data truncated") || err_str.contains("enum") || err_str.contains("CANCELLATION") {
                "A base de dados precisa de ser atualizada. Execute a migration 008_add_vacation_cancellation_status.sql no servidor MySQL."
            } else {
                "Falha ao registar pedido de cancelamento."
            };
            HttpResponse::InternalServerError().body(msg)
        }
    }
}

// Response struct for shared calendar with status information
#[derive(Serialize)]
struct SharedCalendarResponse {
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    status: String,
}

pub async fn get_shared_calendar_vacations(
    state: web::Data<State>,
    session: Session,
    query: web::Query<SharedCalendarQuery>,
    req: HttpRequest, // Added for ETag
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    let year = query.year;

    match Role::get_colleague_user_ids_in_shared_holiday_roles(&state.db.pool, user_id).await {
        Ok(colleague_user_ids) => {
            if colleague_user_ids.is_empty() {
                // No colleagues in shared holiday roles, or user is not in any holiday role
                return json_response_with_etag(
                    &Vec::<SharedCalendarResponse>::new(),
                    &req,
                );
            }

            match VacationRequest::get_approved_and_pending_dates_for_users_in_year(
                &state.db.pool,
                &colleague_user_ids,
                year,
            )
            .await
            {
                Ok(dates) => {
                    // Transform the dates into the response format
                    let response = dates.into_iter()
                        .map(|(start_date, end_date, status)| 
                            SharedCalendarResponse {
                                start_date,
                                end_date,
                                status,
                            })
                        .collect::<Vec<_>>();
                    
                    json_response_with_etag(&response, &req)
                },
                Err(e) => {
                    log::error!("Error fetching vacation dates for colleagues: {}", e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        Err(e) => {
            log::error!("Error fetching colleague user IDs: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
