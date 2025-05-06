import { writable } from "svelte/store";
import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";

export const isAdmin = writable<boolean | null>(null);
export const isAuthenticated = writable<boolean | null>(null);

export async function checkAuthStatus(): Promise<void> {
  // Avoid checking if already definitively known (true/false)
  // let currentAuth = get(isAuthenticated); // Svelte 5 might have a better way
  // if (currentAuth !== null) return;

  // Ensure this runs only on the client
  if (typeof window === "undefined") {
    // console.log("Skipping checkAuthStatus on server.");
    return;
  }

  try {
    const currentPath = window.location.pathname;
    const response = await handleFetch(`${API_BASE_URL}/users/check`, {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "text/plain" },
      body: currentPath,
    });

    if (response.ok) {
      // Status 200-299
      const data: { isAdmin: boolean } = await response.json();
      isAdmin.set(data.isAdmin);
      isAuthenticated.set(true);
    } else if (response.status === 401) {
      // Unauthorized
      isAdmin.set(false);
      isAuthenticated.set(false);
      // No redirect here, middleware handles it
    } else {
      // Other errors (403, 500, etc.)
      console.error(`Auth check failed with status: ${response.status}`);
      isAdmin.set(false);
      isAuthenticated.set(false);
    }
  } catch (error) {
    console.error("Error during client-side auth check fetch:", error);
    isAdmin.set(false);
    isAuthenticated.set(false);
  }
}

export function initializeAuthCheck(): void {
  if (typeof window !== "undefined") {
    // console.log("Initializing auth check..."); // Debugging
    checkAuthStatus();
  }
}
