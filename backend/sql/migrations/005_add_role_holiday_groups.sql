-- Create a new table for role vacation grouping
CREATE TABLE IF NOT EXISTS role_holiday_groups (
    role_id INT UNSIGNED NOT NULL,
    interferes_with_role_id INT UNSIGNED NOT NULL,
    PRIMARY KEY (role_id, interferes_with_role_id),
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE,
    FOREIGN KEY (interferes_with_role_id) REFERENCES roles (id) ON DELETE CASCADE
);

-- Add a comment to explain the purpose
ALTER TABLE role_holiday_groups COMMENT 'Many-to-many relationship between roles that interfere with each other for vacation scheduling.';

-- Add index on interferes_with_role_id for faster lookups
CREATE INDEX idx_role_holiday_groups_interferes_with ON role_holiday_groups (interferes_with_role_id);

-- Add migration to eventually deprecate is_holiday_role field, but keep it for now for backward compatibility
-- ALTER TABLE roles ADD COLUMN has_vacation_conflicts BOOLEAN NOT NULL DEFAULT false COMMENT 'True if this role considers vacation conflicts.'; 