use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::media;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = media)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MediaModel {
    pub id: i32,
    pub filename: String,
    pub original_filename: String,
    pub file_path: String,
    pub mime_type: String,
    pub file_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub metadata: Option<String>,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub uploaded_by: Option<i32>,
}

impl MediaModel {
    pub fn to_domain(&self) -> crate::domain::Media {
        crate::domain::Media {
            id: self.id,
            filename: self.filename.clone(),
            original_filename: self.original_filename.clone(),
            file_path: self.file_path.clone(),
            mime_type: self.mime_type.clone(),
            file_size: self.file_size,
            width: self.width,
            height: self.height,
            metadata: self
                .metadata
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok()),
            uploaded_by: self.uploaded_by,
            title: self.title.clone(),
            alt_text: self.alt_text.clone(),
            caption: self.caption.clone(),
            is_public: self.is_public,
            created_at: DateTime::<Utc>::from_naive_utc_and_offset(self.created_at, Utc),
            updated_at: DateTime::<Utc>::from_naive_utc_and_offset(self.updated_at, Utc),
        }
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = media)]
pub struct NewMedia {
    pub filename: String,
    pub original_filename: String,
    pub file_path: String,
    pub mime_type: String,
    pub file_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub metadata: Option<String>,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub uploaded_by: Option<i32>,
}

impl NewMedia {
    pub fn new(
        filename: String,
        original_filename: String,
        file_path: String,
        mime_type: String,
        file_size: i64,
    ) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            filename,
            original_filename,
            file_path,
            mime_type,
            file_size,
            width: None,
            height: None,
            metadata: None,
            title: None,
            alt_text: None,
            caption: None,
            is_public: true,
            created_at: now,
            updated_at: now,
            uploaded_by: None,
        }
    }

    pub fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata.to_string());
        self
    }

    pub fn with_uploaded_by(mut self, user_id: i32) -> Self {
        self.uploaded_by = Some(user_id);
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = media)]
pub struct MediaUpdate {
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub is_public: Option<bool>,
    pub metadata: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl MediaUpdate {
    pub fn new() -> Self {
        Self {
            title: None,
            alt_text: None,
            caption: None,
            is_public: None,
            metadata: None,
            updated_at: Utc::now().naive_utc(),
        }
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn alt_text(mut self, alt_text: String) -> Self {
        self.alt_text = Some(alt_text);
        self
    }

    pub fn caption(mut self, caption: String) -> Self {
        self.caption = Some(caption);
        self
    }

    pub fn is_public(mut self, is_public: bool) -> Self {
        self.is_public = Some(is_public);
        self
    }
}

impl Default for MediaUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::database::JsonApiResource for MediaModel {
    const TYPE: &'static str = "media";
    type Repository = crate::database::MediaRepository;
    type NewModel = NewMedia;
    type UpdateModel = MediaUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "media"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "filename",
            "original_filename",
            "file_path",
            "mime_type",
            "file_size",
            "width",
            "height",
            "metadata",
            "title",
            "alt_text",
            "caption",
            "is_public",
            "created_at",
            "updated_at",
            "uploaded_by",
        ]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        vec![
            ("filename", json!(self.filename)),
            ("original_filename", json!(self.original_filename)),
            ("file_path", json!(self.file_path)),
            ("mime_type", json!(self.mime_type)),
            ("file_size", json!(self.file_size)),
            ("width", json!(self.width)),
            ("height", json!(self.height)),
            ("metadata", json!(self.metadata)),
            ("title", json!(self.title)),
            ("alt_text", json!(self.alt_text)),
            ("caption", json!(self.caption)),
            ("is_public", json!(self.is_public)),
            ("uploaded_by", json!(self.uploaded_by)),
            ("created_at", json!(self.created_at.and_utc().to_rfc3339())),
            ("updated_at", json!(self.updated_at.and_utc().to_rfc3339())),
        ]
    }

    fn repository(pool: crate::database::pool::DbPool) -> Self::Repository {
        crate::database::MediaRepository::new(pool)
    }
}
