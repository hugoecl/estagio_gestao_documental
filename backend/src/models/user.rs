use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::role::Role; // Import Role if needed for embedding

// Basic User structure (without password)
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub vacation_days_current_year: Option<u16>, // Added field
}

// Structure to hold user data along with their assigned roles
#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithRoles {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub roles: Vec<Role>, // Embed the full Role struct or just IDs/names
}

// Structure used for fetching from DB before aggregation
#[derive(FromRow, Debug)]
pub struct UserRoleRow {
    pub user_id: u32,
    pub username: String,
    pub email: String,
    pub role_id: Option<u32>, // Use Option for LEFT JOIN
    pub role_name: Option<String>,
    pub role_description: Option<String>,
    pub role_is_admin: Option<bool>, // MySQL boolean can be tinyint
    pub role_created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub role_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
