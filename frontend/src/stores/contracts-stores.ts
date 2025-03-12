import { writable } from "svelte/store";

import type { Contract } from "@lib/types/contracts";

export const newContract = writable<{ id: number; contract: Contract } | null>(
  null
);
