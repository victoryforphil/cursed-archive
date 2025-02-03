pub mod cursed_archive {
    tonic::include_proto!("cursed_archive");
}

pub use cursed_archive::{FileChunk, UploadProgress}; 