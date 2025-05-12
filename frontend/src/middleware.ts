import { defineMiddleware } from "astro:middleware";
import API_BASE_URL from "@api/base-url";

// Response type from backend /users/check
interface CheckAuthResponse {
  isAdmin: boolean;
  canManageThisPage?: boolean;
}

export const onRequest = defineMiddleware(async (context, next) => {
  const pathname = context.url.pathname;
  const cookie = context.request.headers.get("cookie") ?? undefined;

  // Ignore static assets etc.
  if (
    pathname.startsWith("/_image/") ||
    pathname.startsWith("/_astro/") ||
    pathname.startsWith("/assets/") ||
    pathname === "/favicon.ico"
  ) {
    return next();
  }

  const isLoginPage = pathname === "/iniciar-sessao/";
  const isRegisterPage = pathname === "/registo/";
  const isAdminRoute = pathname.startsWith("/admin/");
  const isAdminEditPageRoute = pathname.startsWith("/admin/pages/edit/");
  const isPublicAsset = pathname.startsWith("/media/"); // Assume backend handles auth for this too

  let isAuthenticated = false;
  let isSessionAdmin = false;
  let canManageCurrentPage = false; // Specific permission for edit route
  let checkPerformed = false;

  // Perform check only if necessary
  if (cookie || isAdminRoute) {
    // Check if cookie exists OR it's any admin route
    try {
      const response = await fetch(`${API_BASE_URL}/users/check`, {
        method: "POST",
        body: pathname,
        headers: { cookie: cookie, "Content-Type": "text/plain" },
      });
      checkPerformed = true;

      if (response.ok) {
        isAuthenticated = true;
        const data: CheckAuthResponse = await response.json();
        isSessionAdmin = data.isAdmin; // General admin status
        context.locals.isAdmin = isSessionAdmin;
        // If it was the edit page route, check the specific permission
        if (isAdminEditPageRoute) {
          canManageCurrentPage = data.canManageThisPage ?? false;
        }
      } else if (response.status === 401) {
        isAuthenticated = false;
        isSessionAdmin = false;
        canManageCurrentPage = false;
      } else {
        console.error(
          `Auth check failed with status ${response.status} for path: ${pathname}`,
        );
        isAuthenticated = false;
        isSessionAdmin = false;
        canManageCurrentPage = false;
      }
    } catch (error) {
      console.error(`Error during auth check fetch for ${pathname}:`, error);
      isAuthenticated = false;
      isSessionAdmin = false;
      canManageCurrentPage = false;
      checkPerformed = true;
    }
  }

  // --- Routing Logic ---

  // 1. Specific Admin Edit Page Route Protection
  if (isAdminEditPageRoute) {
    if (!checkPerformed && !cookie) return context.redirect("/iniciar-sessao/"); // SSR check
    if (!isAuthenticated) return context.redirect("/iniciar-sessao/"); // Must be logged in

    // Allow if EITHER general admin OR has specific page management permission
    if (!isSessionAdmin && !canManageCurrentPage) {
      console.warn(`User denied access to specific edit page: ${pathname}`);
      return context.redirect("/"); // Redirect non-admins/non-managers away
    }
    // If admin OR canManageThisPage, proceed
    return next();
  }

  // 2. Admin Route Protection (handles various /admin/ paths)
  if (isAdminRoute) {
    // This is only hit if it's NOT an edit page route (which is handled above)
    // All /admin/ routes (not covered by more specific checks above) require authentication.
    if (!checkPerformed && !cookie) return context.redirect("/iniciar-sessao/");
    if (!isAuthenticated) return context.redirect("/iniciar-sessao/");

    // Specific handling for /admin/records/.../acknowledgments
    // These routes rely on backend API permission checks (e.g., can_view_acknowledgments)
    // Normalize pathname to remove trailing slash for this specific check
    const normalizedPathname = pathname.endsWith("/")
      ? pathname.slice(0, -1)
      : pathname;
    const isAdminRecordAcksPage =
      normalizedPathname.startsWith("/admin/records/") &&
      normalizedPathname.endsWith("/acknowledgments");

    if (isAdminRecordAcksPage) {
      // For this specific page, being authenticated is enough for the middleware layer.
      // The backend API handler will perform the more fine-grained permission check.
      return next(); // Explicitly return after calling next()
    }
    // IMPORTANT: Only proceed to the general admin check if it's NOT the acknowledgments page
    else if (!isSessionAdmin) {
      // Check for all other /admin/ routes
      console.warn(
        `Non-admin user denied access to general admin route: ${pathname}. User isSessionAdmin: ${isSessionAdmin}`,
      );
      return context.redirect("/");
    }

    // If authenticated and isSessionAdmin (for general admin routes), or handled by specific conditions above.
    return next();
  }

  // 3. Public Assets (minimal check, rely mostly on backend)
  if (isPublicAsset) {
    if (!checkPerformed && !cookie) return context.redirect("/iniciar-sessao/");
    if (!isAuthenticated) return context.redirect("/iniciar-sessao/");
    return next();
  }

  // 4. Login/Register Page Logic
  if (isLoginPage || isRegisterPage) {
    if (isAuthenticated) return context.redirect("/");
    return next();
  }

  // 5. Default Protected Routes
  if (!checkPerformed && !cookie) {
    // SSR, no cookie -> Let client handle
    return next();
  }
  if (!isAuthenticated) {
    return context.redirect("/iniciar-sessao/");
  }

  // If authenticated, allow access
  return next();
});
