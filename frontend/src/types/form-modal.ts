export const enum FieldType {
  TEXT,
  NUMBER,
  SELECT,
  DATE,
  DATE_RANGE,
  TEXTAREA,
  CHECKBOX,
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
  value: any; // This will be managed by the modal's internal state
  required?: boolean;
  options?: SelectOption[];
  colSpan?: number;
  // Removed searchField as it's not directly used in the modal
  validation_name?: string | null; // Add validation name from backend
  // Removed the client-side validate function, logic will be internal to modal
}

export const enum SubmitResult {
  SUCCESS,
  ERROR,
  UNCHANGED,
}

export type SubmitResponse =
  | [SubmitResult.SUCCESS, Record<string, any>]
  | [Exclude<SubmitResult, SubmitResult.SUCCESS>, null];
