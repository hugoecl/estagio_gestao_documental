import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";

/**
 * Sends a broadcast notification message to users in the specified roles.
 * Requires admin privileges on the backend.
 * @param roleIds An array of role IDs to send the notification to.
 * @param message The message content of the notification.
 * @returns True if the broadcast was accepted by the backend, false otherwise.
 */
export async function broadcastNotification(
  roleIds: number[],
  message: string,
): Promise<{ success: boolean; message: string }> {
  const response = await handleFetch(
    `${API_BASE_URL}/notifications/broadcast`,
    {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ role_ids: roleIds, message }),
    },
  );

  const responseText = await response.text(); // Get response text regardless of status

  if (response.ok) {
    return { success: true, message: responseText || "Mensagem enviada com sucesso." };
  } else {
    console.error(
      `Failed to broadcast notification: ${response.status} ${response.statusText}`,
      responseText,
    );
    return { success: false, message: responseText || `Falha ao enviar mensagem: ${response.statusText}` };
  }
}

// Add other notification-related API functions here if needed in the future,
// e.g., to fetch all notifications (not just unread), or user preferences for notifications.