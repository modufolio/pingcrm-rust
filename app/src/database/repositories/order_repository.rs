use crate::database::models::order::{NewOrder, Order, OrderUpdate};
use crate::database::pool::DbPool;
use appkit_core::jsonapi::{PaginatedResult, QueryParams};

use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct OrderRepository {
    pool: DbPool,
}

impl OrderRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, order_id: i32) -> Result<Option<Order>, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        orders
            .filter(id.eq(order_id))
            .filter(deleted_at.is_null())
            .first::<Order>(&mut conn)
            .await
            .optional()
    }

    pub async fn find_by_account(&self, acct_id: i32) -> Result<Vec<Order>, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        orders
            .filter(account_id.eq(acct_id))
            .filter(deleted_at.is_null())
            .order(created_at.desc())
            .load::<Order>(&mut conn)
            .await
    }

    pub async fn find_by_customer(
        &self,
        cust_id: i32,
    ) -> Result<Vec<Order>, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        orders
            .filter(customer_id.eq(cust_id))
            .filter(deleted_at.is_null())
            .order(created_at.desc())
            .load::<Order>(&mut conn)
            .await
    }

    pub async fn find_by_number(
        &self,
        order_number: &str,
    ) -> Result<Option<Order>, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        orders
            .filter(number.eq(order_number))
            .filter(deleted_at.is_null())
            .first::<Order>(&mut conn)
            .await
            .optional()
    }

    pub async fn create(&self, new_order: NewOrder) -> Result<Order, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(orders)
            .values(&new_order)
            .get_result::<Order>(&mut conn)
            .await
    }

    pub async fn update(
        &self,
        order_id: i32,
        order_update: OrderUpdate,
    ) -> Result<Order, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(orders.filter(id.eq(order_id)))
            .set(&order_update)
            .get_result::<Order>(&mut conn)
            .await
    }

    pub async fn delete(&self, order_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(orders.filter(id.eq(order_id)))
            .set(deleted_at.eq(Some(Utc::now().naive_utc())))
            .execute(&mut conn)
            .await
    }

    pub async fn hard_delete(&self, order_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(orders.filter(id.eq(order_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn restore(&self, order_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(orders.filter(id.eq(order_id)))
            .set(deleted_at.eq(None::<chrono::NaiveDateTime>))
            .execute(&mut conn)
            .await
    }

    pub async fn find_all_with_deleted(
        &self,
        acct_id: i32,
    ) -> Result<Vec<Order>, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        orders
            .filter(account_id.eq(acct_id))
            .order(created_at.desc())
            .load::<Order>(&mut conn)
            .await
    }

    pub async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Order>, diesel::result::Error> {
        use crate::database::schema::orders::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let total_count = orders
            .filter(deleted_at.is_null())
            .count()
            .get_result::<i64>(&mut conn)
            .await?;

        let offset = (params.page.number - 1) * params.page.size;
        let items = orders
            .filter(deleted_at.is_null())
            .order(created_at.desc())
            .limit(params.page.size)
            .offset(offset)
            .load::<Order>(&mut conn)
            .await?;

        Ok(PaginatedResult::new(
            items,
            total_count,
            params.page.number,
            params.page.size,
        ))
    }
}

impl From<DbPool> for OrderRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<Order> for OrderRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<Order>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Order>, diesel::result::Error> {
        self.paginate(params).await
    }

    async fn create(&self, new_item: NewOrder) -> Result<Order, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(&self, id: i32, update: OrderUpdate) -> Result<Order, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<Order>, diesel::result::Error> {
        use crate::database::schema::orders;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        match foreign_key {
            "customer_id" => {
                orders::table
                    .filter(orders::customer_id.eq_any(&ids))
                    .filter(orders::deleted_at.is_null())
                    .load::<Order>(&mut conn)
                    .await
            }
            "account_id" => {
                orders::table
                    .filter(orders::account_id.eq_any(&ids))
                    .filter(orders::deleted_at.is_null())
                    .load::<Order>(&mut conn)
                    .await
            }
            _ => Ok(vec![]),
        }
    }
}
