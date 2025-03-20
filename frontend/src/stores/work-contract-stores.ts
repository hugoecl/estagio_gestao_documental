import { writable } from "svelte/store";

import type {
  WorkContract,
  WorkContractCategories,
  WorkContractCategory,
} from "@lib/types/work-contracts";

export const newCategory = writable<{
  id: number;
  category: WorkContractCategory;
} | null>(null);

export const categories = writable<WorkContractCategories>({});

export const newWorkContract = writable<{
  id: number;
  workContract: WorkContract;
} | null>(null);
