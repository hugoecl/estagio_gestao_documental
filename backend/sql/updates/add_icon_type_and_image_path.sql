-- Add icon_type and icon_image_path columns to custom_pages table
ALTER TABLE custom_pages
ADD COLUMN icon_type VARCHAR(20) DEFAULT 'fontawesome' COMMENT 'Type of icon: fontawesome or image',
ADD COLUMN icon_image_path VARCHAR(255) DEFAULT NULL COMMENT 'Path to uploaded image icon';
 
-- Update existing records to set icon_type explicitly for existing pages with icons
UPDATE custom_pages
SET icon_type = 'fontawesome'
WHERE icon IS NOT NULL AND icon != ''; 