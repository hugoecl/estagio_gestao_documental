import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";
import type {
  FieldType,
  PageField,
  UpdatePageFieldRequest,
  CreatePageFieldRequest,
  ValidationFunction,
} from "@lib/types/fields";
import type { CreateCustomPageRequest } from "@lib/types/custom-page"; // Import if needed for addPageField context

// --- Field Types ---

export async function getFieldTypes(cookie?: string): Promise<FieldType[]> {
  const response = await handleFetch(`${API_BASE_URL}/fields/types`, {
    method: "GET",
    credentials: "include",
    headers: cookie ? { Cookie: cookie } : undefined,
  });
  if (response.ok) {
    return await response.json();
  }
  if (response.status === 304) return []; // Or cached
  throw new Error(`Failed to fetch field types: ${response.statusText}`);
}

// --- Page Fields ---
// Note: Backend routes for getting/adding fields are under /custom_pages/{page_id}/fields

export async function getPageFields(
  pageId: number,
  cookie?: string,
): Promise<PageField[]> {
  const response = await handleFetch(
    `${API_BASE_URL}/custom_pages/${pageId}/fields`,
    {
      method: "GET",
      credentials: "include",
      headers: cookie ? { Cookie: cookie } : undefined,
    },
  );
  if (response.ok) {
    return await response.json();
  }
  if (response.status === 304) return []; // Or cached
  throw new Error(
    `Failed to fetch fields for page ${pageId}: ${response.statusText}`,
  );
}

export async function addPageField(
  pageId: number,
  fieldData: CreatePageFieldRequest,
): Promise<{ success: boolean; fieldId?: number }> {
  const response = await handleFetch(
    `${API_BASE_URL}/custom_pages/${pageId}/fields`,
    {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(fieldData),
    },
  );
  if (response.ok) {
    const fieldIdText = await response.text(); // Backend returns the ID directly
    return { success: true, fieldId: parseInt(fieldIdText, 10) };
  }
  return { success: false };
}

// --- Individual Field Operations ---

export async function updateField(
  fieldId: number,
  fieldData: UpdatePageFieldRequest,
): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/fields/${fieldId}`, {
    method: "PUT",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(fieldData),
  });
  return response.ok;
}

// --- THIS IS THE MISSING EXPORT ---
export async function deleteField(fieldId: number): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/fields/${fieldId}`, {
    method: "DELETE",
    credentials: "include",
  });
  // Check for 200 OK or 204 No Content
  return response.ok || response.status === 204;
}

// --- Validations ---

export async function getValidations(
  cookie?: string,
): Promise<ValidationFunction[]> {
  const response = await handleFetch(`${API_BASE_URL}/fields/validations`, {
    method: "GET",
    credentials: "include",
    headers: cookie ? { Cookie: cookie } : undefined,
  });
  if (response.ok) {
    return await response.json();
  }
  if (response.status === 304) return []; // Or cached
  throw new Error(`Failed to fetch validations: ${response.statusText}`);
}
