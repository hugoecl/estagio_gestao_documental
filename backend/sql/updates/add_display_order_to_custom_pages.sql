-- Add display_order column to custom_pages table
ALTER TABLE custom_pages 
ADD COLUMN display_order INT UNSIGNED DEFAULT 0 NOT NULL 
COMMENT 'Order in which to display pages in navigation';

-- Initialize display_order based on current order (by name within parent groups)
UPDATE custom_pages c1
JOIN (
    SELECT id, parent_path, 
           @row := IF(@prev = parent_path, @row + 1, 0) as row_num,
           @prev := parent_path
    FROM custom_pages, 
    (SELECT @row := 0, @prev := NULL) as r
    ORDER BY parent_path, name
) c2 ON c1.id = c2.id
SET c1.display_order = c2.row_num; 