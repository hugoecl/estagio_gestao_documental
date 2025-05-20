-- Migration to add can_add permission column to page_permissions
-- This permission allows users to add information to empty fields without editing existing data

-- Add the new column to page_permissions table
ALTER TABLE page_permissions ADD COLUMN can_add BOOLEAN NOT NULL DEFAULT FALSE;

-- Set the permission to true for all roles that already have can_edit permission
-- This ensures backward compatibility
UPDATE page_permissions SET can_add = can_edit WHERE 1=1; 