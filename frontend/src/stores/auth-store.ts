import { writable } from "svelte/store";
import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler"; // Use your fetch handler

// Store to hold the admin status. null means unchecked, false/true after check.
export const isAdmin = writable<boolean | null>(null);
export const isAuthenticated = writable<boolean | null>(null); // Track overall auth status

// Function to check authentication status and admin rights
export async function checkAuthStatus(): Promise<void> {
  // Avoid redundant checks if status is already known (true/false)
  // let currentAuthStatus: boolean | null = null;
  // isAuthenticated.subscribe(value => currentAuthStatus = value)(); // Get current value non-reactively
  // if (currentAuthStatus !== null) return; // Already checked

  try {
    // Use the current path for analytics tracking in check endpoint
    const currentPath =
      typeof window !== "undefined" ? window.location.pathname : "/";

    const response = await handleFetch(`${API_BASE_URL}/users/check`, {
      method: "POST",
      credentials: "include", // Send cookies
      headers: {
        "Content-Type": "text/plain", // Send path as plain text
      },
      body: currentPath,
    });

    if (response.ok) {
      const data: { isAdmin: boolean } = await response.json();
      isAdmin.set(data.isAdmin);
      isAuthenticated.set(true);
    } else {
      // If check fails (e.g., 401 Unauthorized), user is not authenticated
      isAdmin.set(false);
      isAuthenticated.set(false);
      // Optional: Redirect here if not on login/register page, though middleware handles this too
      // if (typeof window !== 'undefined' && window.location.pathname !== '/iniciar-sessao/' && window.location.pathname !== '/registo/') {
      //     window.location.href = '/iniciar-sessao/';
      // }
    }
  } catch (error) {
    console.error("Error checking auth status:", error);
    // Assume not authenticated or admin on error
    isAdmin.set(false);
    isAuthenticated.set(false);
  }
}

// Optional: Function to trigger check, maybe called from layout
export function initializeAuthCheck(): void {
  // Run check only on the client
  if (typeof window !== "undefined") {
    checkAuthStatus();
  }
}
