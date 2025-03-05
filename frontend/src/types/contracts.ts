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
}
export type Contracts = Record<number, Contract>;
