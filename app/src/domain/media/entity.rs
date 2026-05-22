use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub id: i32,
    pub filename: String,
    pub original_filename: String,
    pub file_path: String,
    pub mime_type: String,
    pub file_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub uploaded_by: Option<i32>,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Media {
    pub fn is_image(&self) -> bool {
        self.mime_type.starts_with("image/")
    }

    pub fn disk(&self) -> &str {
        "default"
    }

    pub fn original_path(&self) -> PathBuf {
        PathBuf::from("public").join(self.file_path.trim_start_matches('/'))
    }

    pub fn path_hash(&self) -> String {
        let path = self.original_path();
        let path_str = path.to_string_lossy();
        format!("{:x}", md5::compute(path_str.as_bytes()))
    }

    pub fn media_root(&self) -> PathBuf {
        PathBuf::from("public")
            .join("media")
            .join("images")
            .join(self.disk())
            .join(self.path_hash())
    }

    pub fn get_url(&self) -> String {
        format!("/uploads/tus/{}", self.filename)
    }

    pub fn get_media_url(&self, thumb_filename: &str) -> String {
        format!(
            "/media/images/{}/{}/{}",
            self.disk(),
            self.path_hash(),
            thumb_filename
        )
    }

    pub fn get_thumbnail_url(&self, width: Option<i32>, height: Option<i32>, crop: bool) -> String {
        if !self.is_image() {
            return self.get_url();
        }

        let w = width.unwrap_or(300);
        let h = height.unwrap_or(300);

        let thumb_filename = self.generate_thumbnail_filename(w, h, crop);

        self.get_media_url(&thumb_filename)
    }

    fn generate_thumbnail_filename(&self, width: i32, height: i32, crop: bool) -> String {
        let base_name = self
            .filename
            .rsplit_once('.')
            .map(|(name, _)| name)
            .unwrap_or(&self.filename);

        let extension = self
            .filename
            .rsplit_once('.')
            .map(|(_, ext)| ext)
            .unwrap_or("jpg");

        if crop {
            format!("{}-{}x{}-crop.{}", base_name, width, height, extension)
        } else {
            format!("{}-{}x{}.{}", base_name, width, height, extension)
        }
    }

    pub fn get_thumbnail_path(&self, width: i32, height: i32, crop: bool) -> PathBuf {
        let thumb_filename = self.generate_thumbnail_filename(width, height, crop);
        self.media_root().join(thumb_filename)
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "filename": self.filename,
            "original_filename": self.original_filename,
            "file_path": self.file_path,
            "mime_type": self.mime_type,
            "file_size": self.file_size,
            "width": self.width,
            "height": self.height,
            "metadata": self.metadata,
            "title": self.title,
            "alt_text": self.alt_text,
            "caption": self.caption,
            "is_public": self.is_public,
            "is_image": self.is_image(),
            "url": self.get_url(),
            "thumbnail_url": self.get_thumbnail_url(Some(300), Some(300), true),
            "thumbnail_small": self.get_thumbnail_url(Some(150), Some(150), true),
            "thumbnail_medium": self.get_thumbnail_url(Some(300), Some(300), true),
            "thumbnail_large": self.get_thumbnail_url(Some(800), Some(800), false),
            "created_at": self.created_at.to_rfc3339(),
            "updated_at": self.updated_at.to_rfc3339(),
        })
    }
}
