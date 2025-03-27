// Field type definitions
type FieldType =
  | "text"
  | "number"
  | "select"
  | "date"
  | "dateRange"
  | "textarea";

interface SelectOption {
  value: string | number;
  label: string;
}

interface FormField {
  id: string;
  type: FieldType;
  label: string;
  placeholder?: string;
  value: any;
  required?: boolean;
  options?: SelectOption[]; // For select fields
  colSpan?: number; // How many columns to span (default: 1)
}

interface FileData {
  id: string;
  name: string;
  path: string;
  uploadedAt: string;
}
