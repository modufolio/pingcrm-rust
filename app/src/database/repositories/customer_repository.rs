use crate::database::models::*;
use crate::database::pool::DbPool;
use crate::database::schema::customers;
use appkit_core::jsonapi::{PaginatedResult, QueryParams, SortDirection};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct CustomerRepository {
    pool: DbPool,
}

impl CustomerRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        customer_id: i32,
    ) -> Result<Option<Customer>, diesel::result::Error> {
        use crate::database::schema::customers::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        customers
            .filter(id.eq(customer_id))
            .first::<Customer>(&mut conn)
            .await
            .optional()
    }

    pub async fn find_by_account(
        &self,
        acct_id: i32,
    ) -> Result<Vec<Customer>, diesel::result::Error> {
        use crate::database::schema::customers::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        customers
            .filter(account_id.eq(acct_id))
            .load::<Customer>(&mut conn)
            .await
    }

    pub async fn find_by_email(
        &self,
        customer_email: &str,
    ) -> Result<Option<Customer>, diesel::result::Error> {
        use crate::database::schema::customers::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        customers
            .filter(email.eq(customer_email))
            .first::<Customer>(&mut conn)
            .await
            .optional()
    }

    pub async fn create(
        &self,
        new_customer: NewCustomer,
    ) -> Result<Customer, diesel::result::Error> {
        use crate::database::schema::customers::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(customers)
            .values(&new_customer)
            .get_result::<Customer>(&mut conn)
            .await
    }

    pub async fn delete(&self, customer_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::customers::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(customers.filter(id.eq(customer_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn update(
        &self,
        customer_id: i32,
        customer_update: CustomerUpdate,
    ) -> Result<Customer, diesel::result::Error> {
        use crate::database::schema::customers::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(customers.filter(id.eq(customer_id)))
            .set(&customer_update)
            .get_result::<Customer>(&mut conn)
            .await
    }

    pub async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Customer>, diesel::result::Error> {
        use crate::database::schema::customers::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let total = customers.count().get_result::<i64>(&mut conn).await?;

        let offset = (params.page.number - 1) * params.page.size;
        let items = customers
            .order(created_at.desc())
            .limit(params.page.size)
            .offset(offset)
            .load::<Customer>(&mut conn)
            .await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    pub async fn find_with_params(
        &self,
        acct_id: i32,
        search: Option<&str>,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Customer>, diesel::result::Error> {
        use crate::database::schema::customers::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let search_pattern = search.filter(|s| !s.is_empty()).map(|s| format!("%{}%", s));

        let mut count_query = customers.filter(account_id.eq(acct_id)).into_boxed();
        if let Some(ref pattern) = search_pattern {
            count_query =
                count_query.filter(name.like(pattern.clone()).or(email.like(pattern.clone())));
        }
        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        let mut query = customers.filter(account_id.eq(acct_id)).into_boxed();
        if let Some(ref pattern) = search_pattern {
            query = query.filter(name.like(pattern.clone()).or(email.like(pattern.clone())));
        }
        query = Self::apply_customer_sort(query, &params.sort);
        let offset = (params.page.number - 1) * params.page.size;
        query = query.limit(params.page.size).offset(offset);

        let items = query.load::<Customer>(&mut conn).await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    fn apply_customer_sort(
        query: customers::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        sorts: &[(String, SortDirection)],
    ) -> customers::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::customers::dsl::*;

        if sorts.is_empty() {
            return query.then_order_by(name.asc());
        }

        let mut sorted = query;
        for (field, direction) in sorts {
            sorted = match (field.as_str(), direction) {
                ("name", SortDirection::Ascending) => sorted.then_order_by(name.asc()),
                ("name", SortDirection::Descending) => sorted.then_order_by(name.desc()),
                ("email", SortDirection::Ascending) => sorted.then_order_by(email.asc()),
                ("email", SortDirection::Descending) => sorted.then_order_by(email.desc()),
                ("created_at", SortDirection::Ascending) => sorted.then_order_by(created_at.asc()),
                ("created_at", SortDirection::Descending) => {
                    sorted.then_order_by(created_at.desc())
                }
                _ => sorted,
            };
        }
        sorted
    }
}

impl From<DbPool> for CustomerRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<Customer> for CustomerRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<Customer>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Customer>, diesel::result::Error> {
        self.paginate(params).await
    }

    async fn create(&self, new_item: NewCustomer) -> Result<Customer, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(
        &self,
        id: i32,
        update: CustomerUpdate,
    ) -> Result<Customer, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<Customer>, diesel::result::Error> {
        use crate::database::schema::customers;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        match foreign_key {
            "account_id" => {
                customers::table
                    .filter(customers::account_id.eq_any(&ids))
                    .load::<Customer>(&mut conn)
                    .await
            }
            _ => Ok(vec![]),
        }
    }
}
