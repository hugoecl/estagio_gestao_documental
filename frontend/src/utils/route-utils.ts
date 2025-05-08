import type { MenuItem } from "@lib/types/menu"; // Keep this type

// Remove or comment out the large menuItems array
// export const menuItems: MenuItem[] = [ ... ];
export const menuItems: MenuItem[] = []; // Keep as empty for now

export function extractPathsAndTitles(
  items: MenuItem[],
  basePath: string = "",
): Record<string, string> {
  let result: Record<string, string> = {};

  for (const item of items) {
    let currentItemPath: string | undefined;

    // Determine the full path for the current item
    if (item.path) {
      currentItemPath = item.path.startsWith("/")
        ? item.path
        : basePath + item.path;
      if (currentItemPath !== "/") {
        // Ensure trailing slash for consistency if not present
        const normalizedPath = currentItemPath.endsWith("/")
          ? currentItemPath
          : `${currentItemPath}/`;
        result[normalizedPath] = item.title;
      }
    }

    // Recursively process children
    if (item.children) {
      // Determine the base path for children
      let childBasePath = basePath; // Default to parent's base path
      if (item.parentPath) {
        // If parentPath is defined, use it to construct the new base path
        childBasePath = item.parentPath.startsWith("/")
          ? item.parentPath + "/"
          : basePath + item.parentPath + "/";
      } else if (currentItemPath) {
        // If no parentPath but the current item had a path, use that
        // This case might be less common if parentPath is always used for nesting
        childBasePath = currentItemPath.endsWith("/")
          ? currentItemPath
          : `${currentItemPath}/`;
      }
      // Ensure childBasePath ends with a slash
      if (!childBasePath.endsWith("/")) {
        childBasePath += "/";
      }

      const childPaths = extractPathsAndTitles(item.children, childBasePath);
      result = { ...result, ...childPaths };
    }
  }
  return result;
}

// pathNames will be populated dynamically later or fetched alongside the menu
export let pathNames: Record<string, string> = {};

// Function to update pathNames dynamically if needed
export function updatePathNames(items: MenuItem[]) {
  pathNames = extractPathsAndTitles(items);
}
