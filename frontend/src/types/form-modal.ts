// Field type definitions
export type FieldType =
  | "text"
  | "number"
  | "select"
  | "date"
  | "dateRange"
  | "textarea";

export interface SelectOption {
  value: string | number;
  label: string;
}

export interface FormField {
  id: string;
  type: FieldType;
  label: string;
  placeholder?: string;
  value: any;
  required?: boolean;
  options?: SelectOption[]; // For select fields
  colSpan?: number; // How many columns to span (default: 1)
}

export const enum SubmitResult {
  SUCCESS,
  ERROR,
  UNCHANGED,
}
