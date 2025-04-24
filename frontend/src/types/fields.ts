export interface FieldType {
  id: number;
  name: string; // e.g., 'TEXT', 'NUMBER', 'SELECT', 'DATE', 'DATE_RANGE', 'TEXTAREA'
}

// Matches backend PageField
export interface PageField {
  id: number;
  page_id: number;
  name: string; // Internal name/key
  display_name: string; // User-facing label
  field_type_id: number;
  field_type_name: string; // Joined name ('TEXT', 'NUMBER', etc.)
  required: boolean;
  options: any | null; // JSON options (structure varies, e.g., for SELECT)
  validation_name: string | null; // e.g., 'email', 'nif'
  is_searchable: boolean;
  is_displayed_in_table: boolean;
  order_index: number;
}

// Matches backend CreatePageFieldRequest (used within CreateCustomPageRequest)
// Also useful for adding fields individually
export interface CreatePageFieldRequest {
  name: string;
  display_name: string;
  field_type_id: number;
  required: boolean;
  options?: any | null; // JSON
  validation_name?: string | null;
  is_searchable: boolean;
  is_displayed_in_table: boolean;
  order_index: number;
}

// Matches backend UpdatePageFieldRequest
export interface UpdatePageFieldRequest {
  display_name: string;
  field_type_id: number;
  required: boolean;
  options?: any | null; // JSON
  validation_name?: string | null;
  is_searchable: boolean;
  is_displayed_in_table: boolean;
  order_index: number;
}

// Matches backend ValidationFunction
export interface ValidationFunction {
  name: string;
  description: string;
}
