use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use chrono::Datelike; // For year extraction
use serde::Serialize;

use crate::{
    auth::validate_session, // Assuming user_can_view_page might be relevant later
    models::{
        user::User, // Assuming User model exists to fetch vacation_days_current_year
        vacation_request::{CreateVacationRequest, VacationRequest, VacationRequestStatus},
    },
    utils::json_utils::json_response,
    State,
};

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
        return HttpResponse::BadRequest().body("A data de início não pode ser posterior à data de fim.");
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
        Err(sqlx::Error::RowNotFound) => return HttpResponse::NotFound().body("Utilizador não encontrado."),
        Err(e) => {
            log::error!("Error fetching user details for vacation check: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let available_days = user_details.vacation_days_current_year.unwrap_or(0); // Default to 0 if null

    // Calculate already approved days for the current year
    let current_year = today.year();
    let approved_days_count =
        match VacationRequest::count_approved_vacation_days_for_year(
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

    // --- TODO: Initial Conflict Check (Phase 2 - basic, Phase 4 - advanced) ---
    // For now, let's assume a simple check for overlapping requests for the *same user*.
    // Advanced role-based conflict will be in Phase 4.
    let existing_requests = match VacationRequest::get_by_user_id(&state.db.pool, user_id).await {
        Ok(reqs) => reqs,
        Err(e) => {
            log::error!("Error fetching existing requests for user {}: {}", user_id, e);
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
pub async fn get_my_vacation_requests(
    state: web::Data<State>,
    session: Session,
) -> impl Responder {
    let user_id = match validate_session(&session) {
        Ok(id) => id as u32,
        Err(resp) => return resp,
    };

    match VacationRequest::get_by_user_id(&state.db.pool, user_id).await {
        Ok(requests) => json_response(&requests),
        Err(e) => {
            log::error!("Error fetching vacation requests for user {}: {}", user_id, e);
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
        Err(sqlx::Error::RowNotFound) => return HttpResponse::NotFound().body("Utilizador não encontrado."),
        Err(e) => {
            log::error!("Error fetching user's allocated vacation days: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let total_allocated_days = user_details.vacation_days_current_year.unwrap_or(0);
    let current_year = chrono::Utc::now().year();

    let approved_days_taken =
        match VacationRequest::count_approved_vacation_days_for_year(
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
                 let start_date = std::cmp::max(req.start_date, chrono::NaiveDate::from_ymd_opt(current_year, 1, 1).unwrap());
                 let end_date = std::cmp::min(req.end_date, chrono::NaiveDate::from_ymd_opt(current_year, 12, 31).unwrap());
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