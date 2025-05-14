use chrono::{Duration, NaiveDate, Utc};
use sqlx::MySqlPool;

use crate::{
    auth::get_user_ids_with_view_permission,
    models::{
        field::PageField,
        notification::Notification,
        page_record::PageRecord, // Assuming PageRecord model exists
    },
};

const NOTIFICATION_TYPE_DATE_EXPIRY: &str = "DATE_EXPIRY";

pub async fn check_expiring_date_ranges(pool: &MySqlPool) {
    log::info!("Starting hourly check for expiring date ranges...");

    match PageField::get_notification_enabled_date_range_fields(pool).await {
        Ok(fields) => {
            if fields.is_empty() {
                log::info!("No notification-enabled date range fields found.");
                return;
            }

            let today = Utc::now().date_naive();

            for field in fields {
                // Ensure required notification config is present
                let (Some(days_before), Some(target_part)) = (
                    field.notification_days_before,
                    field.notification_target_date_part.as_ref(),
                ) else {
                    log::warn!(
                        "Skipping field ID {} due to missing notification configuration (days_before or target_part)",
                        field.id
                    );
                    continue;
                };

                // Fetch records for the page
                match PageRecord::get_by_page_id(pool, field.page_id).await {
                    Ok(records) => {
                        for record in records {
                            log::trace!("Processing record ID: {}", record.id);

                            // Extract the relevant date from the record's data JSON
                            let due_date_opt =
                                extract_target_date(&record.data, &field.name, target_part);

                            let due_date = match due_date_opt {
                                Some(date) => date,
                                None => {
                                    continue; // Skip record if date is not found or invalid
                                }
                            };

                            // Calculate when the notification should trigger
                            let notification_trigger_date = match due_date
                                .checked_sub_signed(Duration::days(days_before as i64))
                            {
                                Some(date) => date,
                                None => {
                                    log::warn!(
                                        "Could not calculate trigger date for record {}, field {}, due date {}",
                                        record.id,
                                        field.id,
                                        due_date
                                    );
                                    continue;
                                }
                            };

                            // Check if today is the day to notify (or if we missed it slightly)
                            if today >= notification_trigger_date && today < due_date {
                                // Find users who can view this page
                                match get_user_ids_with_view_permission(pool, field.page_id).await {
                                    Ok(user_ids) => {
                                        for user_id in user_ids {
                                            // Check if a notification for this event already exists
                                            match Notification::check_if_event_notification_exists(
                                                pool,
                                                user_id,
                                                record.id,
                                                NOTIFICATION_TYPE_DATE_EXPIRY,
                                                Some(due_date), // Pass the specific due_date
                                            )
                                            .await
                                            {
                                                Ok(exists) => {
                                                    if !exists {
                                                        // Create notification
                                                        let message = format!(
                                                            "O prazo para '{}' no registo #{} está a aproximar-se ({}).",
                                                            field.display_name, // Use field display name
                                                            record.id,
                                                            due_date.format("%d/%m/%Y")
                                                        );

                                                        match Notification::create(
                                                            pool,
                                                            user_id,
                                                            Some(record.id),
                                                            Some(field.page_id),
                                                            Some(field.id),
                                                            NOTIFICATION_TYPE_DATE_EXPIRY,
                                                            &message,
                                                            Some(due_date),
                                                        )
                                                        .await
                                                        {
                                                            Ok(_) => log::info!(
                                                                "Created notification for user {}, record {}",
                                                                user_id,
                                                                record.id
                                                            ),
                                                            Err(e) => log::error!(
                                                                "Failed to create notification for user {}, record {}: {}",
                                                                user_id,
                                                                record.id,
                                                                e
                                                            ),
                                                        }
                                                    } else {
                                                        log::trace!(
                                                            "Unread notification already exists for user {}, record {}",
                                                            user_id,
                                                            record.id
                                                        );
                                                    }
                                                }
                                                Err(e) => log::error!(
                                                    "Error checking existing notification for user {}, record {}: {}",
                                                    user_id,
                                                    record.id,
                                                    e
                                                ),
                                            }
                                        }
                                    }
                                    Err(e) => log::error!(
                                        "Failed to get users with view permission for page {}: {}",
                                        field.page_id,
                                        e
                                    ),
                                }
                            } else {
                                log::trace!(
                                    "Record ID {} not within notification window today (due: {}, trigger: {}, today: {})",
                                    record.id,
                                    due_date,
                                    notification_trigger_date,
                                    today
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to fetch records for page {}: {}", field.page_id, e)
                    }
                }
            }
        }
        Err(e) => log::error!("Failed to fetch notification-enabled fields: {}", e),
    }
    log::info!("Finished hourly check for expiring date ranges.");
}

/// Helper to extract and parse date from JSON based on target part ("start_date" or "end_date")
/// Expects the value associated with `field_name` to be a string "DD/MM/YYYY - DD/MM/YYYY".
fn extract_target_date(
    data: &serde_json::Value,
    field_name: &str,
    target_part: &str,
) -> Option<NaiveDate> {
    log::trace!(
        "Attempting to extract target date '{}' for field '{}' from data: {:?}",
        target_part,
        field_name,
        data
    );

    data.get(field_name)
        .and_then(|value| value.as_str()) // Expect a string like "DD/MM/YYYY - DD/MM/YYYY"
        .and_then(|date_range_str| {
            let parts: Vec<&str> = date_range_str.splitn(2, " - ").collect();
            if parts.len() != 2 {
                log::warn!(
                    "Invalid date range format for field '{}': {}",
                    field_name,
                    date_range_str
                );
                return None; // Invalid format
            }

            let date_str_to_parse = if target_part == "start_date" {
                parts[0]
            } else if target_part == "end_date" {
                parts[1]
            } else {
                log::error!("Invalid target_part specified: {}", target_part);
                return None; // Invalid target_part
            };

            // Parse the "DD/MM/YYYY" string
            NaiveDate::parse_from_str(date_str_to_parse, "%d/%m/%Y").ok()
        })
}
