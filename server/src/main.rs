use clap::{Arg, Command};
use pretty_env_logger;
use tonic::transport::Server;

pub mod proto;
use proto::{FileInjectServiceImpl, cursed_archive::file_inject_service_server::FileInjectServiceServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    pretty_env_logger::init();

    // Parse command line arguments
    let matches = Command::new("cursed-archive-server")
        .version("0.1.0")
        .author("Your Name")
        .about("gRPC server for cursed archive")
        .arg(
            Arg::new("address")
                .short('a')
                .long("address")
                .value_name("ADDRESS")
                .help("The address to bind to")
                .default_value("127.0.0.1:50051"),
        )
        .get_matches();

    let addr = matches
        .get_one::<String>("address")
        .unwrap()
        .parse()
        .expect("Failed to parse address");

    let file_inject_service = FileInjectServiceImpl::default();
    let svc = FileInjectServiceServer::new(file_inject_service);

    log::info!("Starting server on {}", addr);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
