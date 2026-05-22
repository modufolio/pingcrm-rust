use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount {
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl NewAccount {
    pub fn new(name: String) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            name,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = accounts)]
pub struct AccountUpdate {
    pub name: Option<String>,
    pub updated_at: NaiveDateTime,
}

impl AccountUpdate {
    pub fn new() -> Self {
        Self {
            name: None,
            updated_at: Utc::now().naive_utc(),
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

impl Default for AccountUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::database::JsonApiResource for Account {
    const TYPE: &'static str = "accounts";
    type Repository = crate::database::AccountRepository;
    type NewModel = NewAccount;
    type UpdateModel = AccountUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "accounts"
    }

    fn field_names() -> &'static [&'static str] {
        &["id", "name", "created_at", "updated_at"]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        vec![
            ("name", json!(self.name)),
            ("created_at", json!(self.created_at.and_utc().to_rfc3339())),
            ("updated_at", json!(self.updated_at.and_utc().to_rfc3339())),
        ]
    }

    fn repository(pool: crate::database::pool::DbPool) -> Self::Repository {
        crate::database::AccountRepository::new(pool)
    }
}
