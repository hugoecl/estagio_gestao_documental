import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";
import type { Role } from "@lib/types/roles"; // Define this type

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

// Add create/update/delete role functions later if needed
