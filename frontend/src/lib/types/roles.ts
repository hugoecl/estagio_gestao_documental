export interface Role {
  id: number;
  name: string;
  description: string | null;
  is_admin: boolean;
  created_at: string;
  updated_at: string;
}

export interface RoleWithInterferingRoles {
  id: number;
  name: string;
  description: string | null;
  is_admin: boolean;
  created_at: string;
  updated_at: string;
  interfering_role_ids: number[];
}

export interface CreateRoleRequest {
  name: string;
  description: string | null;
  is_admin: boolean;
  interfering_role_ids?: number[];
}

export interface UpdateRoleRequest {
  name: string;
  description: string | null;
  is_admin: boolean;
  interfering_role_ids?: number[];
} 