use actix_multipart::form::MultipartForm;

use super::memory_file::MemoryFile;

#[derive(MultipartForm)]
pub struct FilesFormRequest {
    pub files: Vec<MemoryFile>,
}
