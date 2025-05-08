use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FieldType {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PageField {
    pub id: u32,
    pub page_id: u32,
    pub name: String,
    pub display_name: String,
    pub field_type_id: u32,
    pub field_type_name: String, // Joined from field_types
    pub required: bool,
    pub options: Option<serde_json::Value>,
    pub validation_name: Option<String>,
    pub is_searchable: bool,
    pub is_displayed_in_table: bool,
    pub order_index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePageFieldRequest {
    pub display_name: String,
    pub field_type_id: u32,
    pub required: bool,
    pub options: Option<serde_json::Value>,
    pub validation_name: Option<String>,
    pub is_searchable: bool,
    pub is_displayed_in_table: bool,
    pub order_index: u32,
}

impl FieldType {
    pub async fn get_all(pool: &sqlx::MySqlPool) -> Result<Vec<FieldType>, sqlx::Error> {
        sqlx::query_as!(
            FieldType,
            r#"
            SELECT id, name
            FROM field_types
            ORDER BY id
            "#
        )
        .fetch_all(pool)
        .await
    }
}

impl PageField {
    pub async fn get_by_page_id(
        pool: &sqlx::MySqlPool,
        page_id: u32,
    ) -> Result<Vec<PageField>, sqlx::Error> {
        sqlx::query_as!(
            PageField,
            r#"
            SELECT 
                f.id, f.page_id, f.name, f.display_name, 
                f.field_type_id, t.name as field_type_name, 
                f.required as "required: bool", 
                f.options as "options: _", f.validation_name,
                f.is_searchable as "is_searchable: bool", f.is_displayed_in_table as "is_displayed_in_table: bool", f.order_index
            FROM page_fields f
            JOIN field_types t ON f.field_type_id = t.id
            WHERE f.page_id = ?
            ORDER BY f.order_index
            "#,
            page_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn create(
        pool: &sqlx::MySqlPool,
        page_id: u32,
        field: &crate::models::custom_page::CreatePageFieldRequest,
    ) -> Result<u32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO page_fields (
                page_id, name, display_name, field_type_id, required, 
                options, validation_name, is_searchable, is_displayed_in_table, order_index
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            page_id,
            field.name,
            field.display_name,
            field.field_type_id,
            field.required,
            field.options,
            field.validation_name,
            field.is_searchable,
            field.is_displayed_in_table,
            field.order_index
        )
        .execute(pool)
        .await?;

        Ok(result.last_insert_id() as u32)
    }

    pub async fn update(
        pool: &sqlx::MySqlPool,
        field_id: u32,
        request: &UpdatePageFieldRequest,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE page_fields 
            SET 
                display_name = ?, 
                field_type_id = ?, 
                required = ?,
                options = ?, 
                validation_name = ?, 
                is_searchable = ?,
                is_displayed_in_table = ?, 
                order_index = ?
            WHERE id = ?
            "#,
            request.display_name,
            request.field_type_id,
            request.required,
            request.options,
            request.validation_name,
            request.is_searchable,
            request.is_displayed_in_table,
            request.order_index,
            field_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &sqlx::MySqlPool, field_id: u32) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM page_fields WHERE id = ?"#, field_id)
            .execute(pool)
            .await?;

        Ok(())
    }
}
