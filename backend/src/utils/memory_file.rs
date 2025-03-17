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
                let mut last_underscore_index = 0;
                for (i, c) in original_filename.chars().enumerate() {
                    if c == '_' {
                        last_underscore_index = i;
                    }
                }
                let file_name = original_filename[..last_underscore_index].to_string();
                let file_size = original_filename[last_underscore_index + 1..]
                    .parse::<usize>()
                    .unwrap_or(0);

                (file_name, file_size)
            };

            let mut data = Vec::with_capacity(file_size);

            while let Some(chunk) = field.try_next().await? {
                data.extend_from_slice(&chunk);
            }

            Ok(MemoryFile { data, file_name })
        })
    }
}
