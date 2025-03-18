import API_BASE_URL from "@api/base-url";

import { showAlert, AlertPosition, AlertType } from "@components/alert/alert";
import {
  ContractServices,
  ContractStatus,
  ContractTypes,
  type Contract,
  type Contracts,
} from "@lib/types/contracts";
import { Locations } from "@lib/types/locations";
import { toggleElements } from "src/stores/loading-stores";
import { DMYHMSToDate, DMYToDate } from "@utils/date-utils";
import type {
  WorkContractCategories,
  WorkContractCategory,
} from "@lib/types/work-contracts";

async function handleFetch(
  url: string | URL,
  options: RequestInit
): Promise<Response> {
  try {
    const response = await fetch(url, options);
    if (
      response.status === 401 &&
      window.location.pathname !== "/iniciar-sessao/" &&
      window.location.pathname !== "/registo/"
    ) {
      window.location.pathname = "/iniciar-sessao/";
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

/**
 * @returns boolean indicating if the request was successful the id of the contract created and the id of the first file uploaded
 */
export async function uploadContract(
  formData: FormData
): Promise<[boolean, number, number]> {
  const response = await handleFetch(`${API_BASE_URL}/contracts`, {
    method: "POST",
    credentials: "include",
    body: formData,
  });
  const [contractId, fileId] = (await response.text()).split(",");

  return [response.ok, parseInt(contractId, 10), parseInt(fileId, 10)];
}

interface ContractResponse
  extends Omit<Contract, "location" | "service" | "status" | "type"> {
  location: keyof typeof Locations;
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
      entry.location = Locations[entry.location] as keyof typeof Locations;
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

      entry.date = DMYToDate(entry.dateString);
      entry.dateStart = DMYToDate(entry.dateStartString);
      entry.dateEnd = DMYToDate(entry.dateEndString);

      entry.__searchSupplier = entry.supplier
        .normalize("NFKD")
        .replace(/[\u0300-\u036f]/g, "")
        .toLowerCase();
      entry.__searchLocation = (entry.location as string)
        .normalize("NFKD")
        .replace(/[\u0300-\u036f]/g, "")
        .toLowerCase();
      entry.__searchService = (entry.service as string)
        .normalize("NFKD")
        .replace(/[\u0300-\u036f]/g, "")
        .toLowerCase();
      entry.__searchType = (entry.type as string)
        .normalize("NFKD")
        .replace(/[\u0300-\u036f]/g, "")
        .toLowerCase();
      entry.__searchStatus = (entry.status as string)
        .normalize("NFKD")
        .replace(/[\u0300-\u036f]/g, "")
        .toLowerCase();
      entry.__searchContractNumber = entry.contractNumber
        .toString()
        .normalize("NFKD")
        .replace(/[\u0300-\u036f]/g, "");

      const files = entry.files;
      for (const key in files) {
        files[key].name = files[key].path.split("/").at(-1)!;
      }
    }
    return json;
  }
  return null;
}

/**
 * @returns boolean indicating if the request was successful and the id of the file uploaded
 */
export async function uploadContractFiles(
  contractId: string,
  files: File[]
): Promise<[boolean, number]> {
  const formData = new FormData();
  for (let i = 0, len = files.length; i < len; i++) {
    const file = files[i];
    formData.append("files", file, `${file.name}_${file.size}`);
  }

  const response = await handleFetch(
    `${API_BASE_URL}/contracts/${contractId}/files`,
    {
      method: "POST",
      credentials: "include",
      body: formData,
    }
  );
  if (!response.ok) {
    return [false, -1];
  }

  return [response.ok, parseInt(await response.text(), 10)];
}

export async function deleteContractFile(
  contractId: string,
  fileId: string
): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/contracts/${contractId}/files/${fileId}`,
    {
      method: "DELETE",
      credentials: "include",
    }
  );

  return response.ok;
}

export async function deleteContract(contractId: string): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/contracts/${contractId}`,
    {
      method: "DELETE",
      credentials: "include",
    }
  );

  return response.ok;
}

export async function updateContract(
  contractId: string,
  contract: Contract
): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/contracts/${contractId}`,
    {
      method: "PUT",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        contract_number: contract.contractNumber,
        date: contract.dateString,
        date_start: contract.dateStartString,
        date_end: contract.dateEndString,
        description: contract.description,
        location: contract.locationValue,
        service: contract.serviceValue,
        status: contract.statusValue,
        supplier: contract.supplier,
        type_of_contract: contract.typeValue,
      }),
    }
  );

  return response.ok;
}

export async function getAnalytics(): Promise<PageAnalytics> {
  const response = await handleFetch(`${API_BASE_URL}/users/analytics`, {
    method: "GET",
    credentials: "include",
  });

  if (response.ok) {
    return (await response.json()) as PageAnalytics;
  }
  return [];
}

export async function getWorkContractCategories(): Promise<WorkContractCategories | null> {
  const response = await handleFetch(
    `${API_BASE_URL}/work-contracts/categories`,
    {
      method: "GET",
      credentials: "include",
    }
  );

  if (response.ok) {
    const json = await response.json();
    const entries = Object.values(json) as WorkContractCategory[];
    for (let i = 0, len = entries.length; i < len; i++) {
      const entry = entries[i];
      entry.__searchName = entry.name
        .normalize("NFKD")
        .replace(/[\u0300-\u036f]/g, "")
        .toLowerCase();
      entry.__searchDescription = entry.description
        .normalize("NFKD")
        .replace(/[\u0300-\u036f]/g, "")
        .toLowerCase();

      entry.__createdAtDate = DMYHMSToDate(entry.createdAt);
      entry.__updatedAtDate = DMYHMSToDate(entry.updatedAt);
    }
    return json;
  }
  return null;
}

export async function updateWorkContractCategory(
  categoryId: string,
  category: WorkContractCategory
): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/work-contracts/categories/${categoryId}`,
    {
      method: "PUT",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        name: category.name,
        description: category.description,
      }),
    }
  );
  return response.ok;
}

export async function deleteWorkContractCategory(
  categoryId: string
): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/work-contracts/categories/${categoryId}`,
    {
      method: "DELETE",
      credentials: "include",
    }
  );
  return response.ok;
}

export async function uploadWorkContractCategory(
  name: string,
  description: string
): Promise<[boolean, number]> {
  const response = await handleFetch(
    `${API_BASE_URL}/work-contracts/categories`,
    {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        name,
        description,
      }),
    }
  );
  const categoryId = parseInt(await response.text(), 10);
  return [response.ok, categoryId];
}
