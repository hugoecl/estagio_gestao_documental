export const ContractServices = [
  "Eletricidade",
  "Água",
  "Limpeza",
  "Impressoras",
  "Comunicações",
] as const;
export const ContractLocations = [
  "Viana do Castelo",
  "Braga",
  "Porto",
  "Vila Real",
] as const;
export const ContractStatus = ["Ativo", "Inativo"] as const;
export const ContractTypes = ["Adenda", "Novo", "Renovação"] as const;

// TODO: See createdAt and updatedAt
export interface ContractFiles {
  name: string;
  path: string;
  uploadedAt: string;
}
export interface Contract {
  contractNumber: number;
  date: Date;
  dateString: string;
  dateStart: Date;
  dateStartString: string;
  dateEnd: Date;
  dateEndString: string;
  description: string;
  location: (typeof ContractLocations)[number];
  locationValue: number;
  service: (typeof ContractServices)[number];
  serviceValue: number;
  status: (typeof ContractStatus)[number];
  statusValue: number;
  supplier: string;
  type: (typeof ContractTypes)[number];
  typeValue: number;
  createdAt: string;
  updatedAt: string;
  files: Record<number, ContractFiles>;

  // lowercase versions for performance reasons
  __searchSupplier: string;
  __searchLocation: string;
  __searchService: string;
  __searchContractNumber: string;
  __searchType: string;
  __searchStatus: string;
}
export type Contracts = Record<number, Contract>;
