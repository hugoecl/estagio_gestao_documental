import API_BASE_URL from "@api/base-url";

export async function getAnalytics(cookie?: any): Promise<PageAnalytics> {
  const response = await fetch(`${API_BASE_URL}/users/analytics`, {
    method: "GET",
    credentials: "include",
    // @ts-ignore
    headers: {
      Cookie: cookie,
    },
  });

  if (response.ok) {
    return (await response.json()) as PageAnalytics;
  }
  return [];
}
