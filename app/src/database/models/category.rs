use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = categories)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_visible: bool,
    pub parent_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub account_id: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = categories)]
pub struct NewCategory {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_visible: bool,
    pub parent_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub account_id: Option<i32>,
}

impl NewCategory {
    pub fn new(name: String) -> Self {
        let now = Utc::now().naive_utc();
        let slug = name.to_lowercase().replace(' ', "-");
        Self {
            name,
            slug,
            description: None,
            is_visible: true,
            parent_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            account_id: None,
        }
    }

    pub fn with_account(mut self, account_id: i32) -> Self {
        self.account_id = Some(account_id);
        self
    }

    pub fn with_parent(mut self, parent_id: i32) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = categories)]
pub struct CategoryUpdate {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub is_visible: Option<bool>,
    pub parent_id: Option<i32>,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl CategoryUpdate {
    pub fn new() -> Self {
        Self {
            name: None,
            slug: None,
            description: None,
            is_visible: None,
            parent_id: None,
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn is_visible(mut self, is_visible: bool) -> Self {
        self.is_visible = Some(is_visible);
        self
    }
}

impl Default for CategoryUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::database::JsonApiResource for Category {
    const TYPE: &'static str = "categories";
    type Repository = crate::database::CategoryRepository;
    type NewModel = NewCategory;
    type UpdateModel = CategoryUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "categories"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "name",
            "slug",
            "description",
            "is_visible",
            "parent_id",
            "created_at",
            "updated_at",
            "deleted_at",
            "account_id",
        ]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        vec![
            ("name", json!(self.name)),
            ("slug", json!(self.slug)),
            ("description", json!(self.description)),
            ("is_visible", json!(self.is_visible)),
            ("parent_id", json!(self.parent_id)),
            ("account_id", json!(self.account_id)),
            ("created_at", json!(self.created_at.and_utc().to_rfc3339())),
            ("updated_at", json!(self.updated_at.and_utc().to_rfc3339())),
            (
                "deleted_at",
                json!(self.deleted_at.map(|dt| dt.and_utc().to_rfc3339())),
            ),
        ]
    }

    fn repository(pool: crate::database::pool::DbPool) -> Self::Repository {
        crate::database::CategoryRepository::new(pool)
    }
}
