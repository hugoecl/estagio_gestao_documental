export interface Role {
  id: number;
  name: string;
  description: string | null;
  is_admin: boolean;
  is_holiday_role: boolean; // New field
  created_at: string; // Consider Date
  updated_at: string; // Consider Date
}

export interface CreateRoleRequest {
  name: string;
  description: string | null;
  is_admin: boolean;
  is_holiday_role: boolean; // New field
}

// Matches backend UpdateRoleRequest
export interface UpdateRoleRequest {
  name: string;
  description: string | null;
  is_admin: boolean;
  is_holiday_role: boolean; // New field
}
