use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationFunction {
    pub name: String,
    pub description: String,
}

pub fn get_available_validations() -> Vec<ValidationFunction> {
    vec![
        ValidationFunction {
            name: "nif".to_string(),
            description: "Valida um NIF português".to_string(),
        },
        ValidationFunction {
            name: "email".to_string(),
            description: "Valida um endereço de email".to_string(),
        },
    ]
}
