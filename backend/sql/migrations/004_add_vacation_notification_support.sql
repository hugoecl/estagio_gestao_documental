-- Create a new column for vacation_request_id
ALTER TABLE notifications 
ADD COLUMN vacation_request_id INT UNSIGNED DEFAULT NULL COMMENT 'ID of a vacation request (if this is a vacation-related notification)' AFTER record_id;

-- Add an index to improve query performance
CREATE INDEX idx_vacation_request_id ON notifications(vacation_request_id);

-- Add foreign key reference to vacation_requests table
ALTER TABLE notifications 
ADD CONSTRAINT fk_notification_vacation_request 
FOREIGN KEY (vacation_request_id) REFERENCES vacation_requests(id) ON DELETE CASCADE; 