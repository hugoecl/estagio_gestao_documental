CREATE TABLE IF NOT EXISTS users (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password BINARY(48) NOT NULL,
    vacation_days_current_year SMALLINT UNSIGNED DEFAULT 0 COMMENT 'Remaining vacation days for the current year',
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
    name VARCHAR(255) NOT NULL COMMENT 'Display name of the page or group',
    path VARCHAR(255) NOT NULL UNIQUE COMMENT 'URL path for the page, or base path for group children',
    parent_path VARCHAR(255) COMMENT 'Parent path for nested navigation',
    is_group BOOLEAN NOT NULL DEFAULT false COMMENT 'True if this entry is just a menu group/folder/submenu',
    description TEXT,
    icon VARCHAR(20) COMMENT 'FontAwesome icon name',
    icon_type VARCHAR(20) DEFAULT 'fontawesome' COMMENT 'Type of icon: fontawesome or image',
    icon_image_path VARCHAR(255) DEFAULT NULL COMMENT 'Path to uploaded image icon',
    notify_on_new_record BOOLEAN NOT NULL DEFAULT false COMMENT 'Notify users with access when a new record is created',
    requires_acknowledgment BOOLEAN NOT NULL DEFAULT false COMMENT 'Require users to acknowledge records before viewing details',
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
    notification_enabled BOOLEAN NOT NULL DEFAULT false COMMENT 'If notifications are enabled for this field',
    notification_days_before INT UNSIGNED DEFAULT NULL COMMENT 'How many days before the target date to notify',
    notification_target_date_part VARCHAR(10) DEFAULT NULL COMMENT 'Which part of the date field triggers notification (e.g., start_date, end_date)',
    PRIMARY KEY (id),
    FOREIGN KEY (page_id) REFERENCES custom_pages (id) ON DELETE CASCADE,
    FOREIGN KEY (field_type_id) REFERENCES field_types (id)
    -- No ON DELETE CASCADE for field_type_id as field_types are static definitions
);

-- Roles Table
CREATE TABLE IF NOT EXISTS roles (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    is_holiday_role BOOLEAN NOT NULL DEFAULT false COMMENT 'True if this role is relevant for vacation scheduling conflicts',
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
    can_view_acknowledgments BOOLEAN NOT NULL DEFAULT false,
    can_add BOOLEAN NOT NULL DEFAULT false,
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
    FOREIGN KEY (page_id) REFERENCES custom_pages (id) ON DELETE CASCADE,
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

-- Record Acknowledgments Table
CREATE TABLE IF NOT EXISTS record_acknowledgments (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    user_id INT UNSIGNED NOT NULL,
    record_id INT UNSIGNED NOT NULL,
    acknowledged_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY unique_user_record_acknowledgment (user_id, record_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (record_id) REFERENCES page_records (id) ON DELETE CASCADE
);

-- Notifications Table
CREATE TABLE IF NOT EXISTS notifications (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    user_id INT UNSIGNED NOT NULL COMMENT 'The user receiving the notification',
    record_id INT UNSIGNED DEFAULT NULL COMMENT 'The record the notification relates to, NULL if general broadcast',
    page_id INT UNSIGNED DEFAULT NULL COMMENT 'The page the record belongs to (denormalized for easier linking), NULL if general broadcast',
    field_id INT UNSIGNED NULL COMMENT 'The specific field triggering the notification (if applicable)',
    notification_type VARCHAR(50) NOT NULL COMMENT 'Type of notification (e.g., DATE_EXPIRY, SIGNATURE_REQUIRED, ADMIN_BROADCAST)',
    message TEXT NOT NULL COMMENT 'Notification message content',
    due_date DATE NULL COMMENT 'The relevant date from the record (e.g., expiry date)',
    is_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (record_id) REFERENCES page_records (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (page_id) REFERENCES custom_pages (id) ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY (field_id) REFERENCES page_fields (id) ON DELETE SET NULL ON UPDATE CASCADE,
    INDEX idx_user_unread (user_id, is_read)
);

-- Vacation Requests Table
CREATE TABLE IF NOT EXISTS vacation_requests (
    id INT UNSIGNED AUTO_INCREMENT UNIQUE NOT NULL,
    user_id INT UNSIGNED NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    status ENUM('PENDING', 'APPROVED', 'REJECTED') NOT NULL DEFAULT 'PENDING',
    notes TEXT DEFAULT NULL COMMENT 'User notes on request, or admin notes on action',
    requested_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    approved_by INT UNSIGNED DEFAULT NULL COMMENT 'Admin user ID who actioned the request',
    actioned_at TIMESTAMP NULL DEFAULT NULL COMMENT 'Timestamp of approval/rejection',
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (approved_by) REFERENCES users(id) ON DELETE SET NULL,
    INDEX idx_vacation_user_status (user_id, status),
    INDEX idx_vacation_dates (start_date, end_date)
);
