import API_BASE_URL from "@api/base-url";

import { handleFetch } from "@api/fetch-handler";

export async function getAnalytics(cookie?: any): Promise<PageAnalytics> {
  const response = await handleFetch(`${API_BASE_URL}/users/analytics`, {
    method: "GET",
    credentials: "include",
    // @ts-ignore
    headers: {
      Cookie: cookie,
    },
  });

  if (response.ok) { // Handles 2xx
    try {
      return (await response.json()) as PageAnalytics;
    } catch (e) {
      console.error("getAnalytics: Error parsing JSON from successful analytics response:", e);
      return []; // Return empty if JSON parsing fails even on OK
    }
  }

  // Handle non-OK responses here
  if (response.status === 401) {
    // This log might be redundant if handleFetch already logs it, but can be useful for specific context.
    // console.warn("getAnalytics: Unauthorized (401) accessing analytics. Returning empty array.");
  } else {
    // For other errors (e.g., 500, 404 from API itself)
    let errorText = "";
    try {
      // Try to get error text, but don't fail if this itself errors (e.g., if body is already consumed or not text)
      errorText = await response.text();
    } catch (e_text) {
        // silence
    }
    console.error(`getAnalytics: API request failed with status ${response.status}. Body: ${errorText}`);
  }
  return []; // Return empty for all non-OK responses (including 401)
}
