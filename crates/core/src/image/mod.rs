pub mod options;
pub mod processor;
pub mod thumbnail;

pub use options::{CropPosition, ThumbOptions, ThumbOptionsBuilder};
pub use processor::ImageProcessor;
pub use thumbnail::Thumbnail;

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("Failed to open image: {0}")]
    OpenError(#[from] image::ImageError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid dimensions: {0}")]
    InvalidDimensions(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

pub type Result<T> = std::result::Result<T, ImageError>;
