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

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct VacationRequest {
    pub id: u32,
    pub user_id: u32,
    pub start_date: NaiveDate, // SQL DATE
    pub end_date: NaiveDate,   // SQL DATE
    pub status: VacationRequestStatus,
    pub notes: Option<String>, // User's notes on request, or admin's notes on action
    pub requested_at: DateTime<Utc>,
    pub approved_by: Option<u32>, // User ID of the admin who actioned the request
    pub actioned_at: Option<DateTime<Utc>>, // Timestamp of approval/rejection
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
        sqlx::query_as!(
            VacationRequest,
            r#"
            SELECT
                id, user_id, start_date, end_date,
                status AS "status: _",
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
        .await
    }

    pub async fn get_by_user_id(
        pool: &MySqlPool,
        user_id: u32,
    ) -> Result<Vec<VacationRequest>, sqlx::Error> {
        sqlx::query_as!(
            VacationRequest,
            r#"
            SELECT
                id, user_id, start_date, end_date,
                status AS "status: _",
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
        .await
    }

    /// Fetches all PENDING vacation requests, optionally filtered by a list of user IDs (for a specific role).
    pub async fn get_pending_requests_for_users(
        pool: &MySqlPool,
        user_ids: Option<&[u32]>, // If None, fetch for all users. If Some, filter by these user IDs.
    ) -> Result<Vec<VacationRequest>, sqlx::Error> {
        // The query_as! macro cannot be used directly with dynamically formatted strings.
        // We need to use query_as directly with the sqlx::QueryAs structure.
        // However, the current `VacationRequest` struct doesn't have `user_username`.
        // For simplicity, I'll fetch without username here.
        // The admin view will likely need a dedicated struct or a different query.
        // For now, let's adjust the query to match the `VacationRequest` struct.

        let dynamic_query_str = if let Some(ids) = user_ids {
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            format!(
                r#"
                SELECT
                    id, user_id,
                    start_date, end_date,
                    status AS "status: _",
                    notes,
                    requested_at AS "requested_at!",
                    approved_by,
                    actioned_at
                FROM vacation_requests
                WHERE status = 'PENDING' AND user_id IN ({})
                ORDER BY requested_at ASC
                "#,
                ids.iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            )
        } else {
            r#"
            SELECT
                id, user_id,
                start_date, end_date,
                status AS "status: _",
                notes,
                requested_at AS "requested_at!",
                approved_by,
                actioned_at
            FROM vacation_requests
            WHERE status = 'PENDING'
            ORDER BY requested_at ASC
            "#
            .to_string()
        };

        sqlx::query_as(&dynamic_query_str).fetch_all(pool).await
    }

    /// Admin actions a vacation request (approve or reject).
    pub async fn action_request(
        pool: &MySqlPool,
        request_id: u32,
        admin_id: u32,
        new_status: VacationRequestStatus,
        admin_notes: Option<String>,
    ) -> Result<u64, sqlx::Error> {
        if new_status == VacationRequestStatus::Pending {
            // Admins should only approve or reject
            return Err(sqlx::Error::Protocol(
                "Admin action cannot set status to PENDING.".into(),
            ));
        }

        let result = sqlx::query!(
            r#"
            UPDATE vacation_requests
            SET status = ?, approved_by = ?, actioned_at = CURRENT_TIMESTAMP, notes = ?
            WHERE id = ? AND status = 'PENDING'
            "#,
            new_status as VacationRequestStatus,
            admin_id,
            admin_notes,
            request_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
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