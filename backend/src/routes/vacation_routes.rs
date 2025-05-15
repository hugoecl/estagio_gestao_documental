use actix_web::web;

use crate::handlers::vacation_handlers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/vacation-requests") // Base path for vacation-related requests by users
            .route("", web::post().to(vacation_handlers::submit_vacation_request))
            .route("/me", web::get().to(vacation_handlers::get_my_vacation_requests)),
    );
    // Route for fetching remaining vacation days, as per previous phase description for user-api.
    // Though semantically it could be under /vacation-requests/me/remaining-days,
    // keeping it under /users/me/ as it's a user-specific detail not directly a "request".
    // The handler `get_my_remaining_vacation_days` is in `vacation_handlers.rs`.
    // If we want to group it, we'd need a new scope or adjust existing /users scope.
    // For now, let's add it as a new distinct route as described.
    // This will require adding this init function to the main routes config.
    //
    // Correction: The user request mentioned this route as part of Phase 2:
    // "GET /api/users/me/vacation-days: Fetches the current user's remaining vacation days."
    // So, this route should actually be added to `user_routes.rs` to keep `/users/me/` consistent.
    //
    // However, since the handler `get_my_remaining_vacation_days` was just created in `vacation_handlers.rs`,
    // it's simpler for now to route it from `vacation_routes.rs` but give it the desired path.
    // Alternatively, move the handler. For now, route it here.
    cfg.route(
        "/users/me/vacation-days",
        web::get().to(vacation_handlers::get_my_remaining_vacation_days),
    );

    // Admin-specific vacation routes will be added later in a different scope, e.g., /admin/vacations
}