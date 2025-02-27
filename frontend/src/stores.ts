import { writable } from "svelte/store";

export const isLoading = writable(false);
export const elementsToToggle = writable<HTMLElement[]>([]);

export function toggleLoading() {
  isLoading.update((value) => !value);
}

export function toggleElements() {
  elementsToToggle.update((elements) => {
    for (let i = 0, len = elements.length; i < len; i++) {
      elements[i].hidden = !elements[i].hidden;
    }
    return elements;
  });
}
