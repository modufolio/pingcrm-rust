use crate::database::models::order_item::{NewOrderItem, OrderItem, OrderItemUpdate};
use crate::database::pool::DbPool;
use appkit_core::jsonapi::{PaginatedResult, QueryParams};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct OrderItemRepository {
    pool: DbPool,
}

impl OrderItemRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        item_id: i32,
    ) -> Result<Option<OrderItem>, diesel::result::Error> {
        use crate::database::schema::order_items::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        order_items
            .filter(id.eq(item_id))
            .first::<OrderItem>(&mut conn)
            .await
            .optional()
    }

    pub async fn find_by_order(
        &self,
        ord_id: i32,
    ) -> Result<Vec<OrderItem>, diesel::result::Error> {
        use crate::database::schema::order_items::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        order_items
            .filter(order_id.eq(ord_id))
            .load::<OrderItem>(&mut conn)
            .await
    }

    pub async fn find_by_product(
        &self,
        prod_id: i32,
    ) -> Result<Vec<OrderItem>, diesel::result::Error> {
        use crate::database::schema::order_items::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        order_items
            .filter(product_id.eq(prod_id))
            .load::<OrderItem>(&mut conn)
            .await
    }

    pub async fn create(&self, new_item: NewOrderItem) -> Result<OrderItem, diesel::result::Error> {
        use crate::database::schema::order_items::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(order_items)
            .values(&new_item)
            .get_result::<OrderItem>(&mut conn)
            .await
    }

    pub async fn update(
        &self,
        item_id: i32,
        item_update: OrderItemUpdate,
    ) -> Result<OrderItem, diesel::result::Error> {
        use crate::database::schema::order_items::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(order_items.filter(id.eq(item_id)))
            .set(&item_update)
            .get_result::<OrderItem>(&mut conn)
            .await
    }

    pub async fn delete(&self, item_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::order_items::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(order_items.filter(id.eq(item_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn delete_by_order(&self, ord_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::order_items::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(order_items.filter(order_id.eq(ord_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<OrderItem>, diesel::result::Error> {
        use crate::database::schema::order_items::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let total_count = order_items.count().get_result::<i64>(&mut conn).await?;

        let offset = (params.page.number - 1) * params.page.size;
        let items = order_items
            .order(id.asc())
            .limit(params.page.size)
            .offset(offset)
            .load::<OrderItem>(&mut conn)
            .await?;

        Ok(PaginatedResult::new(
            items,
            total_count,
            params.page.number,
            params.page.size,
        ))
    }
}

impl From<DbPool> for OrderItemRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<OrderItem> for OrderItemRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<OrderItem>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<OrderItem>, diesel::result::Error> {
        self.paginate(params).await
    }

    async fn create(&self, new_item: NewOrderItem) -> Result<OrderItem, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(
        &self,
        id: i32,
        update: OrderItemUpdate,
    ) -> Result<OrderItem, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<OrderItem>, diesel::result::Error> {
        use crate::database::schema::order_items;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        match foreign_key {
            "order_id" => {
                order_items::table
                    .filter(order_items::order_id.eq_any(&ids))
                    .load::<OrderItem>(&mut conn)
                    .await
            }
            "product_id" => {
                order_items::table
                    .filter(order_items::product_id.eq_any(&ids))
                    .load::<OrderItem>(&mut conn)
                    .await
            }
            _ => Ok(vec![]),
        }
    }
}
