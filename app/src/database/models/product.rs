use chrono::{NaiveDate, NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = products)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub description: Option<String>,
    pub price: i32,
    pub old_price: Option<i32>,
    pub cost: Option<i32>,
    pub quantity: i32,
    pub security_stock: i32,
    pub stock_status: String,
    pub backorder: bool,
    pub requires_shipping: bool,
    pub published_at: Option<NaiveDate>,
    pub is_visible: bool,
    pub is_featured: bool,
    pub image: Option<String>,
    pub brand_id: Option<i32>,
    pub account_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub name: String,
    pub slug: String,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub description: Option<String>,
    pub price: i32,
    pub old_price: Option<i32>,
    pub cost: Option<i32>,
    pub quantity: i32,
    pub security_stock: i32,
    pub stock_status: String,
    pub backorder: bool,
    pub requires_shipping: bool,
    pub published_at: Option<NaiveDate>,
    pub is_visible: bool,
    pub is_featured: bool,
    pub image: Option<String>,
    pub brand_id: Option<i32>,
    pub account_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl NewProduct {
    pub fn new(name: String, price: i32) -> Self {
        let now = Utc::now().naive_utc();
        let slug = name.to_lowercase().replace(' ', "-");
        Self {
            name,
            slug,
            sku: None,
            barcode: None,
            description: None,
            price,
            old_price: None,
            cost: None,
            quantity: 0,
            security_stock: 0,
            stock_status: "in_stock".to_string(),
            backorder: false,
            requires_shipping: true,
            published_at: None,
            is_visible: true,
            is_featured: false,
            image: None,
            brand_id: None,
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

    pub fn with_brand(mut self, brand_id: i32) -> Self {
        self.brand_id = Some(brand_id);
        self
    }

    pub fn with_sku(mut self, sku: String) -> Self {
        self.sku = Some(sku);
        self
    }

    pub fn with_quantity(mut self, quantity: i32) -> Self {
        self.quantity = quantity;
        self
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = products)]
pub struct ProductUpdate {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub description: Option<String>,
    pub price: Option<i32>,
    pub old_price: Option<i32>,
    pub cost: Option<i32>,
    pub quantity: Option<i32>,
    pub security_stock: Option<i32>,
    pub stock_status: Option<String>,
    pub backorder: Option<bool>,
    pub requires_shipping: Option<bool>,
    pub published_at: Option<NaiveDate>,
    pub is_visible: Option<bool>,
    pub is_featured: Option<bool>,
    pub image: Option<String>,
    pub brand_id: Option<i32>,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl ProductUpdate {
    pub fn new() -> Self {
        Self {
            name: None,
            slug: None,
            sku: None,
            barcode: None,
            description: None,
            price: None,
            old_price: None,
            cost: None,
            quantity: None,
            security_stock: None,
            stock_status: None,
            backorder: None,
            requires_shipping: None,
            published_at: None,
            is_visible: None,
            is_featured: None,
            image: None,
            brand_id: None,
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

    pub fn price(mut self, price: i32) -> Self {
        self.price = Some(price);
        self
    }

    pub fn quantity(mut self, quantity: i32) -> Self {
        self.quantity = Some(quantity);
        self
    }

    pub fn is_visible(mut self, is_visible: bool) -> Self {
        self.is_visible = Some(is_visible);
        self
    }
}

impl Default for ProductUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::database::JsonApiResource for Product {
    const TYPE: &'static str = "products";
    type Repository = crate::database::ProductRepository;
    type NewModel = NewProduct;
    type UpdateModel = ProductUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "products"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "name",
            "slug",
            "sku",
            "barcode",
            "description",
            "price",
            "old_price",
            "cost",
            "quantity",
            "security_stock",
            "stock_status",
            "status",
            "backorder",
            "requires_shipping",
            "published_at",
            "is_visible",
            "is_featured",
            "image",
            "brand_id",
            "account_id",
            "created_at",
            "updated_at",
            "deleted_at",
        ]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        let price_dollars = self.price as f64 / 100.0;
        let old_price_dollars = self.old_price.map(|p| p as f64 / 100.0);
        let cost_dollars = self.cost.map(|c| c as f64 / 100.0);

        vec![
            ("name", json!(self.name)),
            ("slug", json!(self.slug)),
            ("sku", json!(self.sku)),
            ("barcode", json!(self.barcode)),
            ("description", json!(self.description)),
            ("price", json!(price_dollars)),
            ("old_price", json!(old_price_dollars)),
            ("cost", json!(cost_dollars)),
            ("quantity", json!(self.quantity)),
            ("security_stock", json!(self.security_stock)),
            ("stock_status", json!(self.stock_status)),
            ("backorder", json!(self.backorder)),
            ("requires_shipping", json!(self.requires_shipping)),
            ("published_at", json!(self.published_at)),
            ("is_visible", json!(self.is_visible)),
            ("is_featured", json!(self.is_featured)),
            ("image", json!(self.image)),
            ("brand_id", json!(self.brand_id)),
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
        crate::database::ProductRepository::new(pool)
    }
}
