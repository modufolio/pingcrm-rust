use crate::database::models::*;
use crate::database::pool::DbPool;
use crate::database::schema::organizations;
use appkit_core::jsonapi::{
    FilterCondition, FilterOperator, PaginatedResult, QueryParams, SortDirection,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct OrganizationRepository {
    pool: DbPool,
}

impl OrganizationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(
        &self,
        org_id: i32,
    ) -> Result<Option<Organization>, diesel::result::Error> {
        use crate::database::schema::organizations::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        organizations
            .filter(id.eq(org_id))
            .first::<Organization>(&mut conn)
            .await
            .optional()
    }

    pub async fn find_by_account(
        &self,
        acct_id: i32,
    ) -> Result<Vec<Organization>, diesel::result::Error> {
        use crate::database::schema::organizations::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        organizations
            .filter(account_id.eq(acct_id))
            .load::<Organization>(&mut conn)
            .await
    }

    pub async fn create(
        &self,
        new_org: NewOrganization,
    ) -> Result<Organization, diesel::result::Error> {
        use crate::database::schema::organizations::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(organizations)
            .values(&new_org)
            .get_result::<Organization>(&mut conn)
            .await
    }

    pub async fn delete(&self, org_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::organizations::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(organizations.filter(id.eq(org_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn update(
        &self,
        org_id: i32,
        org_update: OrganizationUpdate,
    ) -> Result<Organization, diesel::result::Error> {
        use crate::database::schema::organizations::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(organizations.filter(id.eq(org_id)))
            .set(&org_update)
            .get_result::<Organization>(&mut conn)
            .await
    }

    pub async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Organization>, diesel::result::Error> {
        use crate::database::schema::organizations::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let mut count_query = organizations.into_boxed();
        for filter in &params.filters {
            count_query = Self::apply_organization_filter(count_query, filter);
        }
        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        let mut query = organizations.into_boxed();
        for filter in &params.filters {
            query = Self::apply_organization_filter(query, filter);
        }
        query = Self::apply_org_sort(query, &params.sort);

        let offset = (params.page.number - 1) * params.page.size;
        let items = query
            .limit(params.page.size)
            .offset(offset)
            .load::<Organization>(&mut conn)
            .await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    fn apply_organization_filter(
        query: organizations::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        filter: &FilterCondition,
    ) -> organizations::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::organizations::dsl::*;

        let Some(value) = filter.value.clone() else {
            return query;
        };

        match filter.field.as_str() {
            "name" => match filter.operator {
                FilterOperator::Eq => query.filter(name.eq(value)),
                FilterOperator::Neq => query.filter(name.ne(value)),
                FilterOperator::Like => query.filter(name.like(value)),
                _ => query,
            },
            "email" => match filter.operator {
                FilterOperator::Eq => query.filter(email.eq(value)),
                FilterOperator::Neq => query.filter(email.ne(value)),
                FilterOperator::Like => query.filter(email.like(value)),
                _ => query,
            },
            "city" => match filter.operator {
                FilterOperator::Eq => query.filter(city.eq(value)),
                FilterOperator::Neq => query.filter(city.ne(value)),
                FilterOperator::Like => query.filter(city.like(value)),
                _ => query,
            },
            "country" => match filter.operator {
                FilterOperator::Eq => query.filter(country.eq(value)),
                FilterOperator::Neq => query.filter(country.ne(value)),
                _ => query,
            },
            "account_id" => match value.parse::<i32>() {
                Ok(v) => match filter.operator {
                    FilterOperator::Eq => query.filter(account_id.eq(Some(v))),
                    FilterOperator::Neq => query.filter(account_id.ne(Some(v))),
                    _ => query,
                },
                Err(_) => query,
            },
            "id" => match value.parse::<i32>() {
                Ok(v) => match filter.operator {
                    FilterOperator::Eq => query.filter(id.eq(v)),
                    FilterOperator::Neq => query.filter(id.ne(v)),
                    FilterOperator::Gt => query.filter(id.gt(v)),
                    FilterOperator::Gte => query.filter(id.ge(v)),
                    FilterOperator::Lt => query.filter(id.lt(v)),
                    FilterOperator::Lte => query.filter(id.le(v)),
                    _ => query,
                },
                Err(_) => query,
            },
            _ => query,
        }
    }

    pub async fn find_with_params(
        &self,
        acct_id: i32,
        search: Option<&str>,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Organization>, diesel::result::Error> {
        use crate::database::schema::organizations::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let search_pattern = search.filter(|s| !s.is_empty()).map(|s| format!("%{}%", s));

        let mut count_query = organizations
            .filter(account_id.eq(Some(acct_id)))
            .into_boxed();
        if let Some(ref pattern) = search_pattern {
            count_query =
                count_query.filter(name.like(pattern.clone()).or(email.like(pattern.clone())));
        }
        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        let mut query = organizations
            .filter(account_id.eq(Some(acct_id)))
            .into_boxed();
        if let Some(ref pattern) = search_pattern {
            query = query.filter(name.like(pattern.clone()).or(email.like(pattern.clone())));
        }
        query = Self::apply_org_sort(query, &params.sort);
        let offset = (params.page.number - 1) * params.page.size;
        query = query.limit(params.page.size).offset(offset);

        let items = query.load::<Organization>(&mut conn).await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    fn apply_org_sort(
        query: organizations::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        sorts: &[(String, SortDirection)],
    ) -> organizations::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::organizations::dsl::*;

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
                ("city", SortDirection::Ascending) => sorted.then_order_by(city.asc()),
                ("city", SortDirection::Descending) => sorted.then_order_by(city.desc()),
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

impl From<DbPool> for OrganizationRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<Organization> for OrganizationRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<Organization>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<Organization>, diesel::result::Error> {
        self.paginate(params).await
    }

    async fn create(
        &self,
        new_item: NewOrganization,
    ) -> Result<Organization, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(
        &self,
        id: i32,
        update: OrganizationUpdate,
    ) -> Result<Organization, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<Organization>, diesel::result::Error> {
        use crate::database::schema::organizations;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        match foreign_key {
            "account_id" => {
                organizations::table
                    .filter(organizations::account_id.eq_any(&ids))
                    .load::<Organization>(&mut conn)
                    .await
            }
            _ => Ok(vec![]),
        }
    }
}
