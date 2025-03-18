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

export interface WorkContractFiles {
  name: string;
  path: string;
  uploadedAt: string;
}
export interface WorkContract {
  name: string;
  nif: string;
  dateStart: Date;
  dateEnd?: Date;
  type: (typeof WorkContractTypes)[number];
  categoryId: number;
  description?: string;
  createdAt: string;
  updatedAt: string;
  files: Record<number, WorkContractFiles>;

  // Things made for the sake of performance that are added when the data is fetched
  dateStartString: string;
  dateEndString?: string;

  typeValue: number;

  __searchName: string;
  __searchType: string;
  __searchDescription?: string;
}

export type WorkContracts = Record<number, WorkContract>;
