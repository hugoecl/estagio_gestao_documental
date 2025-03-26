import { handleFetch } from "@api/fetch-handler";
import API_BASE_URL from "@api/base-url";
import {
  WorkContractTypes,
  type WorkContract,
  type WorkContractCategories,
  type WorkContractCategory,
  type WorkContracts,
} from "@lib/types/work-contracts";
import { toSearchString } from "@utils/search-utils";
import { DMYHMSToDate, DMYToDate } from "@utils/date-utils";
import { Locations } from "@lib/types/locations";

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
      entry.__searchName = toSearchString(entry.name);
      entry.__searchDescription = toSearchString(entry.description);

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

export async function getWorkContracts(): Promise<
  [WorkContracts, WorkContractCategories] | null
> {
  const [categories, response] = await Promise.all([
    await getWorkContractCategories(),
    await handleFetch(`${API_BASE_URL}/work-contracts`, {
      method: "GET",
      credentials: "include",
    }),
  ]);

  if (!response.ok || !categories) {
    return null;
  }

  if (response.ok) {
    const json = await response.json();
    const entries = Object.values(json) as WorkContract[];
    for (let i = 0, len = entries.length; i < len; i++) {
      const entry = entries[i];
      entry.dateStart = DMYToDate(entry.dateStartString);
      if (entry.dateEndString) {
        entry.dateEnd = DMYToDate(entry.dateEndString);
      }

      entry.typeValue = entry.type as unknown as number;
      entry.locationValue = entry.location as unknown as number;

      entry.type = WorkContractTypes[entry.type as unknown as number];

      entry.location = Locations[entry.location as unknown as number];

      entry.category = categories[entry.categoryId].name;

      entry.__searchEmployeeName = toSearchString(entry.employeeName);

      entry.__searchType = toSearchString(entry.type as string);

      entry.__searchLocation = toSearchString(entry.location as string);

      entry.__searchCategory = toSearchString(entry.category);

      if (entry.description) {
        entry.__searchDescription = toSearchString(entry.description);
      }

      const files = entry.files;
      for (const key in files) {
        files[key].name = files[key].path.split("/").at(-1)!;
      }
    }
    return [json, categories];
  }
  return null;
}

export async function updateWorkContract(
  workContractId: string,
  workContract: WorkContract
): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/work-contracts/${workContractId}`,
    {
      method: "PUT",
      credentials: "include",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        employee_name: workContract.employeeName,
        nif: workContract.nif,
        start_date: workContract.dateStartString,
        end_date: workContract.dateEndString,
        type_of_contract: workContract.typeValue,
        location: workContract.locationValue,
        category_id: workContract.categoryId,
        description: workContract.description,
      }),
    }
  );
  return response.ok;
}

export async function uploadWorkContractFiles(
  contractId: string,
  files: File[]
): Promise<[boolean, number]> {
  const formData = new FormData();
  for (let i = 0, len = files.length; i < len; i++) {
    const file = files[i];
    formData.append("files", file, `${file.name}_${file.size}`);
  }

  const response = await handleFetch(
    `${API_BASE_URL}/work-contracts/${contractId}/files`,
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

export async function deleteWorkContractFile(
  workContractId: string,
  fileId: string
): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/work-contracts/${workContractId}/files/${fileId}`,
    {
      method: "DELETE",
      credentials: "include",
    }
  );

  return response.ok;
}

export async function deleteWorkContract(
  workContractId: string
): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/work-contracts/${workContractId}`,
    {
      method: "DELETE",
      credentials: "include",
    }
  );

  return response.ok;
}

export async function uploadWorkContract(
  formData: FormData
): Promise<[boolean, number, number]> {
  const response = await handleFetch(`${API_BASE_URL}/work-contracts`, {
    method: "POST",
    credentials: "include",
    body: formData,
  });
  const [contractId, fileId] = (await response.text()).split(",");
  return [response.ok, parseInt(contractId, 10), parseInt(fileId, 10)];
}
