use actix_multipart::form::{MultipartForm, text::Text};
use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use tokio::{fs, task::JoinSet};

use crate::{
    State,
    utils::{memory_file::MemoryFile, session_utils::validate_session},
};

pub async fn get_contracts() -> impl Responder {
    HttpResponse::Ok().body("Getting contracts")
}

#[derive(MultipartForm)]
pub struct ContractFormRequest {
    #[multipart(rename = "contract-number")]
    contract_number: Text<String>,
    date: Text<String>,
    #[multipart(rename = "date-range")]
    date_range: Text<String>,
    description: Text<String>,
    files: Vec<MemoryFile>,
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

    println!(
        "
        Contract number: {:?}
        Date: {:?}
        Date range: {:?}
        Description: {:?}
        Location: {:?}
        Service: {:?}
        Status: {:?}
        Supplier: {:?}
        Type of contract: {:?}",
        form.contract_number,
        form.date,
        form.date_range,
        form.description,
        form.location,
        form.service,
        form.status,
        form.supplier,
        form.type_of_contract
    );

    let mut set = JoinSet::new();
    for file in form.files.into_iter() {
        set.spawn(async move {
            fs::write(format!("media/contracts/{}", file.file_name), &file.data)
                .await
                .unwrap()
        });
    }

    set.join_all().await;

    HttpResponse::Ok().body("Uploading contract")
}
