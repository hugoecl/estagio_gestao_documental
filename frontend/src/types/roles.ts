export interface Role {
  id: number;
  name: string;
  description: string | null;
  is_admin: boolean;
  created_at: string; // Consider Date
  updated_at: string; // Consider Date
}
