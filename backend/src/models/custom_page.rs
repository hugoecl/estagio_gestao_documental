use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::field::PageField;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CustomPage {
    pub id: u32,
    pub name: String,
    pub path: String,
    pub parent_path: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCustomPageRequest {
    pub name: String,
    pub path: String,
    pub parent_path: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub fields: Vec<CreatePageFieldRequest>,
    pub permissions: Vec<RolePermissionRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCustomPageRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RolePermissionRequest {
    pub role_id: u32,
    pub can_view: bool,
    pub can_create: bool,
    pub can_edit: bool,
    pub can_delete: bool,
    pub can_manage_fields: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePageFieldRequest {
    pub name: String,
    pub display_name: String,
    pub field_type_id: u32,
    pub required: bool,
    pub options: Option<serde_json::Value>,
    pub validation_name: Option<String>,
    pub is_searchable: bool,
    pub is_displayed_in_table: bool,
    pub order_index: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomPageWithFields {
    pub page: CustomPage,
    pub fields: Vec<PageField>,
    pub permissions: Vec<PagePermission>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PagePermission {
    pub id: u32,
    pub page_id: u32,
    pub role_id: u32,
    pub role_name: String, // Joined from roles table
    pub can_view: bool,
    pub can_create: bool,
    pub can_edit: bool,
    pub can_delete: bool,
    pub can_manage_fields: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NavigationItem {
    pub title: String,
    pub path: Option<String>,
    pub parent_path: Option<String>,
    pub icon: Option<String>,
    pub children: Vec<NavigationItem>,
}

impl CustomPage {
    pub async fn create(
        pool: &sqlx::MySqlPool,
        request: &CreateCustomPageRequest,
    ) -> Result<u32, sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Insert the page
        let result = sqlx::query!(
            r#"
            INSERT INTO custom_pages (name, path, parent_path, description, icon)
            VALUES (?, ?, ?, ?, ?)
            "#,
            request.name,
            request.path,
            request.parent_path,
            request.description,
            request.icon
        )
        .execute(&mut *tx)
        .await?;

        let page_id = result.last_insert_id() as u32;

        // Insert the fields
        for field in &request.fields {
            sqlx::query!(
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
            .execute(&mut *tx)
            .await?;
        }

        // Insert permissions
        for permission in &request.permissions {
            sqlx::query!(
                r#"
                INSERT INTO page_permissions (
                    page_id, role_id, can_view, can_create, can_edit, can_delete, can_manage_fields
                )
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
                page_id,
                permission.role_id,
                permission.can_view,
                permission.can_create,
                permission.can_edit,
                permission.can_delete,
                permission.can_manage_fields
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(page_id)
    }

    pub async fn get_all(pool: &sqlx::MySqlPool) -> Result<Vec<CustomPage>, sqlx::Error> {
        sqlx::query_as!(
            CustomPage,
            r#"
            SELECT 
                id, name, path, parent_path, description, 
                icon, created_at as "created_at!", updated_at as "updated_at!"
            FROM custom_pages
            ORDER BY name
            "#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn get_by_id(
        pool: &sqlx::MySqlPool,
        page_id: u32,
    ) -> Result<CustomPageWithFields, sqlx::Error> {
        let page = sqlx::query_as!(
            CustomPage,
            r#"
            SELECT 
                id, name, path, parent_path, description, 
                icon, created_at as "created_at!", updated_at as "updated_at!"
            FROM custom_pages
            WHERE id = ?
            "#,
            page_id
        )
        .fetch_one(pool)
        .await?;

        let fields = PageField::get_by_page_id(pool, page_id).await?;

        let permissions = sqlx::query_as!(
            PagePermission,
            r#"
            SELECT 
                p.id, p.page_id, p.role_id, r.name as role_name,
                p.can_view as "can_view: bool", p.can_create as "can_create: bool", p.can_edit as "can_edit: bool", 
                p.can_delete as "can_delete: bool", p.can_manage_fields as "can_manage_fields: bool"
            FROM page_permissions p
            JOIN roles r ON p.role_id = r.id
            WHERE p.page_id = ?
            "#,
            page_id
        )
        .fetch_all(pool)
        .await?;

        Ok(CustomPageWithFields {
            page,
            fields,
            permissions,
        })
    }

    pub async fn update(
        pool: &sqlx::MySqlPool,
        page_id: u32,
        request: &UpdateCustomPageRequest,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE custom_pages 
            SET name = ?, description = ?, icon = ?
            WHERE id = ?
            "#,
            request.name,
            request.description,
            request.icon,
            page_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &sqlx::MySqlPool, page_id: u32) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM custom_pages WHERE id = ?"#, page_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_navigation_menu(
        pool: &sqlx::MySqlPool,
    ) -> Result<Vec<NavigationItem>, sqlx::Error> {
        let pages = Self::get_all(pool).await?;

        // Build a tree structure of navigation items
        let mut root_items = Vec::new();
        let mut child_items = std::collections::HashMap::new();

        // Group pages by their parent_path
        for page in &pages {
            if let Some(parent_path) = &page.parent_path {
                child_items
                    .entry(parent_path.clone())
                    .or_insert_with(Vec::new)
                    .push(NavigationItem {
                        title: page.name.clone(),
                        path: Some(page.path.clone()),
                        parent_path: page.parent_path.clone(),
                        icon: page.icon.clone(),
                        children: Vec::new(),
                    });
            } else {
                root_items.push(NavigationItem {
                    title: page.name.clone(),
                    path: Some(page.path.clone()),
                    parent_path: None,
                    icon: page.icon.clone(),
                    children: Vec::new(),
                });
            }
        }

        // Add children to their parents
        for item in &mut root_items {
            if let Some(path) = &item.path {
                if let Some(children) = child_items.get(path) {
                    item.children = children.clone();
                }
            }
        }

        Ok(root_items)
    }
}
