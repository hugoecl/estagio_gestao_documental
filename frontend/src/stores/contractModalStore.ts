import { writable } from "svelte/store";
import type { Contract } from "@lib/types/contracts";

export const selectedContractId = writable<string | null>(null);
export const selectedContract = writable<Contract | null>(null);
export const isContractModalSubmitting = writable<boolean>(false);

export function resetContractModalState() {
  selectedContractId.set(null);
  selectedContract.set(null);
  isContractModalSubmitting.set(false);
}
