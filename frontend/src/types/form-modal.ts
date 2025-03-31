export const enum FieldType {
  TEXT,
  NUMBER,
  SELECT,
  DATE,
  DATE_RANGE,
  TEXTAREA,
}

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
  searchField?: string; // For search fields
  validate?: (value: any) => string | null;
}

export const enum SubmitResult {
  SUCCESS,
  ERROR,
  UNCHANGED,
}

export type SubmitResponse =
  | [SubmitResult.SUCCESS, Record<string, any>]
  | [Exclude<SubmitResult, SubmitResult.SUCCESS>, null];
