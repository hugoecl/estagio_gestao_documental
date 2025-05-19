use serde::{Deserialize, Serialize};

// Add this new struct for the reorder request
#[derive(Debug, Deserialize)]
struct ReorderPagesRequest {
    orders: Vec<PageOrder>,
}

#[derive(Debug, Deserialize)]
struct PageOrder {
    id: u32,
    display_order: u32,
}

// Add new endpoint for reordering pages
#[post("/custom_pages/reorder")]
pub async fn reorder_pages(
    db: web::Data<DbPool>,
    auth_session: AuthSession,
    req: web::Json<ReorderPagesRequest>,
) -> impl Responder {
    // Check if user is admin
    if let Some(user_id) = auth_session.user_id() {
        if !auth::is_admin(&db, user_id.0).await.unwrap_or(false) {
            return HttpResponse::Forbidden().json(json!({"error": "Admin access required"}));
        }
        
        // Convert the orders to the format expected by update_multiple_display_orders
        let orders: Vec<(u32, u32)> = req.orders
            .iter()
            .map(|order| (order.id, order.display_order))
            .collect();
            
        // Update the display orders
        match CustomPage::update_multiple_display_orders(&db, &orders).await {
            Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
            Err(e) => {
                eprintln!("Error updating page orders: {}", e);
                HttpResponse::InternalServerError().json(json!({"error": "Failed to update page orders"}))
            }
        }
    } else {
        HttpResponse::Unauthorized().json(json!({"error": "Authentication required"}))
    }
} 