import { writable } from "svelte/store";
import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";

export const isAdmin = writable<boolean | null>(null);
export const isAuthenticated = writable<boolean | null>(null);

// Response type from backend /users/check
interface CheckAuthResponse {
  isAdmin: boolean;
  canManageThisPage?: boolean; // This is optional
}

export async function checkAuthStatus(): Promise<void> {
  // Avoid checking if already definitively known
  // let currentAuth = get(isAuthenticated);
  // if (currentAuth !== null) return;

  if (typeof window === "undefined") return; // Client-side only

  try {
    const currentPath = window.location.pathname;
    const response = await handleFetch(`${API_BASE_URL}/users/check`, {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "text/plain" },
      body: currentPath,
    });

    if (response.ok) {
      const data: CheckAuthResponse = await response.json();
      // Store only the general admin status globally
      isAdmin.set(data.isAdmin);
      isAuthenticated.set(true);
    } else if (response.status === 401) {
      isAdmin.set(false);
      isAuthenticated.set(false);
    } else {
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
    checkAuthStatus();
  }
}
