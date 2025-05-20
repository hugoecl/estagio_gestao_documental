import type { PageField, CreatePageFieldRequest } from "@lib/types/fields";

export interface CustomPage {
  id: number;
  name: string;
  path: string;
  parent_path: string | null;
  is_group: boolean;
  description: string | null;
  icon: string | null;
  icon_type: 'fontawesome' | 'image' | null;
  icon_image_path: string | null;
  created_at: string; // Consider using Date objects after fetching
  updated_at: string; // Consider using Date objects after fetching
}

export interface RolePermissionRequest {
  role_id: number;
  can_view: boolean;
  can_create: boolean;
  can_edit: boolean;
  can_delete: boolean;
  can_manage_fields: boolean;
  can_view_acknowledgments: boolean;
  can_add: boolean;
}

// export interface CreatePageFieldRequest {
//   name: string;
//   display_name: string;
//   field_type_id: number;
//   required: boolean;
//   options: any | null; // JSON value
//   validation_name: string | null;
//   is_searchable: boolean;
//   is_displayed_in_table: boolean;
//   order_index: number;
// }

export interface CreateCustomPageRequest {
  name: string;
  path: string;
  parent_path: string | null;
  is_group: boolean;
  description: string | null;
  icon: string | null;
  icon_type: 'fontawesome' | 'image' | null;
  icon_image?: File | null;
  notify_on_new_record: boolean;
  requires_acknowledgment: boolean;
  fields: CreatePageFieldRequest[]; // Empty if is_group is true
  permissions: RolePermissionRequest[]; // Empty if is_group is true
}

export interface UpdateCustomPageRequest {
  name: string;
  parent_path?: string | null;
  description: string | null;
  icon: string | null;
  icon_type?: 'fontawesome' | 'image' | null;
  icon_image?: File | null;
  notify_on_new_record?: boolean;
  requires_acknowledgment?: boolean;
}

export interface PagePermission {
  id: number;
  page_id: number;
  role_id: number;
  role_name: string;
  can_view: boolean;
  can_create: boolean;
  can_edit: boolean;
  can_delete: boolean;
  can_manage_fields: boolean;
  can_view_acknowledgments?: boolean;
  can_add: boolean; // New permission: can add to empty fields without editing filled ones
}

// Represents the user's specific permissions for the current page
// This might need to be derived or fetched separately if not included in CustomPageWithFields
export interface UserPagePermissions {
  can_view: boolean;
  can_create: boolean;
  can_edit: boolean;
  can_delete: boolean;
  can_manage_fields: boolean;
  can_view_acknowledgments?: boolean;
  can_add: boolean; // New permission: can add to empty fields without editing filled ones
  is_admin: boolean; // Useful shortcut
}

export interface CustomPageWithFields {
  page: CustomPage;
  fields: PageField[]; // Use PageField from fields.ts
  permissions: PagePermission[]; // Permissions for *all* roles
  // We might need to add the current user's specific permissions here
  currentUserPermissions?: UserPagePermissions | null;
}

// Matches backend NavigationItem
export interface NavigationItem {
  title: string;
  path: string | null; // The actual href path for pages, null for groups
  id: number; // ID is now included for admin reordering
  icon: string | null;
  icon_type: 'fontawesome' | 'image' | null;
  icon_image_path: string | null;
  display_order: number; // Display order for menu items
  children: NavigationItem[];
}
