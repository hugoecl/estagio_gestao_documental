import { handleFetch } from "@api/fetch-handler";
import API_BASE_URL from "@api/base-url";
import {
  ContractServices,
  ContractStatus,
  ContractTypes,
  type Contract,
  type Contracts,
} from "@lib/types/contracts";
import { Locations } from "@lib/types/locations";
import { toSearchString } from "@utils/search-utils";
import { DMYToDate } from "@utils/date-utils";

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

      entry.__searchSupplier = toSearchString(entry.supplier);

      entry.__searchLocation = toSearchString(entry.location as string);

      entry.__searchService = toSearchString(entry.service as string);

      entry.__searchType = toSearchString(entry.type as string);

      entry.__searchStatus = toSearchString(entry.status as string);

      entry.__searchContractNumber = toSearchString(
        entry.contractNumber.toString()
      );

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
