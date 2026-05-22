use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = organizations)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Organization {
    pub id: i32,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub account_id: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = organizations)]
pub struct NewOrganization {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub account_id: Option<i32>,
}

impl NewOrganization {
    pub fn new(name: String) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            name,
            email: None,
            phone: None,
            address: None,
            city: None,
            region: None,
            country: None,
            postal_code: None,
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
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = organizations)]
pub struct OrganizationUpdate {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl OrganizationUpdate {
    pub fn new() -> Self {
        Self {
            name: None,
            email: None,
            phone: None,
            address: None,
            city: None,
            region: None,
            country: None,
            postal_code: None,
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

    pub fn address(mut self, address: String) -> Self {
        self.address = Some(address);
        self
    }

    pub fn city(mut self, city: String) -> Self {
        self.city = Some(city);
        self
    }

    pub fn region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub fn country(mut self, country: String) -> Self {
        self.country = Some(country);
        self
    }

    pub fn postal_code(mut self, postal_code: String) -> Self {
        self.postal_code = Some(postal_code);
        self
    }
}

impl Default for OrganizationUpdate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiResource for Organization {
    const TYPE: &'static str = "organizations";
    type Repository = crate::database::OrganizationRepository;
    type NewModel = NewOrganization;
    type UpdateModel = OrganizationUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "organizations"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "name",
            "email",
            "phone",
            "address",
            "city",
            "region",
            "country",
            "postal_code",
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
            ("email", json!(self.email)),
            ("phone", json!(self.phone)),
            ("address", json!(self.address)),
            ("city", json!(self.city)),
            ("region", json!(self.region)),
            ("country", json!(self.country)),
            ("postal_code", json!(self.postal_code)),
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
        crate::database::OrganizationRepository::new(pool)
    }
}
