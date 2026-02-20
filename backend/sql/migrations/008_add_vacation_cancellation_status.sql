-- Add CANCELLATION_REQUESTED and CANCELLED to vacation_requests status.
-- CANCELLATION_REQUESTED: user requested to cancel approved vacation, awaiting admin.
-- CANCELLED: admin approved the cancellation request.

ALTER TABLE vacation_requests
MODIFY COLUMN status ENUM(
    'PENDING',
    'APPROVED',
    'REJECTED',
    'CANCELLATION_REQUESTED',
    'CANCELLED'
) NOT NULL DEFAULT 'PENDING';
