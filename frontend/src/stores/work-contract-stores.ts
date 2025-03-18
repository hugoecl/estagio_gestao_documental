import { writable } from "svelte/store";

import type { WorkContractCategory } from "@lib/types/work-contracts";

export const newCategory = writable<{
  id: number;
  category: WorkContractCategory;
} | null>(null);
