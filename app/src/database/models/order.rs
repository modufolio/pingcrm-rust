use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::orders;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = orders)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Order {
    pub id: i32,
    pub number: String,
    pub status: String,
    pub currency: String,
    pub subtotal: i32,
    pub tax: i32,
    pub shipping_cost: i32,
    pub total: i32,
    pub notes: Option<String>,
    pub customer_id: Option<i32>,
    pub shipping_address_id: Option<i32>,
    pub billing_address_id: Option<i32>,
    pub account_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub number: String,
    pub status: String,
    pub currency: String,
    pub subtotal: i32,
    pub tax: i32,
    pub shipping_cost: i32,
    pub total: i32,
    pub notes: Option<String>,
    pub customer_id: Option<i32>,
    pub shipping_address_id: Option<i32>,
    pub billing_address_id: Option<i32>,
    pub account_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl NewOrder {
    pub fn new(number: String, currency: String) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            number,
            status: "pending".to_string(),
            currency,
            subtotal: 0,
            tax: 0,
            shipping_cost: 0,
            total: 0,
            notes: None,
            customer_id: None,
            shipping_address_id: None,
            billing_address_id: None,
            account_id: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn with_account(mut self, account_id: i32) -> Self {
        self.account_id = Some(account_id);
        self
    }

    pub fn with_customer(mut self, customer_id: i32) -> Self {
        self.customer_id = Some(customer_id);
        self
    }

    pub fn with_status(mut self, status: String) -> Self {
        self.status = status;
        self
    }

    pub fn with_amounts(mut self, subtotal: i32, tax: i32, shipping_cost: i32) -> Self {
        self.subtotal = subtotal;
        self.tax = tax;
        self.shipping_cost = shipping_cost;
        self.total = subtotal + tax + shipping_cost;
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    pub fn with_addresses(
        mut self,
        shipping_address_id: Option<i32>,
        billing_address_id: Option<i32>,
    ) -> Self {
        self.shipping_address_id = shipping_address_id;
        self.billing_address_id = billing_address_id;
        self
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = orders)]
pub struct OrderUpdate {
    pub customer_id: Option<i32>,
    pub number: Option<String>,
    pub status: Option<String>,
    pub currency: Option<String>,
    pub subtotal: Option<i32>,
    pub tax: Option<i32>,
    pub shipping_cost: Option<i32>,
    pub total: Option<i32>,
    pub notes: Option<String>,
    pub shipping_address_id: Option<i32>,
    pub billing_address_id: Option<i32>,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl OrderUpdate {
    pub fn new() -> Self {
        Self {
            customer_id: None,
            number: None,
            status: None,
            currency: None,
            subtotal: None,
            tax: None,
            shipping_cost: None,
            total: None,
            notes: None,
            shipping_address_id: None,
            billing_address_id: None,
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
        }
    }

    pub fn status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    pub fn notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }

    pub fn amounts(mut self, subtotal: i32, tax: i32, shipping_cost: i32) -> Self {
        self.subtotal = Some(subtotal);
        self.tax = Some(tax);
        self.shipping_cost = Some(shipping_cost);
        self.total = Some(subtotal + tax + shipping_cost);
        self
    }
}

impl Default for OrderUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::database::JsonApiResource for Order {
    const TYPE: &'static str = "orders";
    type Repository = crate::database::OrderRepository;
    type NewModel = NewOrder;
    type UpdateModel = OrderUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "orders"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "number",
            "status",
            "currency",
            "subtotal",
            "tax",
            "shipping_cost",
            "total",
            "notes",
            "customer_id",
            "shipping_address_id",
            "billing_address_id",
            "account_id",
            "created_at",
            "updated_at",
            "deleted_at",
        ]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        vec![
            ("number", json!(self.number)),
            ("status", json!(self.status)),
            ("currency", json!(self.currency)),
            ("subtotal", json!(self.subtotal)),
            ("tax", json!(self.tax)),
            ("shipping_cost", json!(self.shipping_cost)),
            ("total", json!(self.total)),
            ("notes", json!(self.notes)),
            ("customer_id", json!(self.customer_id)),
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
        crate::database::OrderRepository::new(pool)
    }
}
