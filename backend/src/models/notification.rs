use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Define public notification type constants that can be used across the application
pub const NOTIFICATION_TYPE_DATE_EXPIRY: &str = "DATE_EXPIRY";
pub const NOTIFICATION_TYPE_ADMIN_BROADCAST: &str = "ADMIN_BROADCAST";
pub const NOTIFICATION_TYPE_VACATION_APPROVED: &str = "VACATION_APPROVED";
pub const NOTIFICATION_TYPE_VACATION_REJECTED: &str = "VACATION_REJECTED";
pub const NOTIFICATION_TYPE_VACATION_CANCELED: &str = "VACATION_CANCELED";
pub const NOTIFICATION_TYPE_VACATION_CANCELLATION_REJECTED: &str = "VACATION_CANCELLATION_REJECTED";
pub const NOTIFICATION_TYPE_VACATION_REQUESTED: &str = "VACATION_REQUESTED";

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: u32,
    pub user_id: u32,
    pub record_id: Option<u32>,  // For page records
    pub vacation_request_id: Option<u32>, // Added for vacation requests
    pub page_id: Option<u32>,
    pub field_id: Option<u32>,
    pub notification_type: String,
    pub message: String,
    pub due_date: Option<NaiveDate>,
    #[serde(rename = "isRead")]
    pub is_read: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    // Optional fields that might be joined in queries later
    // #[sqlx(skip)] // Skip direct mapping if not always present
    // pub record_display_name: Option<String>, // Example: Might join record data later
    // #[sqlx(skip)]
    // pub page_name: Option<String>, // Example: Might join page name later
    // #[sqlx(skip)]
    // pub page_path: Option<String>, // Example: Might join page path later
}

// Optional: Struct for API response that might include joined data
#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: u32,
    #[serde(rename = "userId")]
    pub user_id: u32,
    #[serde(rename = "recordId")]
    pub record_id: Option<u32>,
    #[serde(rename = "vacationRequestId")]
    pub vacation_request_id: Option<u32>, // Added for vacation requests
    #[serde(rename = "pageId")]
    pub page_id: Option<u32>,
    #[serde(rename = "fieldId")]
    pub field_id: Option<u32>,
    #[serde(rename = "notificationType")]
    pub notification_type: String,
    pub message: String,
    #[serde(rename = "dueDate")]
    pub due_date: Option<NaiveDate>,
    #[serde(rename = "isRead")]
    pub is_read: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    // Add fields fetched via JOINs
    #[serde(rename = "pagePath", skip_serializing_if = "Option::is_none")]
    pub page_path: Option<String>,
    #[serde(rename = "pageName", skip_serializing_if = "Option::is_none")]
    pub page_name: Option<String>,
    // Maybe a snippet of the record data for context?
    // pub record_snippet: Option<String>,
}

impl Notification {
    // --- Database Interaction Functions ---

    // Fetch unread notifications for a user
    pub async fn get_unread_by_user(
        pool: &sqlx::MySqlPool,
        user_id: u32,
    ) -> Result<Vec<NotificationResponse>, sqlx::Error> {
        // Join with custom_pages to get path and name
        sqlx::query_as!(
            NotificationResponse, // Map directly to the response struct
            r#"
            SELECT
                n.id, n.user_id, n.record_id, n.vacation_request_id,
                n.page_id, n.field_id, n.notification_type, 
                n.message, n.due_date, n.is_read as "is_read: bool", 
                n.created_at as "created_at!", cp.path as page_path, 
                cp.name as page_name
            FROM notifications n
            LEFT JOIN custom_pages cp ON n.page_id = cp.id
            WHERE n.user_id = ? AND n.is_read = false
            ORDER BY n.created_at DESC
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
    }

    // Fetch count of unread notifications for a user
    pub async fn count_unread_by_user(
        pool: &sqlx::MySqlPool,
        user_id: u32,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM notifications
            WHERE user_id = ? AND is_read = false
            "#,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(result.count)
    }

    // Mark specific notifications as read for a user
    pub async fn mark_as_read(
        pool: &sqlx::MySqlPool,
        user_id: u32,
        notification_ids: &[u32],
    ) -> Result<u64, sqlx::Error> {
        if notification_ids.is_empty() {
            return Ok(0); // No rows affected if no IDs provided
        }

        // Construct the query string with placeholders
        let placeholders = notification_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let query_str = format!(
            r#"
            UPDATE notifications
            SET is_read = true
            WHERE user_id = ? AND id IN ({}) AND is_read = false
            "#,
            placeholders
        ); // Only update unread ones

        let mut query = sqlx::query(&query_str).bind(user_id);

        for id in notification_ids {
            query = query.bind(id);
        }

        let result = query.execute(pool).await?;
        Ok(result.rows_affected())
    }

    // Mark ALL notifications as read for a user
    pub async fn mark_all_as_read(
        pool: &sqlx::MySqlPool,
        user_id: u32,
    ) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE notifications
            SET is_read = true
            WHERE user_id = ? AND is_read = false
            "#,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Checks if any notification (read or unread) for a specific user/record/type/due_date already exists.
    /// This helps prevent re-notifying for the exact same expiring event if it has already been created.
    pub async fn check_if_event_notification_exists(
        pool: &sqlx::MySqlPool,
        user_id: u32,
        record_id: u32,
        notification_type: &str,
        due_date: Option<NaiveDate>, // Add due_date to make the check event-specific
    ) -> Result<bool, sqlx::Error> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) 
            FROM notifications 
            WHERE user_id = ? 
              AND record_id = ? 
              AND notification_type = ? 
              AND due_date <=> ?  -- Use NULL-safe equality for due_date
            "#,
            user_id,
            record_id,
            notification_type,
            due_date
        )
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }

    /// Creates a new notification.
    pub async fn create(
        pool: &sqlx::MySqlPool,
        user_id: u32,
        record_id: Option<u32>, 
        vacation_request_id: Option<u32>, // Added for vacation requests
        page_id: Option<u32>,
        field_id: Option<u32>,
        notification_type: &str,
        message: &str,
        due_date: Option<NaiveDate>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO notifications
                (user_id, record_id, vacation_request_id, page_id, field_id, notification_type, message, due_date, is_read)
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, false)
            "#,
            user_id,
            record_id,
            vacation_request_id,
            page_id,
            field_id,
            notification_type,
            message,
            due_date
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
