use crate::proto::cursed_archive::FileInfo;
use tonic::{Request, Response, Status};
pub mod cursed_archive {
    tonic::include_proto!("cursed_archive");
    pub mod types {
        tonic::include_proto!("cursed_archive");
    }
}

#[derive(Debug, Default)]
pub struct FileInjectServiceImpl {}

#[tonic::async_trait]
impl cursed_archive::file_inject_service_server::FileInjectService for FileInjectServiceImpl {
    async fn upload_file(
        &self,
        request: Request<cursed_archive::UploadFileRequest>,
    ) -> Result<Response<cursed_archive::UploadFileResponse>, Status> {
        let req = request.into_inner();
        let file = req
            .file
            .ok_or_else(|| Status::invalid_argument("Missing file"))?;

        // Create file info from the request
        let file_info = FileInfo {
            id: "temp-id".to_string(),
            name: file.file_path.clone(),
            original_path: file.file_path.clone(),
            stored_path: "temp-path".to_string(),
            size: file.file_data.len() as i64,
            extension: "".to_string(),
            base_name: "".to_string(),
            parent_directory: "".to_string(),
        };

        Ok(Response::new(cursed_archive::UploadFileResponse {
            file_info: Some(file_info),
        }))
    }
}

pub use cursed_archive::file_inject_service_server::FileInjectServiceServer;
pub use cursed_archive::{UploadFileRequest, UploadFileResponse};
