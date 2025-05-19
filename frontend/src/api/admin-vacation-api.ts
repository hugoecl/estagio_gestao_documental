import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";
import type { Role } from "@lib/types/roles"; // Assuming Role type is defined here
import type { VacationRequestStatus, VacationRequestWithUser } from "@lib/types/vacation"; // Import VacationRequestStatus


/**
 * Fetches all roles that are marked as "holiday roles".
 * Requires admin privileges on the backend.
 * @param cookie Optional cookie string for server-side rendering or specific auth needs.
 * @returns A promise that resolves to an array of Role objects.
 */
export async function getHolidayRoles(cookie?: string): Promise<Role[]> {
  const response = await handleFetch(
    `${API_BASE_URL}/admin/vacations/roles`, // Ensure this matches your backend route
    {
      method: "GET",
      credentials: "include",
      headers: cookie ? { Cookie: cookie } : undefined,
    },
  );

  if (response.ok) {
    return (await response.json()) as Role[];
  }

  // Handle common error cases or let handleFetch manage them
  if (response.status === 401 || response.status === 403) {
    console.warn(
      `getHolidayRoles: Unauthorized (${response.status}) or Forbidden. Returning empty array.`,
    );
    return []; // Return empty on auth errors, as admin page might try to load this initially
  }

  // For other errors, log and return empty or throw
  let errorText = "";
  try {
    errorText = await response.text();
  } catch (e_text) {
    // silence
  }
  console.error(
    `Failed to fetch holiday roles: ${response.status} ${response.statusText}. Body: ${errorText}`,
  );
  // Depending on desired error handling, you might throw an error here
  // throw new Error(`Failed to fetch holiday roles: ${response.statusText}`);
  return []; // Or return empty array for other errors
}

// Future admin vacation API functions can be added here:
// - Function to approve or reject a vacation request

/**
 * Fetches pending vacation requests for users within a specific role.
 * Requires admin privileges.
 * @param roleId The ID of the role.
 * @param cookie Optional cookie string.
 * @returns A promise that resolves to an array of VacationRequestWithUser objects.
 */
export async function getPendingRequestsForRole(
  roleId: number,
  cookie?: string,
): Promise<VacationRequestWithUser[]> {
  const response = await handleFetch(
    `${API_BASE_URL}/admin/vacations/role/${roleId}/pending-requests`,
    {
      method: "GET",
      credentials: "include",
      headers: cookie ? { Cookie: cookie } : undefined,
    },
  );

  if (response.ok) {
    return (await response.json()) as VacationRequestWithUser[];
  }

  if (response.status === 401 || response.status === 403) {
    console.warn(
      `getPendingRequestsForRole: Unauthorized (${response.status}) or Forbidden for role ${roleId}. Returning empty array.`,
    );
    return [];
  }

  let errorText = "";
  try {
    errorText = await response.text();
  } catch (e_text) {
    // silence
  }
  console.error(
    `Failed to fetch pending requests for role ${roleId}: ${response.status} ${response.statusText}. Body: ${errorText}`,
  );
  return []; // Or throw an error
}

/**
 * Admin action to approve or reject a vacation request.
 * @param requestId The ID of the vacation request to action.
 * @param payload The action details (new status and optional admin notes).
 * @returns A promise that resolves to an object indicating success and a message.
 */
export async function actionVacationRequest(
  requestId: number,
  payload: { status: 'APPROVED' | 'REJECTED'; admin_notes?: string | null }, // Use string literals for status
): Promise<{ success: boolean; message?: string }> {
  // Map the uppercase status to PascalCase for the backend
  const mappedStatus = payload.status === 'APPROVED' ? 'Approved' : 'Rejected';
  
  const mappedPayload = {
    status: mappedStatus,
    admin_notes: payload.admin_notes
  };
  
  const response = await handleFetch(
    `${API_BASE_URL}/admin/vacations/request/${requestId}/action`,
    {
      method: "PUT",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(mappedPayload),
    },
  );

  const responseText = await response.text(); // Get text for success or error message

  if (response.ok) {
    return { success: true, message: responseText || "Ação concluída com sucesso." };
  } else {
    console.error(
      `actionVacationRequest failed for request ${requestId}: ${response.status} ${response.statusText}`,
      responseText,
    );
    return { success: false, message: responseText || `Falha ao processar o pedido: ${response.statusText}` };
  }
}