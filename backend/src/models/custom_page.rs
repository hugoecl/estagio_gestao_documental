use ahash::{HashMap, HashMapExt};

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
    pub is_group: bool, // Added field
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
    pub is_group: bool, // Added field
    pub description: Option<String>,
    pub icon: Option<String>,
    // Fields and permissions might be empty if is_group is true
    pub fields: Vec<CreatePageFieldRequest>,
    pub permissions: Vec<RolePermissionRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCustomPageRequest {
    pub name: String,
    pub parent_path: Option<String>, // Allow changing parent
    pub description: Option<String>,
    pub icon: Option<String>,
    // is_group is generally not updatable after creation easily
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserPagePermissions {
    pub can_view: bool,
    pub can_create: bool,
    pub can_edit: bool,
    pub can_delete: bool,
    pub can_manage_fields: bool,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomPageWithFields {
    pub page: CustomPage,
    // Fields and permissions might be empty if it's a group
    pub fields: Vec<PageField>,
    pub permissions: Vec<PagePermission>,
    #[serde(rename = "currentUserPermissions")]
    pub current_user_permissions: Option<UserPagePermissions>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PagePermission {
    pub id: u32,
    pub page_id: u32,
    pub role_id: u32,
    pub role_name: String,
    pub can_view: bool,
    pub can_create: bool,
    pub can_edit: bool,
    pub can_delete: bool,
    pub can_manage_fields: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NavigationItem {
    pub title: String,
    // Path is None for groups, Some(path) for actual pages
    pub path: Option<String>,
    // parent_db_path stores the path used for linking children, even for groups
    #[serde(skip_serializing)] // Don't send this internal field to frontend
    pub parent_db_path: Option<String>,
    #[serde(skip_serializing)] // Don't send this internal field to frontend
    pub db_path: String, // Store the actual path from DB for tree building
    pub icon: Option<String>,
    pub children: Vec<NavigationItem>,
}

impl CustomPage {
    pub async fn create(
        pool: &sqlx::MySqlPool,
        request: &CreateCustomPageRequest,
    ) -> Result<u32, sqlx::Error> {
        // --- Path Cleaning ---
        let mut cleaned_path = request.path.clone();
        if cleaned_path.len() > 1 && cleaned_path.ends_with('/') {
            cleaned_path.pop(); // Remove trailing slash unless it's just "/"
        }
        let cleaned_parent_path = request.parent_path.as_ref().map(|p| {
            let mut parent = p.clone();
            if parent.len() > 1 && parent.ends_with('/') {
                parent.pop();
            }
            parent
        });
        // --- End Path Cleaning ---

        let mut tx = pool.begin().await?;

        // Insert the page/group
        let result = sqlx::query!(
            r#"
            INSERT INTO custom_pages (name, path, parent_path, is_group, description, icon)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            request.name,
            cleaned_path,        // Use cleaned path
            cleaned_parent_path, // Use cleaned parent path
            request.is_group,
            request.description,
            request.icon
        )
        .execute(&mut *tx)
        .await?;

        let page_id = result.last_insert_id() as u32;

        // Only insert fields and permissions if it's NOT a group
        if !request.is_group {
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
        }

        tx.commit().await?;

        Ok(page_id)
    }

    pub async fn get_all(pool: &sqlx::MySqlPool) -> Result<Vec<CustomPage>, sqlx::Error> {
        sqlx::query_as!(
            CustomPage,
            r#"
            SELECT
                id, name, path, parent_path, is_group as "is_group: bool", description,
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
        // user_id: i32, // Pass user_id to check permissions
    ) -> Result<CustomPageWithFields, sqlx::Error> {
        let page = sqlx::query_as!(
            CustomPage,
            r#"
            SELECT
                id, name, path, parent_path, is_group as "is_group: bool", description,
                icon, created_at as "created_at!", updated_at as "updated_at!"
            FROM custom_pages
            WHERE id = ?
            "#,
            page_id
        )
        .fetch_one(pool)
        .await?;

        let mut fields = Vec::new();
        let mut permissions = Vec::new();
        // let mut current_user_permissions = None; // Initialize

        // Only fetch fields and permissions if it's not a group
        if !page.is_group {
            fields = PageField::get_by_page_id(pool, page_id).await?;

            permissions = sqlx::query_as!(
                PagePermission,
                r#"
                SELECT
                    p.id, p.page_id, p.role_id, r.name as role_name,
                    p.can_view as "can_view: bool", p.can_create as "can_create: bool",
                    p.can_edit as "can_edit: bool", p.can_delete as "can_delete: bool",
                    p.can_manage_fields as "can_manage_fields: bool"
                FROM page_permissions p
                JOIN roles r ON p.role_id = r.id
                WHERE p.page_id = ?
                "#,
                page_id
            )
            .fetch_all(pool)
            .await?;

            // Fetch current user's permissions for this specific page
            // This requires the user_id. You'll need to pass it into this function.
            // current_user_permissions = Some(Self::get_user_permissions_for_page(pool, user_id, page_id).await?);
        }

        Ok(CustomPageWithFields {
            page,
            fields,
            permissions,
            current_user_permissions: None, // TODO: Populate this based on user_id
        })
    }

    // TODO: Add a function like this and call it from get_by_id and get_navigation_menu
    // pub async fn get_user_permissions_for_page(pool: &sqlx::MySqlPool, user_id: i32, page_id: u32) -> Result<UserPagePermissions, sqlx::Error> {
    //     let perms = sqlx::query!(
    //         r#"
    //         SELECT
    //             MAX(r.is_admin) as is_admin,
    //             MAX(pp.can_view) as can_view,
    //             MAX(pp.can_create) as can_create,
    //             MAX(pp.can_edit) as can_edit,
    //             MAX(pp.can_delete) as can_delete,
    //             MAX(pp.can_manage_fields) as can_manage_fields
    //         FROM user_roles ur
    //         LEFT JOIN roles r ON r.id = ur.role_id
    //         LEFT JOIN page_permissions pp ON pp.role_id = ur.role_id AND pp.page_id = ?
    //         WHERE ur.user_id = ?
    //         "#,
    //         page_id,
    //         user_id
    //     )
    //     .fetch_one(pool)
    //     .await?;

    //     Ok(UserPagePermissions {
    //         is_admin: perms.is_admin.unwrap_or(0) == 1,
    //         can_view: perms.can_view.unwrap_or(0) == 1,
    //         can_create: perms.can_create.unwrap_or(0) == 1,
    //         can_edit: perms.can_edit.unwrap_or(0) == 1,
    //         can_delete: perms.can_delete.unwrap_or(0) == 1,
    //         can_manage_fields: perms.can_manage_fields.unwrap_or(0) == 1,
    //     })
    // }

    pub async fn update(
        pool: &sqlx::MySqlPool,
        page_id: u32,
        request: &UpdateCustomPageRequest,
    ) -> Result<(), sqlx::Error> {
        // --- Path Cleaning for parent_path ---
        let cleaned_parent_path = request.parent_path.as_ref().map(|p| {
            let mut parent = p.clone();
            if parent.len() > 1 && parent.ends_with('/') {
                parent.pop();
            }
            parent
        });
        // --- End Path Cleaning ---

        sqlx::query!(
            r#"
            UPDATE custom_pages
            SET name = ?, description = ?, icon = ?, parent_path = ?
            WHERE id = ?
            "#,
            request.name,
            request.description,
            request.icon,
            cleaned_parent_path, // Use cleaned parent path
            page_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(pool: &sqlx::MySqlPool, page_id: u32) -> Result<(), sqlx::Error> {
        // Consider deleting children recursively if needed, or preventing deletion if children exist.
        // For now, just deletes the single entry.
        sqlx::query!(r#"DELETE FROM custom_pages WHERE id = ?"#, page_id)
            .execute(pool)
            .await?;

        Ok(())
    }

    // Updated get_navigation_menu
    pub async fn get_navigation_menu(
        pool: &sqlx::MySqlPool,
        // user_id: i32, // Pass user_id to filter based on view permissions
    ) -> Result<Vec<NavigationItem>, sqlx::Error> {
        // Fetch all pages and groups the user can potentially see
        // TODO: Filter this initial query based on user_id and page_permissions.can_view = true OR roles.is_admin = true
        let all_items_raw = sqlx::query_as!(
            CustomPage,
            r#"
            SELECT
                id, name, path, parent_path, is_group as "is_group: bool", description,
                icon, created_at as "created_at!", updated_at as "updated_at!"
            FROM custom_pages
            ORDER BY parent_path, name
            "# // Add WHERE clause here to filter by user permissions
        )
        .fetch_all(pool)
        .await?;

        // Use HashMap to group items by their parent_path (String key)
        let mut items_by_parent: HashMap<Option<String>, Vec<NavigationItem>> = HashMap::new();

        for item_raw in all_items_raw {
            let nav_item = NavigationItem {
                title: item_raw.name.clone(),
                // Path is Some only if it's NOT a group
                path: if item_raw.is_group {
                    None
                } else {
                    Some(item_raw.path.clone())
                },
                parent_db_path: item_raw.parent_path.clone(), // Store parent path from DB
                db_path: item_raw.path.clone(),               // Store actual path from DB
                icon: item_raw.icon.clone(),
                children: Vec::new(),
            };

            // Group by parent_path. Use None key for root items.
            items_by_parent
                .entry(item_raw.parent_path) // Key is Option<String>
                .or_default()
                .push(nav_item);
        }

        // Recursive function to build the tree
        fn build_tree(
            parent_db_path: Option<&str>, // Use Option<&str> for keying into HashMap
            items_by_parent: &mut HashMap<Option<String>, Vec<NavigationItem>>,
        ) -> Vec<NavigationItem> {
            // Get children for the current parent path (key needs to be Option<String>)
            let parent_key = parent_db_path.map(String::from);
            if let Some(mut children) = items_by_parent.remove(&parent_key) {
                for child in &mut children {
                    // Recursively find children for this child, using its db_path
                    child.children = build_tree(Some(&child.db_path), items_by_parent);
                }
                children // Return the processed children
            } else {
                Vec::new() // No children found for this parent
            }
        }

        // Start building the tree from the root (parent_path is None)
        let root_items = build_tree(None, &mut items_by_parent);

        Ok(root_items)
    }
}
