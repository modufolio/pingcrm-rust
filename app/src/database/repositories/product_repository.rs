use crate::database::models::*;
use crate::database::pool::DbPool;
use crate::database::schema::products;
use appkit_core::jsonapi::{
    FilterCondition, FilterOperator, PaginatedResult, QueryParams, SortDirection,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::{debug, info, instrument};

#[derive(Clone)]
pub struct ProductRepository {
    pool: DbPool,
}

impl ProductRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        product_id: i32,
    ) -> Result<Option<Product>, diesel::result::Error> {
        use crate::database::schema::products::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        products
            .filter(id.eq(product_id))
            .first::<Product>(&mut conn)
            .await
            .optional()
    }

    pub async fn find_by_account(
        &self,
        acct_id: i32,
    ) -> Result<Vec<Product>, diesel::result::Error> {
        use crate::database::schema::products::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        products
            .filter(account_id.eq(acct_id))
            .load::<Product>(&mut conn)
            .await
    }

    pub async fn find_by_brand(
        &self,
        brand_id_param: i32,
    ) -> Result<Vec<Product>, diesel::result::Error> {
        use crate::database::schema::products::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        products
            .filter(brand_id.eq(brand_id_param))
            .load::<Product>(&mut conn)
            .await
    }

    pub async fn find_by_category(
        &self,
        category_id_param: i32,
    ) -> Result<Vec<Product>, diesel::result::Error> {
        use crate::database::schema::product_categories;
        use crate::database::schema::products::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let product_ids: Vec<i32> = product_categories::table
            .filter(product_categories::category_id.eq(category_id_param))
            .select(product_categories::product_id)
            .load::<i32>(&mut conn)
            .await?;

        products
            .filter(id.eq_any(product_ids))
            .load::<Product>(&mut conn)
            .await
    }

    pub async fn create(&self, new_product: NewProduct) -> Result<Product, diesel::result::Error> {
        use crate::database::schema::products::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(products)
            .values(&new_product)
            .get_result::<Product>(&mut conn)
            .await
    }

    pub async fn delete(&self, product_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::products::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(products.filter(id.eq(product_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn update(
        &self,
        product_id: i32,
        product_update: ProductUpdate,
    ) -> Result<Product, diesel::result::Error> {
        use crate::database::schema::products::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(products.filter(id.eq(product_id)))
            .set(&product_update)
            .get_result::<Product>(&mut conn)
            .await
    }

    #[instrument(skip(self, params), fields(page = params.page.number, size = params.page.size))]
    pub async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Product>, diesel::result::Error> {
        use crate::database::schema::products::dsl::*;

        debug!(
            "Starting product pagination query with {} filters",
            params.filters.len()
        );

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let mut count_query = products.into_boxed();
        count_query = count_query.filter(deleted_at.is_null());
        for filter in &params.filters {
            count_query = Self::apply_product_filter(count_query, filter);
        }

        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        debug!(total = total, "Counted total products with filters");

        let mut query = products.into_boxed();
        query = query.filter(deleted_at.is_null());
        for filter in &params.filters {
            query = Self::apply_product_filter(query, filter);
        }

        query = Self::apply_product_sort(query, &params.sort);

        let offset = (params.page.number - 1) * params.page.size;
        query = query.limit(params.page.size).offset(offset);

        let items = query.load::<Product>(&mut conn).await?;

        info!(
            returned = items.len(),
            total = total,
            page = params.page.number,
            filters = params.filters.len(),
            "Product pagination completed"
        );

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
    ) -> Result<PaginatedResult<Product>, diesel::result::Error> {
        use crate::database::schema::products::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let search_pattern = search.filter(|s| !s.is_empty()).map(|s| format!("%{}%", s));

        let mut count_query = products
            .filter(account_id.eq(acct_id))
            .filter(deleted_at.is_null())
            .into_boxed();
        if let Some(ref pattern) = search_pattern {
            count_query = count_query.filter(
                name.like(pattern.clone())
                    .or(description.like(pattern.clone())),
            );
        }
        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        let mut query = products
            .filter(account_id.eq(acct_id))
            .filter(deleted_at.is_null())
            .into_boxed();
        if let Some(ref pattern) = search_pattern {
            query = query.filter(
                name.like(pattern.clone())
                    .or(description.like(pattern.clone())),
            );
        }
        query = Self::apply_product_sort(query, &params.sort);
        let offset = (params.page.number - 1) * params.page.size;
        query = query.limit(params.page.size).offset(offset);

        let items = query.load::<Product>(&mut conn).await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    fn apply_product_filter(
        query: products::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        filter: &FilterCondition,
    ) -> products::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::products::dsl::*;

        match filter.field.as_str() {
            "name" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(name.eq(value)),
                        FilterOperator::Neq => query.filter(name.ne(value)),
                        FilterOperator::Like => query.filter(name.like(format!("%{}%", value))),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "slug" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(slug.eq(value)),
                        FilterOperator::Neq => query.filter(slug.ne(value)),
                        FilterOperator::Like => query.filter(slug.like(format!("%{}%", value))),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "stock_status" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(stock_status.eq(value)),
                        FilterOperator::Neq => query.filter(stock_status.ne(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "status" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(stock_status.eq(value)),
                        FilterOperator::Neq => query.filter(stock_status.ne(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "is_visible" => {
                if let Some(ref value) = filter.value {
                    if let Ok(bool_value) = value.parse::<bool>() {
                        match filter.operator {
                            FilterOperator::Eq => query.filter(is_visible.eq(bool_value)),
                            FilterOperator::Neq => query.filter(is_visible.ne(bool_value)),
                            _ => query,
                        }
                    } else {
                        query
                    }
                } else {
                    query
                }
            }
            "is_featured" => {
                if let Some(ref value) = filter.value {
                    if let Ok(bool_value) = value.parse::<bool>() {
                        match filter.operator {
                            FilterOperator::Eq => query.filter(is_featured.eq(bool_value)),
                            FilterOperator::Neq => query.filter(is_featured.ne(bool_value)),
                            _ => query,
                        }
                    } else {
                        query
                    }
                } else {
                    query
                }
            }
            "brand_id" => {
                if let Some(ref value) = filter.value {
                    if let Ok(id_value) = value.parse::<i32>() {
                        match filter.operator {
                            FilterOperator::Eq => query.filter(brand_id.eq(id_value)),
                            FilterOperator::Neq => query.filter(brand_id.ne(id_value)),
                            _ => query,
                        }
                    } else {
                        query
                    }
                } else {
                    query
                }
            }
            "account_id" => {
                if let Some(ref value) = filter.value {
                    if let Ok(id_value) = value.parse::<i32>() {
                        match filter.operator {
                            FilterOperator::Eq => query.filter(account_id.eq(id_value)),
                            FilterOperator::Neq => query.filter(account_id.ne(id_value)),
                            _ => query,
                        }
                    } else {
                        query
                    }
                } else {
                    query
                }
            }
            _ => query,
        }
    }

    fn apply_product_sort(
        query: products::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        sorts: &[(String, SortDirection)],
    ) -> products::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::products::dsl::*;

        if sorts.is_empty() {
            return query.order(created_at.desc());
        }

        let mut query = query;
        for (field, direction) in sorts {
            query = match (field.as_str(), direction) {
                ("name", SortDirection::Ascending) => query.order(name.asc()),
                ("name", SortDirection::Descending) => query.order(name.desc()),
                ("price", SortDirection::Ascending) => query.order(price.asc()),
                ("price", SortDirection::Descending) => query.order(price.desc()),
                ("created_at", SortDirection::Ascending) => query.order(created_at.asc()),
                ("created_at", SortDirection::Descending) => query.order(created_at.desc()),
                ("updated_at", SortDirection::Ascending) => query.order(updated_at.asc()),
                ("updated_at", SortDirection::Descending) => query.order(updated_at.desc()),
                ("quantity", SortDirection::Ascending) => query.order(quantity.asc()),
                ("quantity", SortDirection::Descending) => query.order(quantity.desc()),
                _ => query,
            };
        }
        query
    }
}

impl From<DbPool> for ProductRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<Product> for ProductRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<Product>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Product>, diesel::result::Error> {
        self.paginate(params).await
    }

    async fn create(&self, new_item: NewProduct) -> Result<Product, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(
        &self,
        id: i32,
        update: ProductUpdate,
    ) -> Result<Product, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<Product>, diesel::result::Error> {
        use crate::database::schema::products;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        match foreign_key {
            "brand_id" => {
                products::table
                    .filter(products::brand_id.eq_any(&ids))
                    .filter(products::deleted_at.is_null())
                    .load::<Product>(&mut conn)
                    .await
            }
            "account_id" => {
                products::table
                    .filter(products::account_id.eq_any(&ids))
                    .filter(products::deleted_at.is_null())
                    .load::<Product>(&mut conn)
                    .await
            }
            _ => Ok(vec![]),
        }
    }
}
