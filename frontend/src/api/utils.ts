import API_BASE_URL from "@api/base-url";

import { showAlert, AlertPosition, AlertType } from "@components/Alert/Alert";
import {
  ContractLocations,
  ContractServices,
  ContractStatus,
  ContractTypes,
  type Contract,
  type Contracts,
} from "@lib/types/contracts";
import { toggleElements } from "src/stores/loadingStores";

async function handleFetch(
  url: string | URL,
  options: RequestInit
): Promise<Response> {
  try {
    const response = await fetch(url, options);
    if (
      window.location.pathname !== "/iniciar-sessao" &&
      window.location.pathname !== "registo" &&
      response.status === 401
    ) {
      window.location.href = "/iniciar-sessao";
    }
    return response;
  } catch (error) {
    toggleElements();

    showAlert(
      "Erro ao comunicar com o servidor",
      AlertType.ERROR,
      AlertPosition.TOP
    );
    throw error;
  }
}

export async function registerUser(
  username: string,
  email: string,
  password: string
): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/users/register`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ username, email, password }),
  });
  return response.ok;
}

export async function loginUser(
  email: string,
  password: string
): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/users/login`, {
    method: "POST",
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ email, password }),
  });

  return response.ok;
}

export async function logoutUser(): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/users/logout`, {
    method: "POST",
    credentials: "include",
  });
  return response.ok;
}

export async function uploadContract(formData: FormData): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/contracts`, {
    method: "POST",
    credentials: "include",
    body: formData,
  });
  return response.ok;
}

interface ContractResponse
  extends Omit<Contract, "location" | "service" | "status" | "type"> {
  location: keyof typeof ContractLocations;
  locationValue: number;
  service: keyof typeof ContractServices;
  serviceValue: number;
  status: keyof typeof ContractStatus;
  statusValue: number;
  type: keyof typeof ContractTypes;
  typeValue: number;
  dateString: string;
  dateStartString: string;
  dateEndString: string;
  // lowercase versions for not having to convert them to string/lowercase on each search
  __searchSupplier: string;
  __searchLocation: string;
  __searchService: string;
  __searchType: string;
  __searchStatus: string;
  __searchContractNumber: string;
}

export async function getContracts(): Promise<Contracts | null> {
  const response = await handleFetch(`${API_BASE_URL}/contracts`, {
    method: "GET",
    credentials: "include",
  });

  if (response.ok) {
    const json = await response.json();
    const entries = Object.values(json) as ContractResponse[];
    for (let i = 0, len = entries.length; i < len; i++) {
      const entry = entries[i];
      entry.locationValue = entry.location as number;
      entry.location = ContractLocations[
        entry.location
      ] as keyof typeof ContractLocations;
      entry.serviceValue = entry.service as number;
      entry.service = ContractServices[
        entry.service
      ] as keyof typeof ContractServices;
      entry.statusValue = entry.status as number;
      entry.status = ContractStatus[
        entry.status
      ] as keyof typeof ContractStatus;
      entry.typeValue = entry.type as number;
      entry.type = ContractTypes[entry.type] as keyof typeof ContractTypes;
      entry.date = new Date(entry.dateString);
      entry.dateStart = new Date(entry.dateStartString);
      entry.dateEnd = new Date(entry.dateEndString);

      entry.__searchSupplier = entry.supplier.toLowerCase();
      entry.__searchLocation = (entry.location as string).toLowerCase();
      entry.__searchService = (entry.service as string).toLowerCase();
      entry.__searchType = (entry.type as string).toLowerCase();
      entry.__searchStatus = (entry.status as string).toLowerCase();
      entry.__searchContractNumber = entry.contractNumber.toString();

      const files = entry.files;
      for (const key in files) {
        files[key].name = files[key].path.split("/").at(-1)!;
      }
    }
    return json;
  }
  return null;
}

export async function uploadContractFiles(
  contractId: string,
  files: File[]
): Promise<boolean> {
  return false;
}

export async function deleteContractFile(
  contractId: string,
  fileId: string
): Promise<boolean> {
  return false;
}

export async function deleteContract(contractId: string): Promise<boolean> {
  return false;
}

export async function updateContract(
  contractId: string,
  contract: Contract
): Promise<boolean> {
  return false;
}
