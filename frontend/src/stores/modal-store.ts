import { writable } from "svelte/store";

export const currentModal = writable<HTMLDivElement | null>(null);
