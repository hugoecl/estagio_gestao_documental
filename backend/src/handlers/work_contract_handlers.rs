use actix_web::{HttpResponse, Responder};

pub async fn get() -> impl Responder {
    HttpResponse::Ok().body("Hello from work_contracts")
}
