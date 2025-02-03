fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &[
                format!("{}/../proto/inject.proto", crate_dir),
                format!("{}/../proto/file_info.proto", crate_dir),
            ],
            &[format!("{}/../proto", crate_dir)],
        )?;
    Ok(())
}
