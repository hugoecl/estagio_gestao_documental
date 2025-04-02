import type { Locations } from "@lib/types/locations";
import type { FileData } from "@lib/types/files";

export interface WorkContractCategory {
  name: string;
  description: string;
  createdAt: string;
  updatedAt: string;

  // Things made for the sake of performance that are added when the data is fetched
  __searchName: string;
  __searchDescription: string;
  __createdAtDate: Date;
  __updatedAtDate: Date;
}

export type WorkContractCategories = Record<number, WorkContractCategory>;

export const WorkContractTypes = ["Adenda", "Contrato de Funcionário"] as const;

export interface WorkContract {
  employeeName: string;
  nif: string;
  dateStart: Date;
  dateEnd?: Date;
  type: (typeof WorkContractTypes)[number];
  location: (typeof Locations)[number];
  category: string;
  categoryId: number;
  description?: string;
  createdAt: string;
  updatedAt: string;
  files: Record<number, FileData>;

  // Things made for the sake of performance that are added when the data is fetched
  dateStartString: string;
  dateEndString?: string;

  typeValue: number;
  locationValue: number;

  __searchEmployeeName: string;
  __searchType: string;
  __searchLocation: string;
  __searchCategory: string;
  __searchDescription?: string;
}

export type WorkContracts = Record<number, WorkContract>;
