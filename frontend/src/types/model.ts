import type { FileData } from "@lib/types/files";

export interface Model {
  name: string;
  version: string;
  model: string;
  description: string | null;
  updatedAt: string;
  createdAt: string;
  files: Record<string, FileData>;

  updatedAtDate: Date;
  createdAtDate: Date;

  __searchName: string;
  __searchDescription: string | null;
}

export type Models = Record<string, Model>;
