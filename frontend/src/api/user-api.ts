import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";
import type { UserWithRoles, AssignRoleRequest } from "@lib/types/user";

export async function getUsersWithRoles(
  cookie?: string,
): Promise<UserWithRoles[]> {
  const response = await handleFetch(`${API_BASE_URL}/users/all`, {
    method: "GET",
    credentials: "include",
    headers: cookie ? { Cookie: cookie } : undefined,
  });
  if (response.ok) {
    return await response.json();
  }
  if (response.status === 304) return [];
  throw new Error(`Failed to fetch users: ${response.statusText}`);
}

export async function assignRolesToUser(
  assignment: AssignRoleRequest,
): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/users/roles`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(assignment),
  });
  return response.ok;
}
