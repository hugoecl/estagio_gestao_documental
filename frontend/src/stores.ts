import { writable } from "svelte/store";

/**
 * Is used to store the elements that which visibility needs to be toggled when an unhandled error when fetching occurs.
 * Useful for loading bars/spinners.
 */
export const elementsToToggle = writable<HTMLElement[] | Element[]>([]);

/**
 * Toggles the visibility of all elements in the elementsToToggle store.
 * This is used when unhandled errors occur during fetch requests to stop the loading bars.
 *
 * @returns void
 */
export function toggleElements() {
  elementsToToggle.update((elements) => {
    for (let i = 0, len = elements.length; i < len; i++) {
      elements[i].classList.toggle("hidden");
    }
    return elements;
  });
}
