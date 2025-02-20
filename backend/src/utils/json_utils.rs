use actix_web::{web, HttpResponse};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Json<T>(pub T);

impl<T> Json<T>
where
    T: DeserializeOwned,
{
    pub fn from_bytes(bytes: web::Bytes) -> Result<Self, sonic_rs::Error> {
        let obj = unsafe { sonic_rs::from_slice_unchecked(&bytes).unwrap() }; // Assuming the bytes are valid JSON
        Ok(Json(obj))
    }
}

pub fn json_response<T>(obj: &T) -> HttpResponse
where
    T: Serialize,
{
    let json = sonic_rs::to_string(obj).unwrap();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body::MessageBody, test};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        name: String,
        age: u32,
    }

    #[test]
    async fn test_from_bytes() {
        let data = r#"{"name":"John Doe","age":30}"#;
        let bytes = web::Bytes::from(data);
        let json: Json<TestStruct> = Json::from_bytes(bytes).unwrap();
        assert_eq!(
            json.0,
            TestStruct {
                name: "John Doe".to_string(),
                age: 30
            }
        );
    }

    #[test]
    async fn test_json_response() {
        let obj = TestStruct {
            name: "John Doe".to_string(),
            age: 30,
        };
        let response = json_response(&obj);
        let body = response.into_body().try_into_bytes().unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        let expected_json = r#"{"name":"John Doe","age":30}"#;
        assert_eq!(body_str, expected_json);
    }
}
