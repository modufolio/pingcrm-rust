use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::path::PathBuf;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

use super::storage::{TusStorage, UploadContainer};
use crate::error::AppError;

const CONTAINER_SUFFIX: &str = ".cachecontainer";

pub struct FilesystemStorage {
    upload_dir: PathBuf,
}

impl FilesystemStorage {
    pub fn new(upload_dir: PathBuf) -> Result<Self, AppError> {
        let mut storage = Self {
            upload_dir: PathBuf::new(),
        };
        storage.set_upload_dir(upload_dir)?;
        Ok(storage)
    }

    fn validate_filename(&self, filename: &str) -> Result<(), AppError> {
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(AppError::BadRequest("Invalid filename".to_string()));
        }

        if filename.trim().is_empty() || filename.len() > 255 {
            return Err(AppError::BadRequest("Invalid filename length".to_string()));
        }

        Ok(())
    }

    fn get_file_path(&self, filename: &str) -> PathBuf {
        self.upload_dir.join(filename)
    }

    fn get_container_path(&self, filename: &str) -> PathBuf {
        self.upload_dir
            .join(format!("{}{}", filename, CONTAINER_SUFFIX))
    }

    async fn compute_checksum(&self, filename: &str, algorithm: &str) -> Result<String, AppError> {
        let path = self.get_file_path(filename);
        let data = fs::read(&path).await.map_err(|e| {
            AppError::InternalError(format!("Failed to read file for checksum: {}", e))
        })?;

        let hash = match algorithm {
            "md5" => {
                let digest = md5::compute(&data);
                STANDARD.encode(digest.as_ref())
            }
            "sha1" => {
                use sha1::{Digest, Sha1};
                let mut hasher = Sha1::new();
                hasher.update(&data);
                let result = hasher.finalize();
                STANDARD.encode(result.as_slice())
            }
            "sha256" => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&data);
                let result = hasher.finalize();
                STANDARD.encode(result.as_slice())
            }
            "sha512" => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(&data);
                let result = hasher.finalize();
                STANDARD.encode(result.as_slice())
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Unsupported algorithm: {}",
                    algorithm
                )))
            }
        };

        Ok(hash)
    }
}

#[async_trait]
impl TusStorage for FilesystemStorage {
    fn set_upload_dir(&mut self, upload_dir: PathBuf) -> Result<(), AppError> {
        if !upload_dir.exists() {
            std::fs::create_dir_all(&upload_dir).map_err(|e| {
                AppError::InternalError(format!("Failed to create upload directory: {}", e))
            })?;
        }

        if !upload_dir.is_dir() {
            return Err(AppError::InternalError(
                "Upload path is not a directory".to_string(),
            ));
        }

        self.upload_dir = upload_dir;
        Ok(())
    }

    fn get_upload_dir(&self) -> &PathBuf {
        &self.upload_dir
    }

    async fn exists(&self, filename: &str) -> Result<bool, AppError> {
        self.validate_filename(filename)?;
        let path = self.get_file_path(filename);
        Ok(path.exists())
    }

    async fn create(&self, filename: &str) -> Result<(), AppError> {
        self.validate_filename(filename)?;
        let path = self.get_file_path(filename);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AppError::InternalError(format!("Failed to create directory: {}", e))
            })?;
        }

        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to create file: {}", e)))?;

        Ok(())
    }

    async fn append(&self, filename: &str, data: &[u8]) -> Result<(), AppError> {
        self.validate_filename(filename)?;
        let path = self.get_file_path(filename);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to open file: {}", e)))?;

        file.write_all(data)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to write data: {}", e)))?;

        file.flush()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to flush data: {}", e)))?;

        Ok(())
    }

    async fn get_size(&self, filename: &str) -> Result<u64, AppError> {
        self.validate_filename(filename)?;
        let path = self.get_file_path(filename);

        if !path.exists() {
            return Ok(0);
        }

        let metadata = fs::metadata(&path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to get file metadata: {}", e)))?;

        Ok(metadata.len())
    }

    async fn delete(&self, filename: &str) -> Result<(), AppError> {
        self.validate_filename(filename)?;
        let path = self.get_file_path(filename);

        if path.exists() {
            fs::remove_file(&path)
                .await
                .map_err(|e| AppError::InternalError(format!("Failed to delete file: {}", e)))?;
        }

        Ok(())
    }

    async fn container_create(
        &self,
        filename: &str,
        container: &UploadContainer,
    ) -> Result<(), AppError> {
        self.validate_filename(filename)?;
        let path = self.get_container_path(filename);

        let json = serde_json::to_string_pretty(container).map_err(|e| {
            AppError::InternalError(format!("Failed to serialize container: {}", e))
        })?;

        fs::write(&path, json)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to write container: {}", e)))?;

        Ok(())
    }

    async fn container_exists(&self, filename: &str) -> Result<bool, AppError> {
        self.validate_filename(filename)?;
        let path = self.get_container_path(filename);
        Ok(path.exists())
    }

    async fn container_fetch(&self, filename: &str) -> Result<Option<UploadContainer>, AppError> {
        self.validate_filename(filename)?;
        let path = self.get_container_path(filename);

        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to read container: {}", e)))?;

        let container: UploadContainer = serde_json::from_str(&contents)
            .map_err(|e| AppError::InternalError(format!("Failed to parse container: {}", e)))?;

        if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(&container.expires_at) {
            if expires_at < chrono::Utc::now() {
                let _ = self.container_delete(filename).await;
                let _ = self.delete(filename).await;
                return Ok(None);
            }
        }

        Ok(Some(container))
    }

    async fn container_delete(&self, filename: &str) -> Result<(), AppError> {
        self.validate_filename(filename)?;
        let path = self.get_container_path(filename);

        if path.exists() {
            fs::remove_file(&path).await.map_err(|e| {
                AppError::InternalError(format!("Failed to delete container: {}", e))
            })?;
        }

        Ok(())
    }

    async fn complete(&self, filename: &str) -> Result<bool, AppError> {
        self.validate_filename(filename)?;

        if !self.exists(filename).await? {
            return Err(AppError::NotFound("File not found".to_string()));
        }

        self.container_delete(filename).await?;
        Ok(true)
    }

    fn supports_cross_check(&self) -> bool {
        true
    }

    fn get_cross_check_algorithms(&self) -> Vec<String> {
        vec![
            "md5".to_string(),
            "sha1".to_string(),
            "sha256".to_string(),
            "sha512".to_string(),
        ]
    }

    async fn cross_check(
        &self,
        filename: &str,
        algorithm: &str,
        checksum: &str,
    ) -> Result<bool, AppError> {
        self.validate_filename(filename)?;

        if !self.exists(filename).await? {
            return Ok(false);
        }

        let calculated = self.compute_checksum(filename, algorithm).await?;
        Ok(calculated == checksum)
    }
}
