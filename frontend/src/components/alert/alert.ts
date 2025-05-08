import { mount, unmount } from "svelte";
// @ts-ignore Assume Alert.svelte is fine for now
import Alert from "./Alert.svelte";

export const enum AlertType {
  SUCCESS = 0,
  INFO = 1,
  WARNING = 2,
  ERROR = 3,
}

export const enum AlertPosition {
  TOP = 0,
  BOTTOM_RIGHT = 1,
}

// Declare variables, but don't assign them using 'document' here
let toastTop: HTMLElement | null = null;
let toastBottomRight: HTMLElement | null = null;

// Function to find elements, call this when needed on the client
function findToastContainers() {
  // Only run if in a browser environment
  if (typeof document !== "undefined") {
    toastTop = document.getElementById("toast-top");
    toastBottomRight = document.getElementById("toast-bottom-right");
  }
}

// Use astro:page-load to find elements after navigation
// Ensure this runs only on the client
if (typeof document !== "undefined") {
  document.addEventListener("astro:page-load", () => {
    findToastContainers();
  });
  // Also try to find them on initial load for non-SPA navigation
  findToastContainers();
}

export function showAlert(
  message: string,
  type: AlertType,
  position: AlertPosition,
) {
  // Ensure this function only runs client-side and elements are found
  if (typeof document === "undefined") {
    console.warn("showAlert called on the server. Message:", message);
    return; // Don't run on server
  }

  // Ensure toast containers are found (might not be if called too early)
  if (!toastTop || !toastBottomRight) {
    findToastContainers(); // Try finding them again
  }

  const targetElement =
    position === AlertPosition.TOP ? toastTop : toastBottomRight;

  if (!targetElement) {
    console.error(
      `Toast container for position ${position === AlertPosition.TOP ? "TOP" : "BOTTOM_RIGHT"} not found.`,
    );
    // Fallback: maybe log to console or try appending to body?
    // As a simple fallback, just log the error.
    return;
  }

  // Proceed with mounting the Svelte component
  try {
    const alertInstance = mount(Alert, {
      target: targetElement,
      props: {
        message,
        type,
        position,
      },
    });

    // Auto-remove the alert after a delay
    setTimeout(() => {
      // Check if the component is still mounted before trying to unmount
      // This check might be overly cautious depending on Svelte's internals,
      // but prevents errors if the element was somehow removed manually.
      if (alertInstance && typeof alertInstance.$destroy === "function") {
        // Svelte 5 uses unmount
        unmount(alertInstance);
        // Or if using Svelte 4 style: alertInstance.$destroy();
      }
    }, 3000);
  } catch (error) {
    console.error("Error mounting alert component:", error);
  }
}
