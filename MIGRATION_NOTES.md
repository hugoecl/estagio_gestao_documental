# Migration Notes - Replacing Special Vacation Roles with Role-Based Grouping

This document explains the changes made to simplify the vacation management system by replacing the special "vacation roles" with a more flexible role-based grouping approach.

## Overview

Previously, roles had a boolean flag `is_holiday_role` that marked them as special vacation roles. Users in the same vacation role would have their vacation requests interfere with each other. This system was rigid and complicated.

The new system:
1. Uses a many-to-many relationship between roles to define which roles interfere with each other
2. Allows administrators to specify which roles interfere with each other when creating or editing roles
3. Provides backward compatibility with the existing system

## Database Changes

New table:
```sql
CREATE TABLE IF NOT EXISTS role_holiday_groups (
    role_id INT UNSIGNED NOT NULL,
    interferes_with_role_id INT UNSIGNED NOT NULL,
    PRIMARY KEY (role_id, interferes_with_role_id),
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE,
    FOREIGN KEY (interferes_with_role_id) REFERENCES roles (id) ON DELETE CASCADE
);
```

Migration script to populate new table with existing holiday roles:
```sql
INSERT INTO role_holiday_groups (role_id, interferes_with_role_id)
SELECT r1.id, r2.id
FROM roles r1
JOIN roles r2 ON r1.id != r2.id
WHERE r1.is_holiday_role = true AND r2.is_holiday_role = true
AND NOT EXISTS (
    SELECT 1 FROM role_holiday_groups
    WHERE role_id = r1.id AND interferes_with_role_id = r2.id
);
```

## Backend Changes

1. Updated the `Role` model:
   - Added method `get_colleague_user_ids_in_interfering_roles` - Finds users in roles that interfere with the current user's roles
   - Added method `get_by_id_with_interfering_roles` - Gets a role with its interfering roles
   - Modified `get_colleague_user_ids_in_shared_holiday_roles` to try the new system first, then fall back to the old system

2. New API endpoints:
   - `GET /roles/with-interfering` - Get all roles with their interfering roles
   - `GET /roles/{id}/with-interfering` - Get a specific role with its interfering roles

3. Updated role creation/editing:
   - Added support for `interfering_role_ids` in `CreateRoleRequest` and `UpdateRoleRequest`
   - Implemented bi-directional relationships (if role A interferes with role B, role B also interferes with role A)

## Frontend Changes

1. Updated types:
   - Added `RoleWithInterferingRoles` interface
   - Added `interfering_role_ids` to `CreateRoleRequest` and `UpdateRoleRequest`

2. Added API functions:
   - `getRolesWithInterferingRoles()` and `getRoleWithInterferingRoles()`

## Implementation Steps

To fully implement this change:

1. Run the database migrations (005_add_role_holiday_groups.sql and 006_migrate_holiday_roles.sql)
2. Deploy the backend changes
3. Update the frontend Role management page to:
   - Use the new endpoints for fetching roles with interfering roles
   - Add UI for selecting interfering roles when creating/editing roles
   - Eventually phase out the "is holiday role" checkbox

## Future Work

1. Once the new system is fully adopted, remove the `is_holiday_role` column from the `roles` table
2. Update all vacation-related code to use only the new interfering roles system
3. Consider adding a more user-friendly UI for managing role vacation groupings 