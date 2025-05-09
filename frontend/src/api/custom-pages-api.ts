import API_BASE_URL from "@api/base-url";
import { handleFetch } from "@api/fetch-handler";
import type {
  NavigationItem,
  CustomPage,
  CustomPageWithFields,
  PagePermission,
  RolePermissionRequest,
  CreateCustomPageRequest,
  UpdateCustomPageRequest,
} from "@lib/types/custom-page"; // Define these types later

export async function getNavigationMenu(
  cookie?: string,
): Promise<NavigationItem[]> {
  const response = await handleFetch(`${API_BASE_URL}/custom_pages/menu`, {
    method: "GET",
    credentials: "include",
    headers: cookie ? { Cookie: cookie } : undefined,
  });
  if (response.ok) {
    return await response.json();
  }
  if (response.status === 304) {
    // Not Modified
    // Handle caching if necessary, or just return empty/previous data
    console.warn("Navigation menu not modified (304)");
    return []; // Or return cached data if available
  }
  throw new Error(`Failed to fetch navigation menu: ${response.statusText}`);
}

// Add other functions as needed: getCustomPages, getCustomPageById, getCustomPageByPath, createCustomPage, updateCustomPage, deleteCustomPage, updatePagePermissions
// Example:
export async function getCustomPageByPath(
  path: string,
  cookie?: string,
): Promise<CustomPageWithFields | null> {
  // NOTE: The backend doesn't have a direct /custom_pages/by-path/{path} endpoint.
  // Option 1: Add that endpoint to the backend (Recommended)
  // Option 2: Fetch all pages and filter client-side (Less efficient, requires admin/sufficient permissions)
  // Option 3: Modify backend /custom_pages to accept a 'path' query parameter.

  // Assuming Option 1 or 3 will be implemented. Let's pretend there's an endpoint:
  // Replace with actual endpoint logic once backend is updated
  if (path.endsWith("/")) {
    path = path.slice(0, -1);
  }
  try {
    const response = await handleFetch(
      `${API_BASE_URL}/custom_pages/by-path/${path}`,
      {
        // Fictional endpoint
        method: "GET",
        credentials: "include",
        headers: cookie ? { Cookie: cookie } : undefined,
      },
    );
    if (response.ok) {
      return await response.json();
    }
    if (response.status === 404) {
      return null;
    }
    throw new Error(
      `Failed to fetch page definition for path ${path}: ${response.statusText}`,
    );
  } catch (error) {
    console.error("Error fetching page by path:", error);
    return null; // Or re-throw
  }
}

export async function getCustomPageById(
  pageId: number,
  cookie?: string,
): Promise<CustomPageWithFields | null> {
  try {
    const response = await handleFetch(
      `${API_BASE_URL}/custom_pages/${pageId}`,
      {
        method: "GET",
        credentials: "include",
        headers: cookie ? { Cookie: cookie } : undefined,
      },
    );
    if (response.ok) {
      return await response.json();
    }
    if (response.status === 404) {
      return null;
    }
    throw new Error(
      `Failed to fetch page definition for ID ${pageId}: ${response.statusText}`,
    );
  } catch (error) {
    console.error("Error fetching page by ID:", error);
    return null;
  }
}

export async function getCustomPages(cookie?: string): Promise<CustomPage[]> {
  const response = await handleFetch(`${API_BASE_URL}/custom_pages`, {
    method: "GET",
    credentials: "include",
    headers: cookie ? { Cookie: cookie } : undefined,
  });
  if (response.ok) {
    return await response.json();
  }
  if (response.status === 304) return [];
  throw new Error(`Failed to fetch custom pages: ${response.statusText}`);
}

export async function createCustomPage(
  data: CreateCustomPageRequest,
): Promise<{ success: boolean; pageId?: number }> {
  if (data.path.length > 1 && data.path.endsWith("/")) {
    data.path = data.path.slice(0, -1);
  }

  const response = await handleFetch(`${API_BASE_URL}/custom_pages`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });

  if (response.ok) {
    const pageIdText = await response.text();
    return { success: true, pageId: parseInt(pageIdText, 10) };
  }
  return { success: false };
}

export async function updatePagePermissions(
  pageId: number,
  permissions: RolePermissionRequest[],
): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/custom_pages/${pageId}/permissions`,
    {
      method: "PUT",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(permissions),
    },
  );
  return response.ok;
}

// Add updateCustomPage, deleteCustomPage later if needed for edit/delete functionality

export async function getGroupPages(cookie?: string): Promise<CustomPage[]> {
  const response = await handleFetch(`${API_BASE_URL}/custom_pages/groups`, {
    method: "GET",
    credentials: "include",
    headers: cookie ? { Cookie: cookie } : undefined,
  });
  if (response.ok) {
    return await response.json();
  }
  if (response.status === 304) return []; // Or cached
  throw new Error(`Failed to fetch group pages: ${response.statusText}`);
}

export async function updateCustomPage(
  pageId: number,
  data: UpdateCustomPageRequest,
): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/custom_pages/${pageId}`, {
    method: "PUT",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  return response.ok;
}

// Add deleteCustomPage later
