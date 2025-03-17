export type TableColumn = {
  header: string; // Column header text
  field: string; // Field path in the data object (e.g., "name")
  responsive?: string; // Tailwind classes for responsive behavior (e.g., "hidden sm:table-cell")
};
