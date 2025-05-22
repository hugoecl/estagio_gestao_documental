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

    fn read_field(_: &'t HttpRequest, mut field: Field, _: &'t mut Limits) -> Self::Future {
        Box::pin(async move {
            let (file_name, file_size) = {
                let content_disp = field.content_disposition().unwrap();
                let original_filename = content_disp.get_filename().unwrap();
                
                // Find the last underscore which should separate the filename and size
                // But we need to be careful not to cut off file extensions
                let parts: Vec<&str> = original_filename.rsplitn(2, '_').collect();
                
                let (base_filename, size_part) = if parts.len() == 2 {
                    // If we found an underscore, use the part before it as filename
                    (parts[1].to_string(), parts[0])
                } else {
                    // If no underscore found, use the whole filename
                    (original_filename.to_string(), "0")
                };
                
                // Parse the size, defaulting to 0 if parsing fails
                let file_size = size_part.parse::<usize>().unwrap_or(0);

                (base_filename, file_size)
            };

            let mut data = Vec::with_capacity(file_size);

            while let Some(chunk) = field.try_next().await? {
                data.extend_from_slice(&chunk);
            }

            Ok(MemoryFile { data, file_name })
        })
    }
}
