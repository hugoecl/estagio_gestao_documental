import type { Role } from "@lib/types/roles";

// Basic user info - used for /me endpoint and potentially elsewhere
export interface User {
  id: number;
  username: string;
  email: string;
}

// User with their assigned roles - used for admin user list
export interface UserWithRoles extends User {
  roles: Role[];
  // 'role' string might be a client-side concatenation or a specific primary role display;
  // if it comes from backend, ensure backend provides it or derive on client.
  // For now, assuming 'roles' array is the source of truth.
  role?: string; // Optional: A display string for roles, can be derived client-side
  vacation_days_current_year?: number; // Optional: Number of vacation days for the current year
}

// For creating a user (admin or public registration)
export interface CreateUserRequest {
  username: string;
  email: string;
  password: string;
  // role_ids could be added here if backend /register endpoint is enhanced
  // to allow specifying roles on creation by an admin.
  // For now, role assignment is a separate step via assignRolesToUser.
}

export interface CreateUserResponse {
  success: boolean;
  userId?: number;
  error?: string;
}

// For assigning roles to a user via API
export interface AssignRoleRequest {
  user_id: number;
  role_ids: number[];
}

// --- User Settings Payloads ---

// For updating user's own username/email
export interface UpdateUserDetailsPayload {
  username?: string; // Optional: only send if changed
  email?: string;    // Optional: only send if changed
  current_password: string; // Always required to confirm identity
}

// For user changing their own password
export interface ChangePasswordPayload {
  current_password: string;
  new_password: string;
}

// --- Admin User Modification Payloads ---

// For admin updating a specific user's details
export interface AdminUpdateUserPayload {
  username?: string; // Optional: only send if changed
  email?: string;    // Optional: only send if changed
  vacation_days_current_year?: number; // Optional: only send if changed
  // Admin does not need to provide a password to change these details
}

// For admin setting/changing a specific user's password
export interface AdminSetPasswordPayload {
  new_password: string;
}