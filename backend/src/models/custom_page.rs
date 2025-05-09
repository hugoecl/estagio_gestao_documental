use ahash::{HashMap, HashMapExt, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::auth;

use super::field::PageField;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CustomPage {
    pub id: u32,
    pub name: String,
    pub path: String,
    pub parent_path: Option<String>,
    pub is_group: bool,
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
    pub notification_enabled: Option<bool>, // New field
    pub notification_days_before: Option<u32>, // New field
    pub notification_target_date_part: Option<String>, // New field
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
    pub path: Option<String>, // Null for groups
    #[serde(skip_serializing)]
    pub id: u32, // Keep track of the original ID for permission checks
    #[serde(skip_serializing)]
    pub is_group: bool, // Keep track if it's a group
    #[serde(skip_serializing)]
    pub parent_db_path: Option<String>,
    #[serde(skip_serializing)]
    pub db_path: String,
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
            for field_request in &request.fields {
                // Call PageField::create using the transaction
                // PageField::create already handles the unwrap_or(false) for notification_enabled
                PageField::create_with_tx(&mut tx, page_id, field_request).await?;
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
        user_id: i32, // Pass user_id to check permissions
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
        // current_user_permissions will be calculated later

        let can_view_this = auth::user_can_view_page(pool, user_id, page_id).await?;
        if !can_view_this {
            return Err(sqlx::Error::RowNotFound);
        }

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

            // Permissions will be calculated below
        }
        // No need to calculate permissions within the `if` block anymore

        // Calculate current_user_permissions directly based on whether it's a group
        let final_user_permissions = if !page.is_group {
             Some(Self::get_user_permissions_for_page(pool, user_id, page_id).await?)
        } else {
             let is_admin = sqlx::query_scalar!(
                 "SELECT EXISTS(SELECT 1 FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = ? AND r.is_admin = 1)",
                 user_id
             ).fetch_one(pool).await? == 1;
             Some(UserPagePermissions { is_admin, ..Default::default() })
        };


        Ok(CustomPageWithFields {
            page,
            fields,
            permissions,
            current_user_permissions: final_user_permissions,
        })
    }

    pub async fn get_user_permissions_for_page(
        pool: &sqlx::MySqlPool,
        user_id: i32,
        page_id: u32,
    ) -> Result<UserPagePermissions, sqlx::Error> {
        let perms = sqlx::query!(
            r#"
            SELECT
                MAX(CASE WHEN r.is_admin = 1 THEN 1 ELSE 0 END) as is_admin,
                MAX(CASE WHEN pp.can_view = 1 THEN 1 ELSE 0 END) as can_view,
                MAX(CASE WHEN pp.can_create = 1 THEN 1 ELSE 0 END) as can_create,
                MAX(CASE WHEN pp.can_edit = 1 THEN 1 ELSE 0 END) as can_edit,
                MAX(CASE WHEN pp.can_delete = 1 THEN 1 ELSE 0 END) as can_delete,
                MAX(CASE WHEN pp.can_manage_fields = 1 THEN 1 ELSE 0 END) as can_manage_fields
            FROM user_roles ur
            LEFT JOIN roles r ON r.id = ur.role_id
            LEFT JOIN page_permissions pp ON pp.role_id = ur.role_id AND pp.page_id = ?
            WHERE ur.user_id = ?
            "#,
            page_id,
            user_id
        )
        .fetch_one(pool)
        .await?;

        // Combine role-based permission with admin override
        let is_admin = perms.is_admin.unwrap_or(0) == 1;

        Ok(UserPagePermissions {
            is_admin,
            can_view: is_admin || perms.can_view.unwrap_or(0) == 1,
            can_create: is_admin || perms.can_create.unwrap_or(0) == 1,
            can_edit: is_admin || perms.can_edit.unwrap_or(0) == 1,
            can_delete: is_admin || perms.can_delete.unwrap_or(0) == 1,
            can_manage_fields: is_admin || perms.can_manage_fields.unwrap_or(0) == 1,
        })
    }

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
        user_id: i32,
    ) -> Result<Vec<NavigationItem>, sqlx::Error> {
        // 1. Fetch ALL pages/groups initially, ordered to help with tree construction
        let all_db_items = sqlx::query_as!(
            CustomPage,
            r#"
            SELECT
                id, name, path, parent_path, is_group as "is_group: bool", description,
                icon, created_at as "created_at!", updated_at as "updated_at!"
            FROM custom_pages
            ORDER BY parent_path IS NULL DESC, parent_path ASC, name ASC
            "# // Order by parent_path (nulls first), then by parent_path itself, then name
        )
        .fetch_all(pool)
        .await?;

        // 2. Fetch IDs of pages the user CAN view (non-groups only)
        let viewable_page_ids: HashSet<u32> = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT cp.id
            FROM custom_pages cp
            LEFT JOIN page_permissions pp ON cp.id = pp.page_id
            LEFT JOIN user_roles ur ON pp.role_id = ur.role_id AND ur.user_id = ?
            LEFT JOIN roles ro ON ur.role_id = ro.id
            WHERE
                cp.is_group = 0 AND (ro.is_admin = 1 OR pp.can_view = 1)
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .filter_map(|id_opt| Some(id_opt)) // Filter out potential NULLs from LEFT JOIN if any (though DISTINCT cp.id should handle)
        .collect();

        // 3. Check if user is admin
        let is_admin = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = ? AND r.is_admin = 1)",
            user_id
        )
        .fetch_one(pool)
        .await? == 1;

        // 4. Build a map of all items by their `path` (for groups) or `id` (for pages)
        // and a map for children lookup: parent_path -> Vec<NavigationItem>
        let mut items_by_path = HashMap::new(); // Keyed by their own path (used for parent lookup)
        let mut items_by_id = HashMap::new(); // Keyed by ID (for direct page lookup)

        for db_item in all_db_items {
            let nav_item = NavigationItem {
                id: db_item.id,
                is_group: db_item.is_group,
                title: db_item.name.clone(),
                // For pages, path is its own path. For groups, path is None in NavItem,
                // but we use db_item.path for structuring.
                path: if db_item.is_group {
                    None
                } else {
                    Some(db_item.path.clone())
                },
                parent_db_path: db_item.parent_path.clone(), // Store the DB parent_path
                db_path: db_item.path.clone(),               // Store the DB path for grouping
                icon: db_item.icon.clone(),
                children: Vec::new(),
            };
            items_by_path.insert(db_item.path.clone(), nav_item.clone());
            items_by_id.insert(db_item.id, nav_item);
        }

        // 5. Recursive function to build and filter the tree
        fn build_tree_level(
            current_parent_db_path: Option<&String>, // The DB path of the parent we are looking for children of
            all_items: &HashMap<u32, NavigationItem>, // All items by ID
            viewable_page_ids: &HashSet<u32>,
            is_admin: bool,
        ) -> Vec<NavigationItem> {
            let mut level_children: Vec<NavigationItem> = Vec::new();

            for (_id, item_template) in all_items {
                // Match parent
                if item_template.parent_db_path.as_ref() == current_parent_db_path {
                    let mut current_nav_item = item_template.clone();

                    if current_nav_item.is_group {
                        // It's a group, recursively find its children
                        let grandchildren = build_tree_level(
                            Some(&current_nav_item.db_path), // Children of this group
                            all_items,
                            viewable_page_ids,
                            is_admin,
                        );
                        // A group is visible if it's admin OR it has visible children
                        if is_admin || !grandchildren.is_empty() {
                            current_nav_item.children = grandchildren;
                            level_children.push(current_nav_item);
                        }
                    } else {
                        // It's a page, check direct view permission
                        if is_admin || viewable_page_ids.contains(&current_nav_item.id) {
                            level_children.push(current_nav_item);
                        }
                    }
                }
            }
            // Sort children at this level by title (or other criteria if needed)
            level_children.sort_by(|a, b| a.title.cmp(&b.title));
            level_children
        }

        // Start building from the root (items with parent_db_path = None)
        let root_items = build_tree_level(None, &items_by_id, &viewable_page_ids, is_admin);

        Ok(root_items)
    }
}
