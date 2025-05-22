-- Add CHECKBOX field type
INSERT IGNORE INTO field_types (name)
VALUES ('CHECKBOX');

-- Update version
UPDATE schema_migrations SET version = '20240731_add_checkbox_field_type' WHERE id = 1;


