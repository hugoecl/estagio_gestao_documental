import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";
import type {
  PageRecord,
  PageRecordWithFiles,
  CreatePageRecordRequest,
  UpdatePageRecordRequest,
} from "@lib/types/page-record"; // Define these types later

export async function getPageRecords(
  pageId: number,
  searchQuery?: string,
  cookie?: string,
): Promise<PageRecord[]> {
  let url = `${API_BASE_URL}/records/pages/${pageId}/records`;
  if (searchQuery) {
    url += `?search=${encodeURIComponent(searchQuery)}`;
  }
  const response = await handleFetch(url, {
    method: "GET",
    credentials: "include",
    headers: cookie ? { Cookie: cookie } : undefined,
  });
  if (response.ok) {
    return await response.json();
  }
  if (response.status === 304) {
    console.warn(`Records for page ${pageId} not modified (304)`);
    return []; // Or return cached data
  }
  throw new Error(
    `Failed to fetch records for page ${pageId}: ${response.statusText}`,
  );
}

export async function getRecordById(
  recordId: number,
  cookie?: string,
): Promise<PageRecordWithFiles | null> {
  try {
    const response = await handleFetch(`${API_BASE_URL}/records/${recordId}`, {
      method: "GET",
      credentials: "include",
      headers: cookie ? { Cookie: cookie } : undefined,
    });
    if (response.ok) {
      return await response.json();
    }
    if (response.status === 404) {
      return null;
    }
    throw new Error(
      `Failed to fetch record ${recordId}: ${response.statusText}`,
    );
  } catch (error) {
    console.error(`Error fetching record ${recordId}:`, error);
    return null;
  }
}

export async function createRecord(
  pageId: number,
  data: CreatePageRecordRequest,
): Promise<{ success: boolean; recordId?: number }> {
  const response = await handleFetch(
    `${API_BASE_URL}/records/pages/${pageId}/records`,
    {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    },
  );
  if (response.ok) {
    const recordIdText = await response.text();
    return { success: true, recordId: parseInt(recordIdText, 10) };
  }
  return { success: false };
}

export async function updateRecord(
  recordId: number,
  data: UpdatePageRecordRequest,
): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/records/${recordId}`, {
    method: "PUT",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  return response.ok;
}

export async function deleteRecord(recordId: number): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/records/${recordId}`, {
    method: "DELETE",
    credentials: "include",
  });
  // Check for 204 No Content or 200 OK
  return response.ok || response.status === 204;
}

export async function uploadRecordFiles(
  recordId: number,
  files: File[],
): Promise<{ success: boolean; firstFileId?: number }> {
  if (files.length === 0) return { success: true };

  const formData = new FormData();
  files.forEach((file) => {
    // Backend expects filename_size format
    formData.append("files", file, `${file.name}_${file.size}`);
  });

  const response = await handleFetch(
    `${API_BASE_URL}/records/${recordId}/files`,
    {
      method: "POST",
      credentials: "include",
      body: formData,
    },
  );

  if (response.ok) {
    const firstFileIdText = await response.text();
    return { success: true, firstFileId: parseInt(firstFileIdText, 10) };
  }
  return { success: false };
}

export async function deleteRecordFile(
  recordId: number,
  fileId: number,
): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/records/${recordId}/files/${fileId}`,
    {
      method: "DELETE",
      credentials: "include",
    },
  );
  // Check for 204 No Content or 200 OK
  return response.ok || response.status === 204;
}
