-- First, get all current holiday roles from the old system
-- If two roles are both holiday roles, they should interfere with each other in the new system
INSERT INTO role_holiday_groups (role_id, interferes_with_role_id)
SELECT r1.id, r2.id
FROM roles r1
JOIN roles r2 ON r1.id != r2.id
WHERE r1.is_holiday_role = true AND r2.is_holiday_role = true
AND NOT EXISTS (
    -- Check that this relationship doesn't already exist
    SELECT 1 FROM role_holiday_groups
    WHERE role_id = r1.id AND interferes_with_role_id = r2.id
);

-- Log the migration in the migration history
-- We don't remove the is_holiday_role flag yet for backward compatibility
-- but we'll gradually phase it out 