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

// TODO: See createdAt and updatedAt maybe show them in the modal
export interface ContractFiles {
  name: string;
  path: string;
  uploadedAt: string;
}
export interface Contract {
  contractNumber: number;
  date: Date;
  dateStart: Date;
  dateEnd: Date;
  description: string;
  location: (typeof ContractLocations)[number];
  service: (typeof ContractServices)[number];
  status: (typeof ContractStatus)[number];
  supplier: string;
  type: (typeof ContractTypes)[number];
  createdAt: string;
  updatedAt: string;
  files: Record<number, ContractFiles>;

  // Things made for the sake of performance that are added when the data is fetched
  dateString: string;
  dateStartString: string;
  dateEndString: string;

  locationValue: number;
  serviceValue: number;
  statusValue: number;
  typeValue: number;

  __searchSupplier: string;
  __searchLocation: string;
  __searchService: string;
  __searchContractNumber: string;
  __searchType: string;
  __searchStatus: string;
}
export type Contracts = Record<number, Contract>;
