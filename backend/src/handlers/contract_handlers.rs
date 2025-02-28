use actix_web::{HttpResponse, Responder};

pub async fn get_contracts() -> impl Responder {
    HttpResponse::Ok().body("Getting contracts")
}
