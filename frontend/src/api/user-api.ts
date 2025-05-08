import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";
import type { UserWithRoles, AssignRoleRequest } from "@lib/types/user";
import type { CreateUserRequest, CreateUserResponse } from "@lib/types/user";

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
export async function createUser(userData: CreateUserRequest): Promise<CreateUserResponse> {
    // Use the existing /users/register endpoint
    // Note: The backend register currently doesn't handle assigning roles other than the default.
    // To assign specific roles on creation, the backend endpoint would need modification.
    // For now, we just create the user. Role assignment happens via assignRolesToUser later.
    const response = await handleFetch(`${API_BASE_URL}/users/register`, {
        method: "POST",
        // No credentials needed for public register (usually), but include if backend requires admin auth
        // credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(userData), // Send username, email, password
    });

    if (response.ok) {
        const responseText = await response.text();
        // Try to extract the ID from the success message "User registered successfully with ID X"
        const match = responseText.match(/ID (\d+)$/);
        const userId = match ? parseInt(match[1], 10) : undefined;
        return { success: true, userId: userId };
    } else if (response.status === 409) { // Conflict (username/email exists)
        return { success: false, error: "Username or email already exists." };
    } else {
        const errorText = await response.text();
        return { success: false, error: `Failed to create user: ${errorText || response.statusText}` };
    }
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
