export enum VacationRequestStatus {
  Pending = "PENDING",
  Approved = "APPROVED",
  Rejected = "REJECTED",
  CancellationRequested = "CANCELLATION_REQUESTED",
  Cancelled = "CANCELLED",
}

export interface VacationRequest {
  id: number;
  user_id: number;
  start_date: string; // Keep as YYYY-MM-DD string for easier API interaction initially
  end_date: string;   // Keep as YYYY-MM-DD string
  status: VacationRequestStatus;
  notes: string | null;
  requested_at: string; // ISO 8601 string
  approved_by: number | null;
  actioned_at: string | null; // ISO 8601 string
}

export interface CreateVacationRequestPayload {
  start_date: string; // YYYY-MM-DD
  end_date: string;   // YYYY-MM-DD
  notes?: string | null;
}

export interface RemainingVacationDaysResponse {
  total_allocated_days: number; // Changed to number to match u16/u32 potential
  approved_days_taken: number; // Changed to number to match i64
  pending_days_requested: number; // Changed to number to match i64
  remaining_days: number; // Changed to number to match i64
}

// Additional type for displaying in calendar/list with processed dates
export interface VacationRequestDisplay extends VacationRequest {
  startDateDisplay: string; // DD/MM/YYYY
  endDateDisplay: string;   // DD/MM/YYYY
  requestedAtDisplay: string; // DD/MM/YYYY HH:MM
  actionedAtDisplay?: string; // DD/MM/YYYY HH:MM or empty
  duration: number; // in days
}

// For admin view, includes user details
export interface VacationRequestWithUser extends VacationRequest {
  username: string;
  email: string;
  // Potentially add user's total vacation days or role names if helpful for admin view
}