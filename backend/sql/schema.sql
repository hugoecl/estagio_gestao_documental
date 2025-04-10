CREATE TABLE IF NOT EXISTS users (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password BINARY(48) NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS user_page_analytics (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    user_id INT UNSIGNED NOT NULL,
    page_path VARCHAR(255) NOT NULL,
    visit_count INT UNSIGNED DEFAULT 1 NOT NULL,
    last_visited_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY unique_user_page (user_id, page_path),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- Custom Pages Table
CREATE TABLE IF NOT EXISTS custom_pages (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL COMMENT 'Display name of the page',
    path VARCHAR(255) NOT NULL UNIQUE COMMENT 'URL path for the page',
    parent_path VARCHAR(255) COMMENT 'Parent path for nested navigation',
    description TEXT,
    icon VARCHAR(20) COMMENT 'FontAwesome icon name',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

-- Field Types Table
CREATE TABLE IF NOT EXISTS field_types (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    name VARCHAR(50) NOT NULL UNIQUE,
    PRIMARY KEY (id)
);

-- Insert common field types
INSERT IGNORE INTO field_types (name)
VALUES
    ('TEXT'),
    ('NUMBER'),
    ('SELECT'),
    ('DATE'),
    ('DATE_RANGE'),
    ('TEXTAREA');

-- Page Fields Table
CREATE TABLE IF NOT EXISTS page_fields (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    page_id INT UNSIGNED NOT NULL,
    name VARCHAR(100) NOT NULL COMMENT 'Field name/ID',
    display_name VARCHAR(255) NOT NULL COMMENT 'Human-readable field name',
    field_type_id INT UNSIGNED NOT NULL,
    required BOOLEAN NOT NULL DEFAULT false,
    options JSON COMMENT 'Options for SELECT fields',
    validation_name VARCHAR(100) COMMENT 'Name of validation function',
    is_searchable BOOLEAN NOT NULL DEFAULT true,
    is_displayed_in_table BOOLEAN NOT NULL DEFAULT true,
    order_index INT UNSIGNED NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    FOREIGN KEY (page_id) REFERENCES custom_pages (id) ON DELETE CASCADE,
    FOREIGN KEY (field_type_id) REFERENCES field_types (id)
);

-- Roles Table
CREATE TABLE IF NOT EXISTS roles (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

-- -- Insert default admin role
-- INSERT IGNORE INTO roles (name, description, is_admin) VALUES
-- ('Admin', 'Administrador com acesso completo', true);
-- User Roles Table (Many-to-many relationship)
CREATE TABLE IF NOT EXISTS user_roles (
    user_id INT UNSIGNED NOT NULL,
    role_id INT UNSIGNED NOT NULL,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
);

-- Page Permissions Table
CREATE TABLE IF NOT EXISTS page_permissions (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    page_id INT UNSIGNED NOT NULL,
    role_id INT UNSIGNED NOT NULL,
    can_view BOOLEAN NOT NULL DEFAULT false,
    can_create BOOLEAN NOT NULL DEFAULT false,
    can_edit BOOLEAN NOT NULL DEFAULT false,
    can_delete BOOLEAN NOT NULL DEFAULT false,
    can_manage_fields BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (id),
    UNIQUE KEY unique_page_role (page_id, role_id),
    FOREIGN KEY (page_id) REFERENCES custom_pages (id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
);

-- Page Records Table (for storing dynamic data)
CREATE TABLE IF NOT EXISTS page_records (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    page_id INT UNSIGNED NOT NULL,
    data JSON NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    created_by INT UNSIGNED NOT NULL,
    updated_by INT UNSIGNED,
    PRIMARY KEY (id),
    FOREIGN KEY (page_id) REFERENCES custom_pages (id),
    FOREIGN KEY (created_by) REFERENCES users (id),
    FOREIGN KEY (updated_by) REFERENCES users (id)
);

-- Page Record Files Table
CREATE TABLE IF NOT EXISTS page_record_files (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    record_id INT UNSIGNED NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(255) NOT NULL,
    uploaded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    uploaded_by INT UNSIGNED NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (record_id) REFERENCES page_records (id) ON DELETE CASCADE,
    FOREIGN KEY (uploaded_by) REFERENCES users (id)
);
