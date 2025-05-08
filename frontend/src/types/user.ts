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
  // For creating a user
  export interface CreateUserRequest {
      username: string;
      email: string;
      password: string;
      // role_ids: number[]; // Add this if backend register supports roles
  }
  
  export interface CreateUserResponse {
      success: boolean;
      userId?: number;
      error?: string;
  }
  user_id: number;
  role_ids: number[];
}
