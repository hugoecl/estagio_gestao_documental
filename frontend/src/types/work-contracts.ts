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
