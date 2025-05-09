import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";

/**
 * Submits an acknowledgment for a specific record by the current user.
 * @param recordId The ID of the record to acknowledge.
 * @returns True if the acknowledgment was successful or already existed, false otherwise.
 */
export async function acknowledgeRecord(recordId: number): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/records/${recordId}/acknowledge`,
    {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json", // Though body is empty, good practice
      },
      // No body needed for this specific POST request as per current backend setup
    },
  );
  // Backend returns 201 Created if newly acknowledged, 200 OK if already acknowledged
  return response.ok;
}

/**
 * Checks if the current user has acknowledged a specific record.
 * This will require a new backend endpoint.
 * Let's assume an endpoint like GET /api/records/{record_id}/acknowledgment-status
 * which returns { acknowledged: boolean }
 * @param recordId The ID of the record to check.
 * @returns True if the current user has acknowledged the record, false otherwise.
 */
export async function checkIfRecordAcknowledged(
  recordId: number,
): Promise<boolean> {
  try {
    const response = await handleFetch(
      `${API_BASE_URL}/records/${recordId}/acknowledgment-status`, // NEW Backend endpoint needed
      {
        method: "GET",
        credentials: "include",
      },
    );
    if (response.ok) {
      const data = (await response.json()) as { acknowledged: boolean };
      return data.acknowledged;
    }
    if (response.status === 404) {
      // Could mean record not found or endpoint not implemented yet
      console.warn(
        `Acknowledgment status check for record ${recordId} returned 404.`,
      );
      return false; // Assume not acknowledged if record/endpoint is not found
    }
    console.error(
      `Failed to check acknowledgment status for record ${recordId}: ${response.statusText}`,
    );
    return false; // Default to false on other errors
  } catch (error) {
    console.error("Error checking record acknowledgment status:", error);
    return false; // Default to false on network/fetch errors
  }
}
