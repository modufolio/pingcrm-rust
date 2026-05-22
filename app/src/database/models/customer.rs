use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = customers)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Customer {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub account_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = customers)]
pub struct NewCustomer {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub account_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl NewCustomer {
    pub fn new(name: String) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            name,
            email: None,
            phone: None,
            account_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn with_phone(mut self, phone: String) -> Self {
        self.phone = Some(phone);
        self
    }

    pub fn with_account(mut self, account_id: i32) -> Self {
        self.account_id = Some(account_id);
        self
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = customers)]
pub struct CustomerUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl CustomerUpdate {
    pub fn new() -> Self {
        Self {
            name: None,
            email: None,
            phone: None,
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn phone(mut self, phone: String) -> Self {
        self.phone = Some(phone);
        self
    }
}

impl Default for CustomerUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::database::JsonApiResource for Customer {
    const TYPE: &'static str = "customers";
    type Repository = crate::database::CustomerRepository;
    type NewModel = NewCustomer;
    type UpdateModel = CustomerUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "customers"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "name",
            "email",
            "phone",
            "account_id",
            "created_at",
            "updated_at",
            "deleted_at",
        ]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        vec![
            ("name", json!(self.name)),
            ("email", json!(self.email)),
            ("phone", json!(self.phone)),
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
        crate::database::CustomerRepository::new(pool)
    }
}
