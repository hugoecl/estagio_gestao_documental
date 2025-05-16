import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";
import type {
  VacationRequest,
  CreateVacationRequestPayload,
  RemainingVacationDaysResponse,
} from "@lib/types/vacation"; // Adjust path if your types are in /types/

/**
 * Fetches all vacation requests for the currently logged-in user.
 */
export async function getMyVacationRequests(): Promise<VacationRequest[]> {
  const response = await handleFetch(
    `${API_BASE_URL}/vacation-requests/me`,
    {
      method: "GET",
      credentials: "include",
    },
  );
  if (response.ok) {
    return (await response.json()) as VacationRequest[];
  }
  // Handle non-OK responses (e.g., 401, 500)
  // Consider throwing an error or returning an empty array based on how you want to handle errors upstream
  console.error(
    `Failed to fetch user vacation requests: ${response.statusText}`,
  );
  return []; // Or throw new Error(...)
}

/**
 * Fetches the remaining vacation days for the currently logged-in user.
 */
export async function getMyRemainingVacationDays(): Promise<RemainingVacationDaysResponse | null> {
  const response = await handleFetch(
    `${API_BASE_URL}/users/me/vacation-days`,
    {
      method: "GET",
      credentials: "include",
    },
  );
  if (response.ok) {
    return (await response.json()) as RemainingVacationDaysResponse;
  }
  if (response.status === 401) {
    // Unauthorized, likely session expired or not logged in
    console.warn("getMyRemainingVacationDays: Unauthorized (401).");
    return null;
  }
  console.error(
    `Failed to fetch remaining vacation days: ${response.statusText}`,
  );
  // Consider specific error handling or re-throwing
  return null;
}

/**
 * Submits a new vacation request for the logged-in user.
 * @param data The payload for creating the vacation request.
 * @returns An object with the ID of the created request if successful.
 */
export async function submitVacationRequest(
  data: CreateVacationRequestPayload,
): Promise<{ id: number; success: boolean; message?: string }> {
  const response = await handleFetch(`${API_BASE_URL}/vacation-requests`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  });

  if (response.ok) {
    // Backend returns { "id": new_id } on 201 Created
    const result = (await response.json()) as { id: number };
    return { ...result, success: true };
  } else {
    // Try to get an error message from the backend
    let errorMessage = `Erro ao submeter pedido: ${response.statusText}`;
    try {
      const errorBody = await response.text(); // Use text() first to avoid JSON parse error if not JSON
      if (errorBody) {
        errorMessage = errorBody; // Use the backend's message if provided
      }
    } catch (e) {
      // Ignore if body can't be parsed
    }
    console.error(
      "submitVacationRequest failed:",
      response.status,
      errorMessage,
    );
    return { id: -1, success: false, message: errorMessage };
  }
}

// --- Admin specific vacation API calls would go here in a later phase ---
// e.g., getPendingRequestsForRole, actionVacationRequest

/**
 * Fetches approved and pending vacation date ranges for colleagues in shared holiday roles for a given year.
 * @param year The year for which to fetch shared calendar data.
 * @returns A promise that resolves to an array of objects with start_date, end_date, and status.
 */
export async function getSharedCalendarVacations(year: number): Promise<{start_date: string, end_date: string, status: string}[]> {
  const response = await handleFetch(
    `${API_BASE_URL}/vacation-requests/shared-calendar?year=${year}`,
    {
      method: "GET",
      credentials: "include",
    },
  );

  if (response.ok) {
    return (await response.json()) as {start_date: string, end_date: string, status: string}[];
  }

  if (response.status === 401) {
    console.warn("getSharedCalendarVacations: Unauthorized (401). Returning empty array.");
    return [];
  }

  console.error(
    `Failed to fetch shared calendar vacations for year ${year}: ${response.statusText}`,
  );
  return []; // Or throw an error if preferred
}
