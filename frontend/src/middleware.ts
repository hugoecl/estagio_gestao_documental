import { defineMiddleware } from "astro:middleware";
import API_BASE_URL from "@api/base-url";

export const onRequest = defineMiddleware(async (context, next) => {
  const pathname = context.url.pathname; // Use context.url.pathname
  const cookie = context.request.headers.get("cookie") ?? undefined; // Get cookie

  // Ignore static assets and internal routes
  if (
    pathname.startsWith("/_image/") ||
    pathname.startsWith("/_astro/") ||
    pathname.startsWith("/assets/") || // Add any other asset paths
    pathname === "/favicon.ico"
  ) {
    return next();
  }

  // --- Define Routes ---
  const isLoginPage = pathname === "/iniciar-sessao/";
  const isRegisterPage = pathname === "/registo/";
  const isAdminRoute = pathname.startsWith("/admin/");
  const isPublicAsset = pathname.startsWith("/media/"); // Assuming media is public *after* auth check

  // --- Server-Side Rendering (SSR) vs. Client-Side Navigation ---
  // Astro's `context.request.headers.get('x-astro-request')` might indicate client-side nav,
  // but a simpler check is often sufficient. If a cookie exists, it *might* be valid.
  // The crucial part is how we handle the backend response.

  // --- Authentication Check ---
  let isAuthenticated = false;
  let isAdmin = false;
  let checkPerformed = false;

  // Perform check only if necessary:
  // - Always check on client-side navigation (cookie likely present)
  // - Only check on server-side for /admin/ routes to enforce immediate security
  // - Skip server-side check for non-admin routes to avoid 401 noise
  if (cookie || isAdminRoute) {
    // Check if cookie exists OR it's an admin route
    try {
      const response = await fetch(`${API_BASE_URL}/users/check`, {
        method: "POST",
        body: pathname, // Send the path for analytics
        headers: {
          cookie: cookie, // Pass cookie if available
          "Content-Type": "text/plain",
        },
        // credentials: 'include', // Not needed server-side, header is explicit
      });
      checkPerformed = true; // Mark that we attempted the check

      if (response.ok) {
        isAuthenticated = true;
        const data: { isAdmin: boolean } = await response.json();
        isAdmin = data.isAdmin;
      } else if (response.status === 401) {
        isAuthenticated = false;
        isAdmin = false;
        // Log the 401 only if it's unexpected (e.g., during client nav)
        // if (context.request.headers.get('x-astro-request')) { // Example check for client nav
        // console.warn(`Auth check failed (401) for path: ${pathname}`);
        // }
      } else {
        // Handle other potential errors from the check endpoint
        console.error(
          `Auth check failed with status ${response.status} for path: ${pathname}`,
        );
        // Decide how to handle - potentially treat as unauthenticated?
        isAuthenticated = false;
        isAdmin = false;
      }
    } catch (error) {
      console.error(`Error during auth check fetch for ${pathname}:`, error);
      // Treat as unauthenticated on fetch error
      isAuthenticated = false;
      isAdmin = false;
      checkPerformed = true; // Mark check as attempted
    }
  }

  // --- Routing Logic ---

  // 1. Admin Route Protection
  if (isAdminRoute) {
    if (!checkPerformed && !cookie) {
      // SSR, no cookie, trying to access /admin -> redirect to login
      return context.redirect("/iniciar-sessao/");
    }
    if (!isAuthenticated) {
      // Failed check or 401 -> redirect to login
      return context.redirect("/iniciar-sessao/");
    }
    if (!isAdmin) {
      // Authenticated but not admin -> redirect to home (or 403 page)
      console.warn(`Non-admin user denied access to: ${pathname}`);
      return context.redirect("/"); // Or context.redirect('/403');
    }
    // If authenticated and admin, proceed
    return next();
  }

  // 2. Public Assets (already authenticated users can access)
  // Note: The backend /media endpoint should also re-validate the session
  if (isPublicAsset) {
    if (!checkPerformed && !cookie) {
      // SSR, no cookie, trying to access /media -> redirect to login (or let backend handle 401)
      return context.redirect("/iniciar-sessao/");
    }
    if (!isAuthenticated) {
      // Failed check or 401 -> redirect to login (or let backend handle 401)
      return context.redirect("/iniciar-sessao/");
    }
    // If authenticated, allow access
    return next();
  }

  // 3. Login/Register Page Logic
  if (isLoginPage || isRegisterPage) {
    if (isAuthenticated) {
      // If already logged in, redirect away from login/register
      return context.redirect("/");
    }
    // If not logged in, allow access
    return next();
  }

  // 4. Default Protected Routes (all others)
  if (!checkPerformed && !cookie) {
    // SSR, no cookie, trying to access protected route -> Let client handle redirect
    return next(); // Render the page, client-side JS will redirect if needed
    // Alternatively, redirect immediately: return context.redirect('/iniciar-sessao/');
  }
  if (!isAuthenticated) {
    // Failed check or 401 -> redirect to login
    return context.redirect("/iniciar-sessao/");
  }

  // If authenticated, allow access to the requested page
  return next();
});
