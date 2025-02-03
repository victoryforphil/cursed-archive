use std::path::PathBuf;

use clap::Parser;
use futures::StreamExt;
use log::info;
use pretty_env_logger;

mod proto;
mod services;

use services::FileInjectService;
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct InjectArgs {
    #[clap(short, long = "server", default_value = "127.0.0.1:50051")]
    server: String,
    #[clap(short, long)]
    files: Vec<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    pretty_env_logger::init();

    info!("Starting injest: {}", env!("CARGO_PKG_VERSION"));
    // Parse command line arguments
    let args = InjectArgs::parse();

    // Create service client
    info!("Connecting to server: {}", args.server);
    let mut service = FileInjectService::new(args.server).await?;

    // Upload each file
    for file_path in args.files {
        log::info!("Uploading file: {:#?}", file_path);
        
        let mut progress_stream = service.upload_file(file_path).await?;
        
        while let Some(progress) = progress_stream.next().await {
            match progress {
                Ok(p) => {
                    log::info!(
                        "Progress for {}: {}/{} bytes ({}%)", 
                        p.file_info.unwrap().name,
                        p.bytes_received,
                        p.total_size,
                        (p.bytes_received as f64 / p.total_size as f64 * 100.0) as u32
                    );
                    if p.complete {
                        log::info!("Upload complete!");
                    }
                }
                Err(e) => {
                    log::error!("Error during upload: {}", e);
                    break;
                }
            }
        }
    }

    Ok(())
}
