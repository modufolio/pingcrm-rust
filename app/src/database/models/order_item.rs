use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = order_items)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct OrderItem {
    pub id: i32,
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_price: i32,
    pub total: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = order_items)]
pub struct NewOrderItem {
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_price: i32,
    pub total: i32,
}

impl NewOrderItem {
    pub fn new(order_id: i32, product_id: i32, quantity: i32, unit_price: i32) -> Self {
        Self {
            order_id,
            product_id,
            quantity,
            unit_price,
            total: quantity * unit_price,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = order_items)]
pub struct OrderItemUpdate {
    pub product_id: Option<i32>,
    pub quantity: Option<i32>,
    pub unit_price: Option<i32>,
    pub total: Option<i32>,
}

impl OrderItemUpdate {
    pub fn new() -> Self {
        Self {
            product_id: None,
            quantity: None,
            unit_price: None,
            total: None,
        }
    }

    pub fn quantity(mut self, quantity: i32) -> Self {
        self.quantity = Some(quantity);
        self
    }

    pub fn unit_price(mut self, unit_price: i32) -> Self {
        self.unit_price = Some(unit_price);
        self
    }

    pub fn total(mut self, total: i32) -> Self {
        self.total = Some(total);
        self
    }

    pub fn product_id(mut self, product_id: i32) -> Self {
        self.product_id = Some(product_id);
        self
    }
}

impl Default for OrderItemUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::database::JsonApiResource for OrderItem {
    const TYPE: &'static str = "order_items";
    type Repository = crate::database::OrderItemRepository;
    type NewModel = NewOrderItem;
    type UpdateModel = OrderItemUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "order_items"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "order_id",
            "product_id",
            "quantity",
            "unit_price",
            "total",
        ]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        vec![
            ("order_id", json!(self.order_id)),
            ("product_id", json!(self.product_id)),
            ("quantity", json!(self.quantity)),
            ("unit_price", json!(self.unit_price)),
            ("total", json!(self.total)),
        ]
    }

    fn repository(pool: crate::database::pool::DbPool) -> Self::Repository {
        crate::database::OrderItemRepository::new(pool)
    }
}
