// Import showAlert dynamically ONLY if needed client-side, or guard its usage
// import { AlertPosition, AlertType, showAlert } from "@components/alert/alert"; // Keep import if guarded
import { toggleElements } from "@stores/loading-stores";

export async function handleFetch(
  url: string | URL,
  options: RequestInit,
): Promise<Response> {
  try {
    const response = await fetch(url, options);

    // Check if running in the browser before doing client-side redirects or alerts
    if (typeof window !== "undefined") {
      if (
        response.status === 401 &&
        window.location.pathname !== "/iniciar-sessao/" &&
        window.location.pathname !== "/registo/"
      ) {
        // Redirect to login only on the client
        window.location.pathname = "/iniciar-sessao/";
        // Return a dummy response or throw an error to stop further client processing
        // Or simply let the redirect happen. Returning the original response might be confusing.
        // Throwing an error might be better to signal interruption.
        throw new Error("Unauthorized, redirecting to login.");
      }
    } else {
      // Server-side specific checks if needed (e.g., logging 401s)
      if (response.status === 401) {
        console.warn(`Server-side fetch to ${url} resulted in 401.`);
      }
    }

    return response; // Return the response whether on client or server
  } catch (error: any) {
    // Check if running in the browser before trying to show an alert
    if (typeof window !== "undefined") {
      // Only toggle UI elements and show alert on the client
      toggleElements(); // Assuming this store/function is client-safe or does nothing on server

      // Dynamically import showAlert only when needed on the client
      const { showAlert, AlertType, AlertPosition } = await import(
        "@components/alert/alert"
      );
      showAlert(
        "Erro ao comunicar com o servidor",
        AlertType.ERROR,
        AlertPosition.TOP,
      );
    } else {
      // Log the error on the server
      console.error(`Server-side fetch error for ${url}:`, error.message);
    }

    // Re-throw the error so the calling code (server or client) knows something went wrong
    throw error;
  }
}
