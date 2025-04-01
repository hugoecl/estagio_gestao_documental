use actix_web::{HttpResponse, Responder};

pub async fn get_models() -> impl Responder {
    HttpResponse::Ok().body("Hello Models")
}
