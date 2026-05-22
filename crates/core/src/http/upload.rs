use crate::error::{AppError, AppResult};
use axum::body::Bytes;
use axum_extra::extract::Multipart;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

pub const DEFAULT_MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone)]
pub enum AllowedMimeType {
    Image,

    Video,

    Audio,

    Pdf,

    Text,

    Json,

    Xml,

    Specific(String),
}

impl AllowedMimeType {
    pub fn matches(&self, mime_type: &str) -> bool {
        match self {
            AllowedMimeType::Image => mime_type.starts_with("image/"),
            AllowedMimeType::Video => mime_type.starts_with("video/"),
            AllowedMimeType::Audio => mime_type.starts_with("audio/"),
            AllowedMimeType::Pdf => mime_type == "application/pdf",
            AllowedMimeType::Text => mime_type.starts_with("text/"),
            AllowedMimeType::Json => mime_type == "application/json" || mime_type == "text/json",
            AllowedMimeType::Xml => mime_type == "application/xml" || mime_type == "text/xml",
            AllowedMimeType::Specific(allowed) => mime_type == allowed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UploadConfig {
    pub max_file_size: usize,

    pub allowed_mime_types: Vec<AllowedMimeType>,

    pub allowed_extensions: Vec<String>,

    pub upload_dir: PathBuf,

    pub generate_unique_names: bool,

    pub preserve_original_names: bool,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            allowed_mime_types: Vec::new(),
            allowed_extensions: Vec::new(),
            upload_dir: PathBuf::from("uploads"),
            generate_unique_names: true,
            preserve_original_names: false,
        }
    }
}

impl UploadConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    pub fn allow_mime_type(mut self, mime_type: AllowedMimeType) -> Self {
        self.allowed_mime_types.push(mime_type);
        self
    }

    pub fn allow_extension(mut self, ext: impl Into<String>) -> Self {
        self.allowed_extensions.push(ext.into());
        self
    }

