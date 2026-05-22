use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadContainer {
    pub length: Option<u64>,
    pub deferred: bool,
    pub metadata: std::collections::HashMap<String, String>,
    pub is_partial: bool,
    pub partials: Vec<String>,
    pub created_at: String,
    pub expires_at: String,
    pub location: String,
    pub checksum: Option<ChecksumData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecksumData {
    pub algorithm: String,
    pub value: String,
}

#[async_trait]
pub trait TusStorage: Send + Sync {
    fn set_upload_dir(&mut self, upload_dir: PathBuf) -> Result<(), AppError>;

    fn get_upload_dir(&self) -> &PathBuf;

    async fn exists(&self, filename: &str) -> Result<bool, AppError>;

    async fn create(&self, filename: &str) -> Result<(), AppError>;

    async fn append(&self, filename: &str, data: &[u8]) -> Result<(), AppError>;

    async fn get_size(&self, filename: &str) -> Result<u64, AppError>;

    async fn delete(&self, filename: &str) -> Result<(), AppError>;

    async fn container_create(
        &self,
        filename: &str,
        container: &UploadContainer,
    ) -> Result<(), AppError>;

    async fn container_exists(&self, filename: &str) -> Result<bool, AppError>;

    async fn container_fetch(&self, filename: &str) -> Result<Option<UploadContainer>, AppError>;

    async fn container_delete(&self, filename: &str) -> Result<(), AppError>;

    async fn complete(&self, filename: &str) -> Result<bool, AppError>;

    fn supports_cross_check(&self) -> bool;

    fn get_cross_check_algorithms(&self) -> Vec<String>;

    async fn cross_check(
        &self,
        filename: &str,
        algorithm: &str,
        checksum: &str,
    ) -> Result<bool, AppError>;
}
