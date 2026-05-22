use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Thumbnail {
    pub path: PathBuf,
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Thumbnail {
    pub fn new(path: impl Into<PathBuf>, url: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            url: url.into(),
            width: None,
            height: None,
        }
    }

    pub fn with_dimensions(
        path: impl Into<PathBuf>,
        url: impl Into<String>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            path: path.into(),
            url: url.into(),
            width: Some(width),
            height: Some(height),
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn width(&self) -> Option<u32> {
        self.width
    }

    pub fn height(&self) -> Option<u32> {
        self.height
    }
}
