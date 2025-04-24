import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";
import type {
  FieldType,
  PageField,
  ValidationFunction,
  CreatePageFieldRequest,
  UpdatePageFieldRequest,
} from "@lib/types/fields"; // Define these types later

export async function getFieldTypes(cookie?: string): Promise<FieldType[]> {
  const response = await handleFetch(`${API_BASE_URL}/fields/types`, {
    method: "GET",
    credentials: "include",
    headers: cookie ? { Cookie: cookie } : undefined,
  });
  if (response.ok) {
    return await response.json();
  }
  if (response.status === 304) {
    console.warn("Field types not modified (304)");
    return [];
  }
  throw new Error(`Failed to fetch field types: ${response.statusText}`);
}

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
  if (response.status === 304) {
    console.warn(`Page fields for ${pageId} not modified (304)`);
    return [];
  }
  throw new Error(
    `Failed to fetch page fields for ${pageId}: ${response.statusText}`,
  );
}

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
  if (response.status === 304) {
    console.warn("Validations not modified (304)");
    return [];
  }
  throw new Error(`Failed to fetch validations: ${response.statusText}`);
}

// Add functions for add/update/delete fields if needed for admin UI
