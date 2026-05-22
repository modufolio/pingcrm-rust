mod checksum;

pub mod config;
mod filesystem_storage;
mod metadata;
mod server;
mod storage;

pub use checksum::ChecksumInfo;
pub use config::TusConfig;
pub use filesystem_storage::FilesystemStorage;
pub use metadata::UploadMetadata;
pub use server::TusServer;
pub use storage::TusStorage;

pub const TUS_VERSION: &str = "1.0.0";

pub const TUS_EXTENSIONS: &[&str] = &[
    "creation",
    "checksum",
    "concatenation",
    "termination",
    "creation-with-upload",
];

pub const CHECKSUM_ALGORITHMS: &[&str] = &["md5", "sha1", "sha256", "sha512"];
