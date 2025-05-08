use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleAssignment {
    pub user_id: u32,
    pub role_ids: Vec<u32>,
}

impl Role {
    pub async fn create(
        pool: &sqlx::MySqlPool,
        request: &CreateRoleRequest,
    ) -> Result<u32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            INSERT INTO roles (name, description, is_admin)
            VALUES (?, ?, ?)
            "#,
            request.name,
            request.description,
            request.is_admin
        )
        .execute(pool)
        .await?;

        Ok(result.last_insert_id() as u32)
    }

    pub async fn get_all(pool: &sqlx::MySqlPool) -> Result<Vec<Role>, sqlx::Error> {
        sqlx::query_as!(
            Role,
            r#"
            SELECT id, name, description, is_admin as "is_admin: bool", created_at as "created_at!", updated_at as "updated_at!"
            FROM roles
            ORDER BY name
            "#
        )
        .fetch_all(pool)
        .await
    }

    pub async fn get_by_id(pool: &sqlx::MySqlPool, role_id: u32) -> Result<Role, sqlx::Error> {
        sqlx::query_as!(
            Role,
            r#"
            SELECT id, name, description, is_admin as "is_admin: bool", created_at as "created_at!", updated_at as "updated_at!"
            FROM roles
            WHERE id = ?
            "#,
            role_id
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &sqlx::MySqlPool,
        role_id: u32,
        request: &UpdateRoleRequest,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE roles 
            SET name = ?, description = ?, is_admin = ?
            WHERE id = ?
            "#,
            request.name,
            request.description,
            request.is_admin,
            role_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &sqlx::MySqlPool, role_id: u32) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM roles WHERE id = ?"#, role_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_roles_by_user_id(
        pool: &sqlx::MySqlPool,
        user_id: u32,
    ) -> Result<Vec<Role>, sqlx::Error> {
        sqlx::query_as!(
            Role,
            r#"
            SELECT r.id, r.name, r.description, r.is_admin as "is_admin: bool", r.created_at as "created_at!", r.updated_at as "updated_at!"
            FROM roles r
            JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = ?
            "#,
            user_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn assign_roles_to_user(
        pool: &sqlx::MySqlPool,
        assignment: &UserRoleAssignment,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Remove existing roles
        sqlx::query!(
            r#"DELETE FROM user_roles WHERE user_id = ?"#,
            assignment.user_id
        )
        .execute(&mut *tx)
        .await?;

        // Add new roles
        for role_id in &assignment.role_ids {
            sqlx::query!(
                r#"
                INSERT INTO user_roles (user_id, role_id)
                VALUES (?, ?)
                "#,
                assignment.user_id,
                role_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn update_page_permissions(
        pool: &sqlx::MySqlPool,
        page_id: u32,
        permissions: &Vec<crate::models::custom_page::RolePermissionRequest>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Remove existing permissions
        sqlx::query!(r#"DELETE FROM page_permissions WHERE page_id = ?"#, page_id)
            .execute(&mut *tx)
            .await?;

        // Add new permissions
        for permission in permissions {
            sqlx::query!(
                r#"
                INSERT INTO page_permissions (
                    page_id, role_id, can_view, can_create, can_edit, 
                    can_delete, can_manage_fields
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

        Ok(())
    }
}
