use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};

use crate::{State, utils::session_utils::validate_session};

pub async fn get_contracts() -> impl Responder {
    HttpResponse::Ok().body("Getting contracts")
}

#[derive(Debug, MultipartForm)]
pub struct ContractFormRequest {
    #[multipart(rename = "contract-number")]
    contract_number: Text<String>,
    date: Text<String>,
    #[multipart(rename = "date-range")]
    date_range: Text<String>,
    description: Text<String>,
    files: Vec<TempFile>,
    location: Text<String>,
    service: Text<String>,
    status: Text<i32>,
    supplier: Text<String>,
    #[multipart(rename = "type")]
    type_of_contract: Text<i32>,
}

pub async fn upload_contract(
    session: Session,
    state: web::Data<State>,
    MultipartForm(form): MultipartForm<ContractFormRequest>,
) -> impl Responder {
    if let Err(response) = validate_session(&session) {
        return response;
    }

    println!("\n{:?}", form);

    HttpResponse::Ok().body("Uploading contract")
}