    pub fn upload_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.upload_dir = dir.into();
        self
    }

    pub fn generate_unique_names(mut self, generate: bool) -> Self {
        self.generate_unique_names = generate;
        self
    }

    pub fn preserve_original_names(mut self, preserve: bool) -> Self {
        self.preserve_original_names = preserve;
        self
    }

    pub fn validate(&self, file: &UploadedFile) -> AppResult<()> {
        if file.size > self.max_file_size {
            return Err(AppError::PayloadTooLarge(format!(
                "File size {} exceeds maximum allowed size of {}",
                file.size, self.max_file_size
            )));
        }

        if !self.allowed_mime_types.is_empty() {
            if let Some(ref mime_type) = file.content_type {
                let allowed = self
                    .allowed_mime_types
                    .iter()
                    .any(|allowed| allowed.matches(mime_type));

                if !allowed {
                    return Err(AppError::UnsupportedMediaType(format!(
                        "MIME type '{}' is not allowed",
                        mime_type
                    )));
                }
            }
        }

        if !self.allowed_extensions.is_empty() {
            if let Some(ref filename) = file.filename {
                let ext = Path::new(filename)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");

                let allowed = self.allowed_extensions.iter().any(|e| e == ext);

                if !allowed {
                    return Err(AppError::BadRequest(format!(
                        "File extension '{}' is not allowed",
                        ext
                    )));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UploadedFile {
    pub filename: Option<String>,

    pub content_type: Option<String>,

    pub size: usize,

    pub data: Bytes,
}

impl UploadedFile {
    pub fn new(data: Bytes) -> Self {
        let size = data.len();
        Self {
            filename: None,
            content_type: None,
            size,
            data,
        }
    }

    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub fn extension(&self) -> Option<&str> {
        self.filename
            .as_ref()
            .and_then(|name| Path::new(name).extension().and_then(|ext| ext.to_str()))
    }

    pub async fn save(&self, path: impl AsRef<Path>) -> AppResult<PathBuf> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AppError::InternalError(format!("Failed to create upload directory: {}", e))
            })?;
        }

        let mut file = File::create(path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create file: {}", e)))?;

        file.write_all(&self.data)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to write file: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to sync file: {}", e)))?;

        Ok(path.to_path_buf())
    }

    pub async fn save_with_config(&self, config: &UploadConfig) -> AppResult<PathBuf> {
        config.validate(self)?;

        let filename = if config.generate_unique_names {
            self.generate_unique_filename()
        } else if config.preserve_original_names {
            self.filename
                .clone()
                .unwrap_or_else(|| self.generate_unique_filename())
        } else {
            self.generate_unique_filename()
        };

        let path = config.upload_dir.join(&filename);

        self.save(&path).await
    }

    fn generate_unique_filename(&self) -> String {
        let uuid = uuid::Uuid::new_v4();
        let ext = self.extension().unwrap_or("bin");
        format!("{}.{}", uuid, ext)
    }
}

pub struct FileUploadHandler {
    config: UploadConfig,
}

impl FileUploadHandler {
    pub fn new(config: UploadConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self {
            config: UploadConfig::default(),
        }
    }

    pub async fn process_multipart(
        &self,
        mut multipart: Multipart,
    ) -> AppResult<Vec<UploadedFile>> {
        let mut files = Vec::new();

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {}", e)))?
        {
            let filename = field.file_name().map(|s| s.to_string());
            let content_type = field.content_type().map(|s| s.to_string());

            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read field data: {}", e)))?;

            let mut file = UploadedFile::new(data);
            if let Some(name) = filename {
                file = file.with_filename(name);
            }
            if let Some(ct) = content_type {
                file = file.with_content_type(ct);
            }

            self.config.validate(&file)?;

            files.push(file);
        }

        Ok(files)
    }

    pub async fn save_multipart(&self, multipart: Multipart) -> AppResult<Vec<PathBuf>> {
        let files = self.process_multipart(multipart).await?;
        let mut paths = Vec::new();

        for file in files {
            let path = file.save_with_config(&self.config).await?;
            paths.push(path);
        }

        Ok(paths)
    }
}

pub mod presets {
    use super::*;

    pub fn images() -> UploadConfig {
        UploadConfig::new()
            .max_size(5 * 1024 * 1024)
            .allow_mime_type(AllowedMimeType::Image)
            .allow_extension("jpg")
            .allow_extension("jpeg")
            .allow_extension("png")
            .allow_extension("gif")
            .allow_extension("webp")
    }

    pub fn documents() -> UploadConfig {
        UploadConfig::new()
            .max_size(10 * 1024 * 1024)
            .allow_mime_type(AllowedMimeType::Pdf)
            .allow_mime_type(AllowedMimeType::Specific("application/msword".to_string()))
            .allow_mime_type(AllowedMimeType::Specific(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .to_string(),
            ))
            .allow_extension("pdf")
            .allow_extension("doc")
            .allow_extension("docx")
    }

    pub fn videos() -> UploadConfig {
        UploadConfig::new()
            .max_size(100 * 1024 * 1024)
            .allow_mime_type(AllowedMimeType::Video)
            .allow_extension("mp4")
            .allow_extension("mov")
            .allow_extension("avi")
            .allow_extension("webm")
    }

    pub fn avatars() -> UploadConfig {
        UploadConfig::new()
            .max_size(2 * 1024 * 1024)
            .allow_mime_type(AllowedMimeType::Image)
            .allow_extension("jpg")
            .allow_extension("jpeg")
            .allow_extension("png")
            .allow_extension("webp")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_type_matching() {
        assert!(AllowedMimeType::Image.matches("image/jpeg"));
        assert!(AllowedMimeType::Image.matches("image/png"));
        assert!(!AllowedMimeType::Image.matches("video/mp4"));

        assert!(AllowedMimeType::Pdf.matches("application/pdf"));
        assert!(!AllowedMimeType::Pdf.matches("application/json"));

        let specific = AllowedMimeType::Specific("application/json".to_string());
        assert!(specific.matches("application/json"));
        assert!(!specific.matches("text/json"));
    }

    #[test]
    fn test_upload_config() {
        let config = UploadConfig::new()
            .max_size(1024)
            .allow_extension("jpg")
            .allow_mime_type(AllowedMimeType::Image);

        assert_eq!(config.max_file_size, 1024);
        assert_eq!(config.allowed_extensions.len(), 1);
        assert_eq!(config.allowed_mime_types.len(), 1);
    }

    #[test]
    fn test_uploaded_file() {
        let data = Bytes::from("test data");
        let file = UploadedFile::new(data.clone())
            .with_filename("test.jpg")
            .with_content_type("image/jpeg");

        assert_eq!(file.filename, Some("test.jpg".to_string()));
        assert_eq!(file.content_type, Some("image/jpeg".to_string()));
        assert_eq!(file.size, data.len());
        assert_eq!(file.extension(), Some("jpg"));
    }

    #[test]
    fn test_validate_file_size() {
        let config = UploadConfig::new().max_size(100);

        let small_file = UploadedFile::new(Bytes::from("small"));
        assert!(config.validate(&small_file).is_ok());

        let large_file = UploadedFile::new(Bytes::from(vec![0u8; 1000]));
        assert!(config.validate(&large_file).is_err());
    }

    #[test]
    fn test_validate_mime_type() {
        let config = UploadConfig::new().allow_mime_type(AllowedMimeType::Image);

        let image = UploadedFile::new(Bytes::from("data")).with_content_type("image/jpeg");
        assert!(config.validate(&image).is_ok());

        let pdf = UploadedFile::new(Bytes::from("data")).with_content_type("application/pdf");
        assert!(config.validate(&pdf).is_err());
    }

    #[test]
    fn test_validate_extension() {
        let config = UploadConfig::new()
            .allow_extension("jpg")
            .allow_extension("png");

        let jpg = UploadedFile::new(Bytes::from("data")).with_filename("photo.jpg");
        assert!(config.validate(&jpg).is_ok());

        let pdf = UploadedFile::new(Bytes::from("data")).with_filename("doc.pdf");
        assert!(config.validate(&pdf).is_err());
    }

    #[test]
    fn test_presets() {
        let images = presets::images();
        assert_eq!(images.max_file_size, 5 * 1024 * 1024);

        let docs = presets::documents();
        assert_eq!(docs.max_file_size, 10 * 1024 * 1024);

        let videos = presets::videos();
        assert_eq!(videos.max_file_size, 100 * 1024 * 1024);

        let avatars = presets::avatars();
        assert_eq!(avatars.max_file_size, 2 * 1024 * 1024);
    }
}
