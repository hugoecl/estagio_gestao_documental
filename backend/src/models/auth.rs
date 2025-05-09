use crate::models::{
    custom_page::{PagePermission, UserPagePermissions},
    role::Role,
};
use sqlx::MySqlPool;

// ... (validate_session, is_admin, user_can_... functions) ...

pub async fn calculate_user_page_permissions(
    pool: &MySqlPool,
    user_id: i32,
    page_permissions: &[PagePermission], // Permissions specific to the page
) -> Result<UserPagePermissions, sqlx::Error> {
    let user_roles = Role::get_roles_by_user_id(pool, user_id as u32).await?;

    let mut calculated_perms = UserPagePermissions::default(); // Start with no permissions

    for role in user_roles {
        if role.is_admin {
            // Admin gets all permissions
            calculated_perms = UserPagePermissions {
                can_view: true,
                can_create: true,
                can_edit: true,
                can_delete: true,
                can_manage_fields: true,
                can_view_acknowledgments: true, // Admin can view acknowledgments
                is_admin: true,
            };
            break; // No need to check other roles if admin
        }

        // Find permissions for this specific role on this page
        if let Some(perm) = page_permissions.iter().find(|p| p.role_id == role.id) {
            calculated_perms.can_view |= perm.can_view;
            calculated_perms.can_create |= perm.can_create;
            calculated_perms.can_edit |= perm.can_edit;
            calculated_perms.can_delete |= perm.can_delete;
            calculated_perms.can_manage_fields |= perm.can_manage_fields;
            calculated_perms.can_view_acknowledgments |= perm.can_view_acknowledgments;
        }
    }

    Ok(calculated_perms)
}
