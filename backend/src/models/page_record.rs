use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PageRecord {
    pub id: u32,
    pub page_id: u32,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: u32,
    pub updated_by: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePageRecordRequest {
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePageRecordRequest {
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PageRecordFile {
    pub id: u32,
    pub record_id: u32,
    pub file_name: String,
    pub file_path: String,
    pub uploaded_at: DateTime<Utc>,
    pub uploaded_by: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageRecordWithFiles {
    pub record: PageRecord,
    pub files: Vec<PageRecordFile>,
}

impl PageRecord {
    pub async fn create(
        pool: &sqlx::MySqlPool,
        request: &CreatePageRecordRequest,
        page_id: u32,
        user_id: u32,
    ) -> Result<u32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO page_records (page_id, data, created_by, updated_by)
            VALUES (?, ?, ?, ?)
            "#,
            page_id,
            request.data,
            user_id,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.last_insert_id() as u32)
    }

    pub async fn get_by_page_id(
        pool: &sqlx::MySqlPool,
        page_id: u32,
    ) -> Result<Vec<PageRecord>, sqlx::Error> {
        let records = sqlx::query_as!(
            PageRecord,
            r#"
            SELECT
                id,
                page_id,
                data as `data: serde_json::Value`,
                created_at as "created_at!",
                updated_at as "updated_at!",
                created_by,
                updated_by
            FROM page_records
            WHERE page_id = ?
            "#,
            page_id
        )
        .fetch_all(pool) // Pass pool directly, not &pool
        .await?; // Propagate potential error

        Ok(records) // Wrap the successful result in Ok
    }

    pub async fn get_by_id(
        pool: &sqlx::MySqlPool,
        record_id: u32,
    ) -> Result<PageRecordWithFiles, sqlx::Error> {
        let record = sqlx::query_as!(
            PageRecord,
            r#"
            SELECT
                id,
                page_id,
                data as `data: serde_json::Value`,
                created_at as "created_at!",
                updated_at as "updated_at!",
                created_by,
                updated_by
            FROM page_records
            WHERE id = ?
            "#,
            record_id
        )
        .fetch_one(pool)
        .await?;

        let files = sqlx::query_as!(
            PageRecordFile,
            r#"
            SELECT id, record_id, file_name, file_path, uploaded_at as "uploaded_at!", uploaded_by
            FROM page_record_files
            WHERE record_id = ?
            "#,
            record_id
        )
        .fetch_all(pool)
        .await?;

        Ok(PageRecordWithFiles { record, files })
    }

    pub async fn update(
        pool: &sqlx::MySqlPool,
        record_id: u32,
        request: &UpdatePageRecordRequest,
        user_id: u32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE page_records
            SET data = ?, updated_by = ?
            WHERE id = ?
            "#,
            request.data,
            user_id,
            record_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &sqlx::MySqlPool, record_id: u32) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM page_records WHERE id = ?"#, record_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn add_file(
        pool: &sqlx::MySqlPool,
        record_id: u32,
        file_name: &str,
        file_path: &str,
        user_id: u32,
    ) -> Result<u32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO page_record_files (record_id, file_name, file_path, uploaded_by)
            VALUES (?, ?, ?, ?)
            "#,
            record_id,
            file_name,
            file_path,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.last_insert_id() as u32)
    }

    pub async fn delete_file(pool: &sqlx::MySqlPool, file_id: u32) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM page_record_files WHERE id = ?"#, file_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn search_records(
        pool: &sqlx::MySqlPool,
        page_id: u32,
        search_term: &str,
    ) -> Result<Vec<PageRecord>, sqlx::Error> {
        // Get all records for the page
        let records = Self::get_by_page_id(pool, page_id).await?;

        // If search term is empty, return all records
        if search_term.is_empty() {
            return Ok(records);
        }

        // Get searchable fields for the page
        let fields = crate::models::field::PageField::get_by_page_id(pool, page_id).await?;
        let searchable_fields: Vec<&crate::models::field::PageField> =
            fields.iter().filter(|f| f.is_searchable).collect();

        // Filter records by search term
        let search_term = search_term.to_lowercase();
        let filtered_records = records
            .into_iter()
            .filter(|record| {
                if let serde_json::Value::Object(obj) = &record.data {
                    for field in &searchable_fields {
                        if let Some(value) = obj.get(&field.name) {
                            let value_str = value.to_string().to_lowercase();
                            if value_str.contains(&search_term) {
                                return true;
                            }
                        }
                    }
                }
                false
            })
            .collect();

        Ok(filtered_records)
    }

    pub async fn get_page_id_for_record(pool: &sqlx::MySqlPool, record_id: u32) -> Result<u32, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT page_id FROM page_records WHERE id = ?"#,
            record_id
        )
        .fetch_one(pool)
        .await?;
        Ok(result)
    }
}
