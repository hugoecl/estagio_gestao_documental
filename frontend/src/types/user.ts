import type { Role } from "@lib/types/roles";

// Basic user info
export interface User {
  id: number;
  username: string;
  email: string;
}

// User with their assigned roles
export interface UserWithRoles extends User {
  roles: Role[];
  role: string;
}

// For assigning roles via API
export interface AssignRoleRequest {
  user_id: number;
  role_ids: number[];
}
