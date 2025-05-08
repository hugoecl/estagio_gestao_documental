export interface PageRecord {
  id: number;
  page_id: number;
  data: Record<string, any>; // The dynamic JSON data
  created_at: string; // Consider Date
  updated_at: string; // Consider Date
  created_by: number;
  updated_by: number | null;

  // Add client-side processed fields if needed (like formatted dates)
  processedData?: Record<string, any>;
}

export interface CreatePageRecordRequest {
  data: Record<string, any>;
}

export interface UpdatePageRecordRequest {
  data: Record<string, any>;
}

export interface PageRecordFile {
  id: number;
  record_id: number;
  file_name: string;
  file_path: string;
  uploaded_at: string; // Consider Date
  uploaded_by: number;

  // Client-side helper
  url?: string;
}

export interface PageRecordWithFiles {
  record: PageRecord;
  files: PageRecordFile[];
}
