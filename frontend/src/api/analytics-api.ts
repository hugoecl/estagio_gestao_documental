import { handleFetch } from "@api/fetch-handler";
import API_BASE_URL from "@api/base-url";

export async function getAnalytics(): Promise<PageAnalytics> {
  const response = await handleFetch(`${API_BASE_URL}/users/analytics`, {
    method: "GET",
    credentials: "include",
  });

  if (response.ok) {
    return (await response.json()) as PageAnalytics;
  }
  return [];
}
