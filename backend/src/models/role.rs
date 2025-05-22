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
    pub interfering_role_ids: Option<Vec<u32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_admin: bool,
    pub interfering_role_ids: Option<Vec<u32>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRoleAssignment {
    pub user_id: u32,
    pub role_ids: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleWithInterferingRoles {
    #[serde(flatten)]
    pub role: Role,
    pub interfering_role_ids: Vec<u32>,
}

impl Role {
    pub async fn create(
        pool: &sqlx::MySqlPool,
        request: &CreateRoleRequest,
    ) -> Result<u32, sqlx::Error> {
        let mut tx = pool.begin().await?;
        
        let result = sqlx::query!(
            r#"
            INSERT INTO roles (name, description, is_admin)
            VALUES (?, ?, ?)
            "#,
            request.name,
            request.description,
            request.is_admin
        )
        .execute(&mut *tx)
        .await?;

        let role_id = result.last_insert_id() as u32;
        
        if let Some(interfering_ids) = &request.interfering_role_ids {
            for interfering_id in interfering_ids {
                sqlx::query!(
                    r#"
                    INSERT INTO role_holiday_groups (role_id, interferes_with_role_id)
                    VALUES (?, ?)
                    "#,
                    role_id,
                    interfering_id
                )
                .execute(&mut *tx)
                .await?;
                
                sqlx::query!(
                    r#"
                    INSERT INTO role_holiday_groups (role_id, interferes_with_role_id)
                    VALUES (?, ?)
                    "#,
                    interfering_id,
                    role_id
                )
                .execute(&mut *tx)
                .await?;
            }
        }
        
        tx.commit().await?;
        Ok(role_id)
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

    pub async fn get_by_id_with_interfering_roles(pool: &sqlx::MySqlPool, role_id: u32) -> Result<RoleWithInterferingRoles, sqlx::Error> {
        let role = Self::get_by_id(pool, role_id).await?;
        
        let interfering_role_ids = sqlx::query_scalar!(
            r#"
            SELECT interferes_with_role_id 
            FROM role_holiday_groups
            WHERE role_id = ?
            "#,
            role_id
        )
        .fetch_all(pool)
        .await?;
        
        Ok(RoleWithInterferingRoles {
            role,
            interfering_role_ids,
        })
    }

    pub async fn update(
        pool: &sqlx::MySqlPool,
        role_id: u32,
        request: &UpdateRoleRequest,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        
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
        .execute(&mut *tx)
        .await?;
        
        if let Some(interfering_ids) = &request.interfering_role_ids {
            sqlx::query!("DELETE FROM role_holiday_groups WHERE role_id = ?", role_id)
                .execute(&mut *tx)
                .await?;
            
            sqlx::query!("DELETE FROM role_holiday_groups WHERE interferes_with_role_id = ?", role_id)
                .execute(&mut *tx)
                .await?;
            
            for interfering_id in interfering_ids {
                if *interfering_id != role_id {
                    sqlx::query!(
                        r#"
                        INSERT INTO role_holiday_groups (role_id, interferes_with_role_id)
                        VALUES (?, ?)
                        "#,
                        role_id,
                        interfering_id
                    )
                    .execute(&mut *tx)
                    .await?;
                    
                    sqlx::query!(
                        r#"
                        INSERT INTO role_holiday_groups (role_id, interferes_with_role_id)
                        VALUES (?, ?)
                        "#,
                        interfering_id,
                        role_id
                    )
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }
        
        tx.commit().await?;
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

        sqlx::query!(
            r#"DELETE FROM user_roles WHERE user_id = ?"#,
            assignment.user_id
        )
        .execute(&mut *tx)
        .await?;

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

        sqlx::query!(r#"DELETE FROM page_permissions WHERE page_id = ?"#, page_id)
            .execute(&mut *tx)
            .await?;

        for permission in permissions {
            sqlx::query!(
                r#"
                INSERT INTO page_permissions (
                    page_id, role_id, can_view, can_create, can_edit,
                    can_delete, can_manage_fields, can_view_acknowledgments, can_add
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                page_id,
                permission.role_id,
                permission.can_view,
                permission.can_create,
                permission.can_edit,
                permission.can_delete,
                permission.can_manage_fields,
                permission.can_view_acknowledgments,
                permission.can_add
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_user_ids_by_role_id(
        pool: &sqlx::MySqlPool,
        role_id: u32,
    ) -> Result<Vec<u32>, sqlx::Error> {
        let user_ids = sqlx::query_scalar!(
            r#"
            SELECT user_id FROM user_roles WHERE role_id = ?
            "#,
            role_id
        )
        .fetch_all(pool)
        .await?;
        Ok(user_ids)
    }

    pub async fn get_colleague_user_ids_in_interfering_roles(
        pool: &sqlx::MySqlPool,
        user_id: u32,
    ) -> Result<Vec<u32>, sqlx::Error> {
        let user_role_ids = sqlx::query_scalar!(
            r#"
            SELECT role_id 
            FROM user_roles 
            WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;
        
        if user_role_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let role_id_placeholders = user_role_ids
            .iter()
            .map(|_: &u32| "?")
            .collect::<Vec<_>>()
            .join(",");
            
        let query_str = format!(
            r#"
            SELECT DISTINCT interferes_with_role_id
            FROM role_holiday_groups
            WHERE role_id IN ({})
            "#,
            role_id_placeholders
        );
        
        let mut query_builder = sqlx::query_scalar::<_, u32>(&query_str);
        for role_id in &user_role_ids {
            query_builder = query_builder.bind(*role_id);
        }
        
        let interfering_role_ids = query_builder.fetch_all(pool).await?;
        
        if interfering_role_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let interfering_role_placeholders = interfering_role_ids
            .iter()
            .map(|_: &u32| "?")
            .collect::<Vec<_>>()
            .join(",");
            
        let query_str = format!(
            r#"
            SELECT DISTINCT user_id
            FROM user_roles
            WHERE role_id IN ({}) AND user_id != ?
            "#,
            interfering_role_placeholders
        );
        
        let mut query_builder = sqlx::query_scalar::<_, u32>(&query_str);
        for role_id in &interfering_role_ids {
            query_builder = query_builder.bind(*role_id);
        }
        query_builder = query_builder.bind(user_id);
        
        let colleague_ids = query_builder.fetch_all(pool).await?;
        
        Ok(colleague_ids)
    }

    pub async fn get_colleague_user_ids_in_shared_holiday_roles(
        pool: &sqlx::MySqlPool,
        user_id: u32,
    ) -> Result<Vec<u32>, sqlx::Error> {
        Self::get_colleague_user_ids_in_interfering_roles(pool, user_id).await
    }
    
    pub async fn get_all_with_interfering_roles(pool: &sqlx::MySqlPool) -> Result<Vec<RoleWithInterferingRoles>, sqlx::Error> {
        let roles = Self::get_all(pool).await?;
        
        let mut result = Vec::with_capacity(roles.len());
        
        for role in roles {
            let interfering_role_ids = sqlx::query_scalar!(
                r#"
                SELECT interferes_with_role_id 
                FROM role_holiday_groups
                WHERE role_id = ?
                "#,
                role.id
            )
            .fetch_all(pool)
            .await?;
            
            result.push(RoleWithInterferingRoles {
                role,
                interfering_role_ids,
            });
        }
        
        Ok(result)
    }
}
