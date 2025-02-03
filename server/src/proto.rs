use crate::proto::cursed_archive::FileInfo;
use futures::Stream;
use log::{debug, error, info};
use std::{path::PathBuf, pin::Pin};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
pub mod cursed_archive {
    tonic::include_proto!("cursed_archive");
    pub mod types {
        tonic::include_proto!("cursed_archive");
    }
}
use tokio::io::AsyncWriteExt;
use cursed_archive::{FileChunk, UploadProgress};

const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks

#[derive(Debug, Default)]
pub struct FileInjectServiceImpl {}

#[tonic::async_trait]
impl cursed_archive::file_inject_service_server::FileInjectService for FileInjectServiceImpl {
    type UploadFileStream = Pin<Box<dyn Stream<Item = Result<UploadProgress, Status>> + Send + 'static>>;

    async fn upload_file(
        &self,
        request: Request<Streaming<FileChunk>>,
    ) -> Result<Response<Self::UploadFileStream>, Status> {
        let mut stream = request.into_inner();
        info!("Uploading file: {:#?}", stream);
        // Get the first chunk which contains file metadata
        let first_chunk = stream.message().await?
            .ok_or_else(|| Status::invalid_argument("Empty stream"))?;
            
        let file_path = PathBuf::from(first_chunk.file_path);
        let total_size = first_chunk.total_size;
        
        // Create file info
        let file_info = FileInfo {
            id: "temp-id".to_string(),
            name: file_path.file_name().unwrap().to_string_lossy().to_string(),
            original_path: file_path.to_string_lossy().to_string(),
            stored_path: "temp-path".to_string(),
            size: total_size as i64,
            extension: file_path.extension().unwrap().to_string_lossy().to_string(),
            base_name: file_path.file_name().unwrap().to_string_lossy().to_string(),
            parent_directory: file_path.parent().unwrap().to_string_lossy().to_string(),
        };

        debug!("File info: {:#?}", file_info);
        // Create channel for sending progress updates
        let (tx, rx) = mpsc::channel(128);
        let output_stream = ReceiverStream::new(rx);
        let file_info_clone = file_info.clone();
        // Process chunks in background task
        tokio::spawn(async move {
            let mut bytes_received = first_chunk.chunk_data.len() as u64;
            
            // Send first progress update
            let _ = tx.send(Ok(UploadProgress {
                file_info: Some(file_info_clone.clone()),
                bytes_received,
                total_size,
                complete: false,
            })).await;
            debug!("Sent first progress update");
            let storage_path = std::path::Path::new("uploads");
            if !storage_path.exists() {
                tokio::fs::create_dir_all(storage_path).await.map_err(|e| {
                    error!("Failed to create storage directory: {}", e);
                    Status::internal(format!("Failed to create storage directory: {}", e))
                }).unwrap();
            }

            // Append chunk data to file
            let storage_path_full = storage_path.join(&file_info_clone.name);
            info!("Storage path: {:#?}", storage_path_full);
            // Write first chunk data to file
            tokio::fs::write(&storage_path_full, &first_chunk.chunk_data)
                .await
                .map_err(|e| {
                    error!("Failed to write first chunk: {}", e);
                    Status::internal(format!("Failed to write first chunk: {}", e))
                }).unwrap();

            // Process remaining chunks
            while let Ok(Some(chunk)) = stream.message().await {
                bytes_received += chunk.chunk_data.len() as u64;

                // Append chunk data to file
                tokio::fs::OpenOptions::new()
                    .append(true)
                    .open(&storage_path_full)
                    .await
                    .map_err(|e| {
                        error!("Failed to open file: {}", e);
                        Status::internal(format!("Failed to open file: {}", e))
                    }).unwrap()
                    .write_all(&chunk.chunk_data)
                    .await
                    .map_err(|e| {
                        error!("Failed to write chunk: {}", e);
                        Status::internal(format!("Failed to write chunk: {}", e))
                    }).unwrap();

                // Only send progress update if not complete
                if bytes_received < total_size {
                    let _ = tx.send(Ok(UploadProgress {
                        file_info: Some(file_info_clone.clone()),
                        bytes_received,
                        total_size,
                        complete: false,
                    })).await;
                }
            }

            // Send final completion update once
            if bytes_received >= total_size {
                let _ = tx.send(Ok(UploadProgress {
                    file_info: Some(file_info_clone.clone()),
                    bytes_received,
                    total_size,
                    complete: true,
                })).await;
            }
        });
        info!("Uploaded file: {:#?}", file_info.clone());
        Ok(Response::new(Box::pin(output_stream) as Self::UploadFileStream))
    }
}
