use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, sqlx::Type)]
#[sqlx(type_name = "ENUM('PENDING', 'APPROVED', 'REJECTED')", rename_all = "UPPERCASE")]
pub enum VacationRequestStatus {
    Pending,
    Approved,
    Rejected,
}

// Intermediate struct for fetching from DB, with status as String
#[derive(Debug, FromRow, Clone)]
struct VacationRequestDbRow {
    pub id: u32,
    pub user_id: u32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: String, // Fetched as String
    pub notes: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub approved_by: Option<u32>,
    pub actioned_at: Option<DateTime<Utc>>,
}

// Main struct with proper enum, used in application logic and API responses
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VacationRequest {
    pub id: u32,
    pub user_id: u32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: VacationRequestStatus, // Proper enum
    pub notes: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub approved_by: Option<u32>,
    pub actioned_at: Option<DateTime<Utc>>,
}

impl From<VacationRequestDbRow> for VacationRequest {
    fn from(db_row: VacationRequestDbRow) -> Self {
        let status_enum = match db_row.status.as_str() {
            "PENDING" => VacationRequestStatus::Pending,
            "APPROVED" => VacationRequestStatus::Approved,
            "REJECTED" => VacationRequestStatus::Rejected,
            _ => {
                log::warn!("Unknown vacation status string '{}', defaulting to Pending", db_row.status);
                VacationRequestStatus::Pending // Default or handle error
            }
        };
        VacationRequest {
            id: db_row.id,
            user_id: db_row.user_id,
            start_date: db_row.start_date,
            end_date: db_row.end_date,
            status: status_enum,
            notes: db_row.notes,
            requested_at: db_row.requested_at,
            approved_by: db_row.approved_by,
            actioned_at: db_row.actioned_at,
        }
    }
}

// DTO for creating a new vacation request by a user
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVacationRequest {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub notes: Option<String>,
}

// DTO for an admin to action a vacation request
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionVacationRequest {
    pub status: VacationRequestStatus, // Will be either APPROVED or REJECTED
    pub admin_notes: Option<String>,   // Admin's notes for the action
}

// Intermediate struct for fetching from DB, with status as String
#[derive(Debug, FromRow, Clone)]
struct VacationRequestWithUserDbRow {
    pub id: u32,
    pub user_id: u32,
    pub username: String,
    pub email: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: String, // Fetched as String
    pub notes: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub approved_by: Option<u32>,
    pub actioned_at: Option<DateTime<Utc>>,
}

// Main struct for API responses, with proper enum
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VacationRequestWithUser {
    pub id: u32,
    pub user_id: u32,
    pub username: String,
    pub email: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: VacationRequestStatus, // Proper enum
    pub notes: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub approved_by: Option<u32>,
    pub actioned_at: Option<DateTime<Utc>>,
}

impl From<VacationRequestWithUserDbRow> for VacationRequestWithUser {
    fn from(db_row: VacationRequestWithUserDbRow) -> Self {
        let status_enum = match db_row.status.as_str() {
            "PENDING" => VacationRequestStatus::Pending,
            "APPROVED" => VacationRequestStatus::Approved,
            "REJECTED" => VacationRequestStatus::Rejected,
            _ => {
                log::warn!("Unknown vacation status string '{}', defaulting to Pending", db_row.status);
                VacationRequestStatus::Pending // Default or handle error
            }
        };
        VacationRequestWithUser {
            id: db_row.id,
            user_id: db_row.user_id,
            username: db_row.username,
            email: db_row.email,
            start_date: db_row.start_date,
            end_date: db_row.end_date,
            status: status_enum,
            notes: db_row.notes,
            requested_at: db_row.requested_at,
            approved_by: db_row.approved_by,
            actioned_at: db_row.actioned_at,
        }
    }
}

