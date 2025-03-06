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
  const response = await handleFetch(`${API_BASE_URL}/contracts/upload`, {
    method: "POST",
    credentials: "include",
    body: formData,
  });
  return response.ok;
}

interface ContractResponse
  extends Omit<Contract, "location" | "service" | "status" | "type"> {
  location: keyof typeof ContractLocations;
  service: keyof typeof ContractServices;
  status: keyof typeof ContractStatus;
  type: keyof typeof ContractTypes;
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
      entries[i].location = ContractLocations[
        entries[i].location
      ] as keyof typeof ContractLocations;
      entries[i].service = ContractServices[
        entries[i].service
      ] as keyof typeof ContractServices;
      entries[i].status = ContractStatus[
        entries[i].status
      ] as keyof typeof ContractStatus;
      entries[i].type = ContractTypes[
        entries[i].type
      ] as keyof typeof ContractTypes;
      entries[i].date = new Date(entries[i].dateString);
      entries[i].dateStart = new Date(entries[i].dateStartString);
      entries[i].dateEnd = new Date(entries[i].dateEndString);

      entries[i].__searchSupplier = entries[i].supplier.toLowerCase();
      entries[i].__searchLocation = (
        entries[i].location as string
      ).toLowerCase();
      entries[i].__searchService = (entries[i].service as string).toLowerCase();
      entries[i].__searchType = (entries[i].type as string).toLowerCase();
      entries[i].__searchStatus = (entries[i].status as string).toLowerCase();
      entries[i].__searchContractNumber = entries[i].contractNumber.toString();

      const files = entries[i].files;
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
