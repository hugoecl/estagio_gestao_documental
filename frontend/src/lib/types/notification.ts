// Corresponds to the NotificationResponse struct in the backend
export interface NotificationResponse {
  id: number;
  userId: number;
  recordId?: number | null; // Can be null for broadcasts
  vacationRequestId?: number | null; // Added for vacation request notifications
  pageId?: number | null;   // Can be null for broadcasts
  fieldId?: number | null; // Optional number or null
  notificationType: string;
  message: string;
  dueDate?: string | null; // Date string (e.g., "YYYY-MM-DD") or null
  isRead: boolean;
  createdAt: string; // ISO 8601 date string
  pagePath?: string | null; // Optional page path
  pageName?: string | null; // Optional page name
  // record_snippet?: string | null; // Example if added later
}
