use futures::Stream;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_stream::StreamExt;
use tonic::Request;
use crate::proto::cursed_archive::file_inject_service_client::FileInjectServiceClient;
use crate::proto::UploadProgress;
use crate::proto::FileChunk;
const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks

pub struct FileInjectService {
    client: FileInjectServiceClient<tonic::transport::Channel>,
}

impl FileInjectService {
    pub async fn new(addr: String) -> Result<Self, anyhow::Error> {
        let client = FileInjectServiceClient::connect(format!("http://{}", addr)).await?;
        Ok(Self { client })
    }

    pub async fn upload_file(
        &mut self,
        file_path: PathBuf,
    ) -> Result<impl Stream<Item = Result<UploadProgress, tonic::Status>>, anyhow::Error> {
        let path = file_path.clone();
        let mut file = File::open(path.clone()).await?;
        let total_size = file.metadata().await?.len();
        
        // Create a stream of chunks
        let stream = async_stream::stream! {
            let mut buffer = vec![0; CHUNK_SIZE];
            let mut bytes_sent = 0u64;
            let mut offset = 0u64;

            // First chunk
            match file.read(&mut buffer).await {
                Ok(n) if n > 0 => {
                    bytes_sent += n as u64;
                    yield FileChunk {
                        file_path: path.to_string_lossy().to_string(),
                        total_size,
                        chunk_data: buffer[..n].to_vec(),
                        offset,
                        bytes_sent,
                    };
                    offset += n as u64;
                }
                _ => return,
            }

            // Remaining chunks
            while let Ok(n) = file.read(&mut buffer).await {
                if n == 0 {
                    break;
                }
                bytes_sent += n as u64;
                yield FileChunk {
                    file_path: path.to_string_lossy().to_string(),
                    total_size,
                    chunk_data: buffer[..n].to_vec(),
                    offset,
                    bytes_sent,
                };
                offset += n as u64;
            }
        };

        let response = self.client.upload_file(Request::new(stream)).await?;
        Ok(response.into_inner())
    }
} 