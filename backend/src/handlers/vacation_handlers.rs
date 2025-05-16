use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use chrono::Datelike; // For year extraction
use serde::Serialize;
use sqlx::Row; // Added for Row::get method

use crate::{
    State,
    auth::validate_session, // Assuming user_can_view_page might be relevant later
    models::{
        role::Role, // Added for shared calendar logic
        user::User, // Assuming User model exists to fetch vacation_days_current_year
        vacation_request::{CreateVacationRequest, VacationRequest, VacationRequestStatus},
    },
    utils::json_utils::{json_response, json_response_with_etag},
};
use actix_web::HttpRequest; // Added for HttpRequest
use serde::Deserialize;

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

    let requested_days_count = (request_data.end_date - request_data.start_date).num_days() + 1;
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

    if (approved_days_count + requested_days_count) as u32 > available_days as u32 {
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
        Ok(request_id) => HttpResponse::Created().json(serde_json::json!({ "id": request_id })),
        Err(e) => {
            log::error!("Error creating vacation request: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// Handler for a user to fetch their own vacation requests
pub async fn get_my_vacation_requests(state: web::Data<State>, session: Session) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    match VacationRequest::get_by_user_id(&state.db.pool, user_id).await {
        Ok(requests) => json_response(&requests),
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
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
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
    let current_year = chrono::Utc::now().year();

    let approved_days_taken = match VacationRequest::count_approved_vacation_days_for_year(
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

    let mut pending_days_requested = 0;
    for req in all_user_requests {
        if req.status == VacationRequestStatus::Pending {
            // Ensure the request is for the current year before counting
            if req.start_date.year() == current_year || req.end_date.year() == current_year {
                let start_date = std::cmp::max(
                    req.start_date,
                    chrono::NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap(),
                );
                let end_date = std::cmp::min(
                    req.end_date,
                    chrono::NaiveDate::from_ymd_opt(current_year, 12, 31).unwrap(),
                );
                if start_date <= end_date {
                    pending_days_requested += (end_date - start_date).num_days() + 1;
                }
            }
        }
    }

    let remaining_days = total_allocated_days as i64 - approved_days_taken;

    json_response(&RemainingVacationDaysResponse {
        total_allocated_days: total_allocated_days as u16,
        approved_days_taken,
        pending_days_requested,
        remaining_days,
    })
}

// Add this handler to the end of src/handlers/vacation_handlers.rs

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
