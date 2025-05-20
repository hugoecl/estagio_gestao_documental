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
  try {
    // Always use FormData for consistency
    const formData = new FormData();
    
    // Debug image upload info
    
    // Add all standard fields
    formData.append('name', data.name);
    formData.append('path', data.path);
    formData.append('parent_path', data.parent_path || '');
    formData.append('is_group', data.is_group.toString());
    formData.append('description', data.description || '');
    formData.append('icon', data.icon || '');
    formData.append('icon_type', data.icon_type || '');
    formData.append('notify_on_new_record', data.notify_on_new_record.toString());
    formData.append('requires_acknowledgment', data.requires_acknowledgment.toString());
    
    // Add icon image if available
    if (data.icon_image) {
      formData.append('icon_image', data.icon_image);
      // Verify the file was added to FormData
    }
    
    // Add permissions and fields as JSON strings
    formData.append('permissions', JSON.stringify(data.permissions));
    formData.append('fields', JSON.stringify(data.fields));
    
    // Log all keys in the FormData
    const keys = [];
    formData.forEach((value, key) => {
      keys.push(key);
    });
    
    const response = await handleFetch(`${API_BASE_URL}/custom_pages`, {
      method: "POST",
      credentials: "include",
      body: formData,
    });
    
    const responseText = await response.text();
    
    if (response.ok) {
      const pageId = parseInt(responseText, 10);
      return { success: true, pageId };
    }
    
    return { success: false };
  } catch (error) {
    console.error("Error creating custom page:", error);
    return { success: false };
  }
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
  try {
    // Always use FormData for consistency
    const formData = new FormData();
    
    // Add all standard fields
    formData.append('name', data.name);
    if (data.parent_path !== undefined) {
      formData.append('parent_path', data.parent_path || '');
    }
    formData.append('description', data.description || '');
    formData.append('icon', data.icon || '');
    
    // Always include icon_type to ensure it's set correctly
    // If icon_type is null, it means we want to clear any existing icon
    if (data.icon_type === null) {
      // Explicitly set to null or empty to clear existing icon
      formData.append('icon_type', '');
      // Append a field to indicate we want to clear the image
      formData.append('clear_icon_image', 'true');
    } else {
      formData.append('icon_type', data.icon_type || '');
      // If icon_type is 'fontawesome' and we previously had an image, clear it
      if (data.icon_type === 'fontawesome') {
        formData.append('clear_icon_image', 'true');
      }
    }
    
    if (data.notify_on_new_record !== undefined) {
      formData.append('notify_on_new_record', data.notify_on_new_record.toString());
    }
    if (data.requires_acknowledgment !== undefined) {
      formData.append('requires_acknowledgment', data.requires_acknowledgment.toString());
    }
    
    // Add icon image if available
    if (data.icon_image) {
      formData.append('icon_image', data.icon_image);
    }
    
    const response = await handleFetch(
      `${API_BASE_URL}/custom_pages/${pageId}`,
      {
        method: "PUT",
        credentials: "include",
        body: formData,
      },
    );
    
    return response.ok;
  } catch (error) {
    console.error("Error updating custom page:", error);
    return false;
  }
}

export async function deleteCustomPage(pageId: number): Promise<boolean> {
  const response = await handleFetch(
    `${API_BASE_URL}/custom_pages/${pageId}`,
    {
      method: "DELETE",
      credentials: "include",
    },
  );
  // Backend returns 200 OK or 204 No Content on successful deletion
  return response.ok || response.status === 204;
}

// Add deleteCustomPage later

// Add function to reorder custom pages
export async function reorderPages(
  orders: { id: number; display_order: number }[]
): Promise<boolean> {
  const response = await handleFetch(`${API_BASE_URL}/custom_pages/reorder`, {
    method: "POST",
    credentials: "include",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ orders }),
  });

  return response.ok;
}
