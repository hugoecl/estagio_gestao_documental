import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";
import type {
  Role,
  CreateRoleRequest,
  UpdateRoleRequest,
} from "@lib/types/roles"; // Define this type

export async function getRoles(cookie?: string): Promise<Role[]> {
  const response = await handleFetch(`${API_BASE_URL}/roles`, {
    method: "GET",
    credentials: "include",
    headers: cookie ? { Cookie: cookie } : undefined,
  });
  if (response.ok) {
    return await response.json();
  }
  if (response.status === 304) return [];
  throw new Error(`Failed to fetch roles: ${response.statusText}`);
}

export async function createRole(
  data: CreateRoleRequest,
): Promise<{ success: boolean; roleId?: number }> {
  const response = await handleFetch(`${API_BASE_URL}/roles`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  if (response.ok) {
    // Assuming backend returns the new role ID in the body
    const newRoleIdText = await response.text();
    try {
      return { success: true, roleId: parseInt(newRoleIdText, 10) };
    } catch {
      return { success: true }; // Success but couldn't parse ID
    }
  }
  return { success: false };
}

export async function updateRole(
  roleId: number,
  data: UpdateRoleRequest,
): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/roles/${roleId}`, {
    method: "PUT",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  return response.ok;
}

export async function deleteRole(roleId: number): Promise<boolean> {
  // Prevent deleting the default Admin role (ID 1 is assumed, adjust if different)
  if (roleId === 1) {
    console.warn(
      "Attempted to delete the Admin role (ID 1). Operation blocked.",
    );
    // Optionally throw an error or return false with a specific message
    // throw new Error("Cannot delete the default Admin role.");
    return false;
  }
  const response = await handleFetch(`${API_BASE_URL}/roles/${roleId}`, {
    method: "DELETE",
    credentials: "include",
  });
  return response.ok || response.status === 204; // OK or No Content
}
