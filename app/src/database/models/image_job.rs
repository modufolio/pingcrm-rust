use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::database::schema::image_jobs;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = image_jobs)]
pub struct ImageJob {
    pub id: i32,
    pub disk: String,
    pub filename: String,
    pub original_filename: String,
    pub options: String,
    pub status: String,
    pub processed_at: Option<NaiveDateTime>,
    pub accessed_at: Option<NaiveDateTime>,
    pub access_count: i32,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = image_jobs)]
pub struct NewImageJob {
    pub disk: String,
    pub filename: String,
    pub original_filename: String,
    pub options: String,
    pub status: String,
}

impl NewImageJob {
    pub fn new(
        disk: String,
        filename: String,
        original_filename: String,
        options: serde_json::Value,
    ) -> Self {
        Self {
            disk,
            filename,
            original_filename,
            options: options.to_string(),
            status: "pending".to_string(),
        }
    }
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = image_jobs)]
pub struct UpdateImageJob {
    pub status: Option<String>,
    pub processed_at: Option<NaiveDateTime>,
    pub accessed_at: Option<NaiveDateTime>,
    pub access_count: Option<i32>,
    pub error_message: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl ImageJob {
    pub fn get_options(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.options)
    }

    pub fn is_processed(&self) -> bool {
        self.status == "processed"
    }

    pub fn is_failed(&self) -> bool {
        self.status == "failed"
    }
}
