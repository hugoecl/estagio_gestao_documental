export interface FieldType {
  id: number;
  name: string; // e.g., "TEXT", "NUMBER", "SELECT", "DATE", "DATE_RANGE", "TEXTAREA"
}

export interface PageField {
  id: number;
  page_id: number;
  name: string; // Internal field name/key
  display_name: string; // User-facing label
  field_type_id: number;
  field_type_name: string; // Joined from field_types table
  required: boolean;
  options: any | null; // JSON value, typically string[] for SELECT
  validation_name: string | null;
  is_searchable: boolean;
  is_displayed_in_table: boolean;
  order_index: number;
}

export interface UpdatePageFieldRequest {
  display_name: string;
  field_type_id: number;
  required: boolean;
  options: any | null;
  validation_name: string | null;
  is_searchable: boolean;
  is_displayed_in_table: boolean;
  order_index: number;
}

// Matches backend models/validation.rs
export interface ValidationFunction {
  name: string;
  description: string;
}

// Re-export CreatePageFieldRequest if needed elsewhere, or keep it in custom-page.ts
export type { CreatePageFieldRequest } from "./custom-page";
