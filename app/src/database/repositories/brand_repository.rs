use crate::database::models::*;
use crate::database::pool::DbPool;
use crate::database::schema::brands;
use appkit_core::jsonapi::{PaginatedResult, QueryParams, SortDirection};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct BrandRepository {
    pool: DbPool,
}

impl BrandRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, brand_id: i32) -> Result<Option<Brand>, diesel::result::Error> {
        use crate::database::schema::brands::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        brands
            .filter(id.eq(brand_id))
            .first::<Brand>(&mut conn)
            .await
            .optional()
    }

    pub async fn find_by_account(&self, acct_id: i32) -> Result<Vec<Brand>, diesel::result::Error> {
        use crate::database::schema::brands::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        brands
            .filter(account_id.eq(acct_id))
            .load::<Brand>(&mut conn)
            .await
    }

    pub async fn create(&self, new_brand: NewBrand) -> Result<Brand, diesel::result::Error> {
        use crate::database::schema::brands::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(brands)
            .values(&new_brand)
            .get_result::<Brand>(&mut conn)
            .await
    }

    pub async fn update(
        &self,
        brand_id: i32,
        brand_update: BrandUpdate,
    ) -> Result<Brand, diesel::result::Error> {
        use crate::database::schema::brands::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(brands.filter(id.eq(brand_id)))
            .set(&brand_update)
            .get_result::<Brand>(&mut conn)
            .await
    }

    pub async fn delete(&self, brand_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::brands::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(brands.filter(id.eq(brand_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Brand>, diesel::result::Error> {
        use crate::database::schema::brands::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let total = brands.count().get_result::<i64>(&mut conn).await?;

        let offset = (params.page.number - 1) * params.page.size;
        let items = brands
            .order(created_at.desc())
            .limit(params.page.size)
            .offset(offset)
            .load::<Brand>(&mut conn)
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
    ) -> Result<PaginatedResult<Brand>, diesel::result::Error> {
        use crate::database::schema::brands::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let search_pattern = search.filter(|s| !s.is_empty()).map(|s| format!("%{}%", s));

        let mut count_query = brands.filter(account_id.eq(acct_id)).into_boxed();
        if let Some(ref pattern) = search_pattern {
            count_query = count_query.filter(name.like(pattern.clone()));
        }
        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        let mut query = brands.filter(account_id.eq(acct_id)).into_boxed();
        if let Some(ref pattern) = search_pattern {
            query = query.filter(name.like(pattern.clone()));
        }
        query = Self::apply_brand_sort(query, &params.sort);
        let offset = (params.page.number - 1) * params.page.size;
        query = query.limit(params.page.size).offset(offset);

        let items = query.load::<Brand>(&mut conn).await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    fn apply_brand_sort(
        query: brands::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        sorts: &[(String, SortDirection)],
    ) -> brands::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::brands::dsl::*;

        if sorts.is_empty() {
            return query.then_order_by(name.asc());
        }

        let mut sorted = query;
        for (field, direction) in sorts {
            sorted = match (field.as_str(), direction) {
                ("name", SortDirection::Ascending) => sorted.then_order_by(name.asc()),
                ("name", SortDirection::Descending) => sorted.then_order_by(name.desc()),
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

impl From<DbPool> for BrandRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<Brand> for BrandRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<Brand>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Brand>, diesel::result::Error> {
        self.paginate(params).await
    }

    async fn create(&self, new_item: NewBrand) -> Result<Brand, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(&self, id: i32, update: BrandUpdate) -> Result<Brand, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<Brand>, diesel::result::Error> {
        use crate::database::schema::brands;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        match foreign_key {
            "account_id" => {
                brands::table
                    .filter(brands::account_id.eq_any(&ids))
                    .load::<Brand>(&mut conn)
                    .await
            }
            _ => Ok(vec![]),
        }
    }
}
