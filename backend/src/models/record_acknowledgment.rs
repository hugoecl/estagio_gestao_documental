use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RecordAcknowledgment {
    pub id: u32,
    pub user_id: u32,
    pub record_id: u32,
    pub acknowledged_at: DateTime<Utc>,
}

// Struct for returning acknowledgment details with username
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AcknowledgmentDetail {
    pub user_id: u32,
    pub username: String, // Joined from users table
    pub email: String,    // Joined from users table
    pub record_id: u32,
    pub acknowledged_at: DateTime<Utc>,
}

impl RecordAcknowledgment {
    /// Creates a new acknowledgment for a user and record.
    /// Returns Ok(true) if a new acknowledgment was created.
    /// Returns Ok(false) if the user had already acknowledged this record (no new row inserted).
    /// Returns Err on database error.
    pub async fn create(
        pool: &MySqlPool,
        user_id: u32,
        record_id: u32,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT IGNORE INTO record_acknowledgments (user_id, record_id)
            VALUES (?, ?)
            "#,
            user_id,
            record_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0) // True if a new row was inserted
    }

    /// Fetches all acknowledgment details (including username) for a specific record.
    pub async fn get_acknowledgments_for_record(
        pool: &MySqlPool,
        record_id: u32,
    ) -> Result<Vec<AcknowledgmentDetail>, sqlx::Error> {
        sqlx::query_as!(
            AcknowledgmentDetail,
            r#"
            SELECT
                ra.user_id,
                u.username,
                u.email,
                ra.record_id,
                ra.acknowledged_at as "acknowledged_at!"
            FROM record_acknowledgments ra
            JOIN users u ON ra.user_id = u.id
            WHERE ra.record_id = ?
            ORDER BY ra.acknowledged_at DESC
            "#,
            record_id
        )
        .fetch_all(pool)
        .await
    }

    /// Checks if a specific user has acknowledged a specific record.
    pub async fn has_user_acknowledged(
        pool: &MySqlPool,
        user_id: u32,
        record_id: u32,
    ) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM record_acknowledgments
            WHERE user_id = ? AND record_id = ?
            "#,
            user_id,
            record_id
        )
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }
}