impl VacationRequest {
    pub async fn create(
        pool: &MySqlPool,
        user_id: u32,
        request_data: &CreateVacationRequest,
    ) -> Result<u32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO vacation_requests (user_id, start_date, end_date, status, notes)
            VALUES (?, ?, ?, ?, ?)
            "#,
            user_id,
            request_data.start_date,
            request_data.end_date,
            VacationRequestStatus::Pending as VacationRequestStatus, // New requests are always PENDING
            request_data.notes
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_id() as u32)
    }

    pub async fn get_by_id(
        pool: &MySqlPool,
        request_id: u32,
    ) -> Result<Option<VacationRequest>, sqlx::Error> {
        let row_opt = sqlx::query_as!(
            VacationRequestDbRow, // Fetch as DbRow first
            r#"
            SELECT
                id, user_id, start_date, end_date,
                status,
                notes,
                requested_at AS "requested_at!",
                approved_by,
                actioned_at
            FROM vacation_requests
            WHERE id = ?
            "#,
            request_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(row_opt.map(VacationRequest::from)) // Convert to VacationRequest
    }

    pub async fn get_by_user_id(
        pool: &MySqlPool,
        user_id: u32,
    ) -> Result<Vec<VacationRequest>, sqlx::Error> {
        let rows = sqlx::query_as!(
            VacationRequestDbRow, // Fetch as DbRow first
            r#"
            SELECT
                id, user_id, start_date, end_date,
                status,
                notes,
                requested_at AS "requested_at!",
                approved_by,
                actioned_at
            FROM vacation_requests
            WHERE user_id = ?
            ORDER BY start_date DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(VacationRequest::from).collect()) // Convert each row
    }

    /// Fetches all PENDING vacation requests, optionally filtered by a list of user IDs (for a specific role),
    /// including the username and email of the user who made the request.
    pub async fn get_pending_requests_with_users(
        pool: &MySqlPool,
        user_ids: Option<&[u32]>,
    ) -> Result<Vec<VacationRequestWithUser>, sqlx::Error> {
        let base_query = r#"
            SELECT
                vr.id, vr.user_id, u.username, u.email,
                vr.start_date, vr.end_date,
                vr.status, // Fetch status as String
                vr.notes,
                vr.requested_at AS "requested_at!",
                vr.approved_by,
                vr.actioned_at
            FROM vacation_requests vr
            JOIN users u ON vr.user_id = u.id
            WHERE vr.status = 'PENDING'
        "#;

        let query_str = if let Some(ids) = user_ids {
            if ids.is_empty() {
                return Ok(Vec::new()); // No users to fetch for, return empty
            }
            format!(
                "{} AND vr.user_id IN ({}) ORDER BY vr.requested_at ASC",
                base_query,
                ids.iter().map(|id| id.to_string()).collect::<Vec<String>>().join(",")
            )
        } else {
            // Fetch for all users if user_ids is None (though admin might usually filter by role)
            format!("{} ORDER BY vr.requested_at ASC", base_query)
        };
        
        let rows = sqlx::query_as::<_, VacationRequestWithUserDbRow>(&query_str) // Fetch as DbRow
            .fetch_all(pool)
            .await?;

        Ok(rows.into_iter().map(VacationRequestWithUser::from).collect()) // Convert
    }

    /// Admin actions a vacation request (approve or reject), and deducts days if approved.
    /// Returns Ok(true) if action was successful (and days deducted if approved).
    /// Returns Ok(false) if the request was not in PENDING state (already actioned).
    /// Returns Err for database errors or if not enough vacation days for approval.
    pub async fn action_request_with_days_deduction(
        pool: &MySqlPool,
        request_id: u32,
        admin_id: u32,
        new_status: VacationRequestStatus,
        admin_notes: Option<String>,
    ) -> Result<bool, sqlx::Error> {
        if new_status == VacationRequestStatus::Pending {
            return Err(sqlx::Error::Protocol(
                "Admin action cannot set status to PENDING.".into(),
            ));
        }

        let mut tx = pool.begin().await?;

        // Fetch the request details to get user_id, start_date, and end_date
        // Fetch status as String first, then convert
        let request_details_raw = sqlx::query!(
            r#"
            SELECT user_id, start_date, end_date, status
            FROM vacation_requests
            WHERE id = ?
            "#,
            request_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let (user_id_for_deduction, request_start_date, request_end_date, current_db_status_str) =
            match request_details_raw {
                Some(req) => (req.user_id, req.start_date, req.end_date, req.status),
                None => {
                    tx.rollback().await?;
                    return Err(sqlx::Error::RowNotFound); // Request not found
                }
            };
        
        // Convert status string to enum
        let current_db_status: VacationRequestStatus = match current_db_status_str.as_str() {
            "PENDING" => VacationRequestStatus::Pending,
            "APPROVED" => VacationRequestStatus::Approved,
            "REJECTED" => VacationRequestStatus::Rejected,
            _ => {
                log::error!("Invalid status string '{}' from database for request_id: {}", current_db_status_str, request_id);
                tx.rollback().await?;
                return Err(sqlx::Error::Decode("Invalid status string from database".into()));
            }
        };

        // Ensure the request is currently PENDING before actioning
        if current_db_status != VacationRequestStatus::Pending {
            tx.rollback().await?;
            log::warn!("Attempted to action non-pending request_id: {}, current status: {:?}", request_id, current_db_status);
            return Ok(false); // Indicate request was not actioned because not pending
        }


        if new_status == VacationRequestStatus::Approved {
            // Calculate number of days for the request
            let duration_days = (request_end_date - request_start_date).num_days() + 1;
            if duration_days <= 0 {
                tx.rollback().await?;
                return Err(sqlx::Error::Protocol("Invalid request duration.".into()));
            }

            // Fetch user's current available vacation days
            let user_vacation_days = sqlx::query_scalar!(
                "SELECT vacation_days_current_year FROM users WHERE id = ?",
                user_id_for_deduction
            )
            .fetch_one(&mut *tx)
            .await?
            .unwrap_or(0); // Default to 0 if NULL

            if user_vacation_days < duration_days as u16 {
                tx.rollback().await?;
                return Err(sqlx::Error::Protocol(
                    "Not enough vacation days available.".into(),
                ));
            }

            // Deduct days
            let new_available_days = user_vacation_days - duration_days as u16;
            sqlx::query!(
                "UPDATE users SET vacation_days_current_year = ? WHERE id = ?",
                new_available_days,
                user_id_for_deduction
            )
            .execute(&mut *tx)
            .await?;
        }

        // Update the vacation request status
        let update_result = sqlx::query!(
            r#"
            UPDATE vacation_requests
            SET status = ?, approved_by = ?, actioned_at = CURRENT_TIMESTAMP, notes = ?
            WHERE id = ? AND status = 'PENDING' 
            "#, // Ensure it's still PENDING to avoid race conditions
            new_status as VacationRequestStatus,
            admin_id,
            admin_notes,
            request_id
        )
        .execute(&mut *tx)
        .await?;

        if update_result.rows_affected() == 0 {
             // This means the request was not in PENDING state when update was attempted (e.g. actioned by another admin)
            tx.rollback().await?;
            log::warn!("Vacation request {} was already actioned or not found when trying to update status.", request_id);
            return Ok(false); 
        }

        tx.commit().await?;
        Ok(true) // Successfully actioned
    }

    /// Fetches all approved vacation dates for a list of user IDs within a given year.
    /// Used for conflict checking and displaying on shared calendars.
    pub async fn get_approved_dates_for_users_in_year(
        pool: &MySqlPool,
        user_ids: &[u32],
        year: i32,
    ) -> Result<Vec<(NaiveDate, NaiveDate)>, sqlx::Error> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }

        let start_of_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let end_of_year = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

        // Constructing the IN clause dynamically for user_ids
        let user_ids_placeholder = user_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let query_str = format!(
            r#"
            SELECT start_date, end_date
            FROM vacation_requests
            WHERE user_id IN ({})
              AND status = 'APPROVED'
              AND (
                  (start_date <= ? AND end_date >= ?) OR 
                  (start_date <= ? AND end_date >= ?) OR 
                  (start_date >= ? AND end_date <= ?)    
              )
            "#,
            user_ids_placeholder
        );
       
        let mut query_builder = sqlx::query(&query_str);
        for user_id in user_ids {
            query_builder = query_builder.bind(user_id);
        }
        query_builder = query_builder
            .bind(end_of_year) 
            .bind(start_of_year)
            .bind(end_of_year) 
            .bind(start_of_year)
            .bind(start_of_year) 
            .bind(end_of_year);

        let rows = query_builder
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|row| {
                let start_date: NaiveDate = sqlx::Row::get(&row, "start_date");
                let end_date: NaiveDate = sqlx::Row::get(&row, "end_date");
                (start_date, end_date)
            })
            .collect();

        Ok(rows)
    }

    pub async fn get_approved_and_pending_dates_for_users_in_year(
        pool: &MySqlPool,
        user_ids: &[u32],
        year: i32,
    ) -> Result<Vec<(NaiveDate, NaiveDate, String)>, sqlx::Error> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }

        let start_of_year = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let end_of_year = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

        // Constructing the IN clause dynamically for user_ids
        let user_ids_placeholder = user_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let query_str = format!(
            r#"
            SELECT start_date, end_date, status
            FROM vacation_requests
            WHERE user_id IN ({})
              AND status IN ('APPROVED', 'PENDING')
              AND (
                  (start_date <= ? AND end_date >= ?) OR 
                  (start_date <= ? AND end_date >= ?) OR 
                  (start_date >= ? AND end_date <= ?)    
              )
            "#,
            user_ids_placeholder
        );
       
        let mut query_builder = sqlx::query(&query_str);
        for user_id in user_ids {
            query_builder = query_builder.bind(user_id);
        }
        query_builder = query_builder
            .bind(end_of_year) 
            .bind(start_of_year)
            .bind(end_of_year) 
            .bind(start_of_year)
            .bind(start_of_year) 
            .bind(end_of_year);

        let rows = query_builder
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|row| {
                let start_date: NaiveDate = sqlx::Row::get(&row, "start_date");
                let end_date: NaiveDate = sqlx::Row::get(&row, "end_date");
                let status: String = sqlx::Row::get(&row, "status");
                (start_date, end_date, status)
            })
            .collect();

        Ok(rows)
    }

    // Function to count the number of approved vacation days for a user in a given year.
    pub async fn count_approved_vacation_days_for_year(
        pool: &MySqlPool,
        user_id: u32,
        year: i32,
    ) -> Result<i64, sqlx::Error> {
        // Define the start and end of the year for filtering
        let year_start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let year_end = NaiveDate::from_ymd_opt(year, 12, 31).unwrap();

        let approved_requests = sqlx::query!(
            r#"
            SELECT start_date, end_date
            FROM vacation_requests
            WHERE user_id = ?
              AND status = 'APPROVED'
              AND start_date <= ? 
              AND end_date >= ?   
            "#,
            user_id,
            year_end,   
            year_start, 
        )
        .fetch_all(pool)
        .await?;

        let mut total_days_in_year = 0;

        for req in approved_requests {
            // Clamp the request dates to the given year
            let effective_start_date = std::cmp::max(req.start_date, year_start);
            let effective_end_date = std::cmp::min(req.end_date, year_end);

            // Calculate duration if effective_start_date is not after effective_end_date
            if effective_start_date <= effective_end_date {
                total_days_in_year += (effective_end_date - effective_start_date).num_days() + 1;
            }
        }
        Ok(total_days_in_year)
    }
}