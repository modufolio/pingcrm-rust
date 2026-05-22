use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = addresses)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Address {
    pub id: i32,
    pub label: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub country: String,
    pub postal_code: String,
    pub phone: Option<String>,
    pub is_default: bool,
    pub address_type: String,
    pub customer_id: Option<i32>,
    pub brand_id: Option<i32>,
    pub order_id: Option<i32>,
    pub account_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = addresses)]
pub struct NewAddress {
    pub label: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub city: String,
    pub region: Option<String>,
    pub country: String,
    pub postal_code: String,
    pub phone: Option<String>,
    pub is_default: bool,
    pub address_type: String,
    pub customer_id: Option<i32>,
    pub brand_id: Option<i32>,
    pub order_id: Option<i32>,
    pub account_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl NewAddress {
    pub fn new(
        address_type: String,
        address_line1: String,
        city: String,
        country: String,
        postal_code: String,
    ) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            label: None,
            address_line1,
            address_line2: None,
            city,
            region: None,
            country,
            postal_code,
            phone: None,
            is_default: false,
            address_type,
            customer_id: None,
            brand_id: None,
            order_id: None,
            account_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub fn with_customer(mut self, customer_id: i32) -> Self {
        self.customer_id = Some(customer_id);
        self
    }

    pub fn with_account(mut self, account_id: i32) -> Self {
        self.account_id = Some(account_id);
        self
    }

    pub fn with_brand(mut self, brand_id: i32) -> Self {
        self.brand_id = Some(brand_id);
        self
    }

    pub fn with_order(mut self, order_id: i32) -> Self {
        self.order_id = Some(order_id);
        self
    }

    pub fn set_default(mut self, is_default: bool) -> Self {
        self.is_default = is_default;
        self
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = addresses)]
pub struct AddressUpdate {
    pub label: Option<String>,
    pub address_type: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
    pub is_default: Option<bool>,
    pub updated_at: NaiveDateTime,
}

impl AddressUpdate {
    pub fn new() -> Self {
        Self {
            label: None,
            address_type: None,
            address_line1: None,
            address_line2: None,
            city: None,
            region: None,
            country: None,
            postal_code: None,
            phone: None,
            is_default: None,
            updated_at: Utc::now().naive_utc(),
        }
    }

    pub fn address_type(mut self, address_type: String) -> Self {
        self.address_type = Some(address_type);
        self
    }

    pub fn address_line1(mut self, address: String) -> Self {
        self.address_line1 = Some(address);
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

impl Default for AddressUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::database::JsonApiResource for Address {
    const TYPE: &'static str = "addresses";
    type Repository = crate::database::AddressRepository;
    type NewModel = NewAddress;
    type UpdateModel = AddressUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "addresses"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "label",
            "address_line1",
            "address_line2",
            "city",
            "region",
            "country",
            "postal_code",
            "phone",
            "is_default",
            "address_type",
            "customer_id",
            "brand_id",
            "order_id",
            "account_id",
            "created_at",
            "updated_at",
        ]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        vec![
            ("label", json!(self.label)),
            ("address_line1", json!(self.address_line1)),
            ("address_line2", json!(self.address_line2)),
            ("city", json!(self.city)),
            ("region", json!(self.region)),
            ("country", json!(self.country)),
            ("postal_code", json!(self.postal_code)),
            ("phone", json!(self.phone)),
            ("is_default", json!(self.is_default)),
            ("address_type", json!(self.address_type)),
            ("customer_id", json!(self.customer_id)),
            ("brand_id", json!(self.brand_id)),
            ("order_id", json!(self.order_id)),
            ("account_id", json!(self.account_id)),
            ("created_at", json!(self.created_at.and_utc().to_rfc3339())),
            ("updated_at", json!(self.updated_at.and_utc().to_rfc3339())),
        ]
    }

    fn repository(pool: crate::database::pool::DbPool) -> Self::Repository {
        crate::database::AddressRepository::new(pool)
    }
}
