use actix_multipart::{
    Field, MultipartError,
    form::{FieldReader, Limits},
};
use actix_web::HttpRequest;
use futures_core::future::LocalBoxFuture;
use futures_util::TryStreamExt;

pub struct MemoryFile {
    pub data: Vec<u8>,
    pub file_name: String,
}

impl<'t> FieldReader<'t> for MemoryFile {
    type Future = LocalBoxFuture<'t, Result<Self, MultipartError>>;

    fn read_field(_: &'t HttpRequest, mut field: Field, limits: &'t mut Limits) -> Self::Future {
        Box::pin(async move {
            let mut data = Vec::new();

            while let Some(chunk) = field.try_next().await? {
                limits.try_consume_limits(chunk.len(), false)?;
                data.extend_from_slice(chunk.as_ref());
            }

            Ok(MemoryFile {
                data,
                file_name: field
                    .content_disposition()
                    .unwrap()
                    .get_filename()
                    .unwrap()
                    .to_string(),
            })
        })
    }
}
