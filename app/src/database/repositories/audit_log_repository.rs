use crate::database::models::*;
use crate::database::pool::DbPool;
use crate::database::schema::audit_logs;
use appkit_core::jsonapi::{
    FilterCondition, FilterOperator, PaginatedResult, QueryParams, SortDirection,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct AuditLogRepository {
    pool: DbPool,
}

impl AuditLogRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, log_id: i32) -> Result<Option<AuditLog>, diesel::result::Error> {
        use crate::database::schema::audit_logs::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        audit_logs
            .filter(id.eq(log_id))
            .first::<AuditLog>(&mut conn)
            .await
            .optional()
    }

    pub async fn create(&self, new_log: NewAuditLog) -> Result<AuditLog, diesel::result::Error> {
        use crate::database::schema::audit_logs::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(audit_logs)
            .values(&new_log)
            .get_result::<AuditLog>(&mut conn)
            .await
    }

    pub async fn find_with_params(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<AuditLog>, diesel::result::Error> {
        use crate::database::schema::audit_logs::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let mut count_query = audit_logs.into_boxed();
        for filter in &params.filters {
            count_query = Self::apply_audit_filter(count_query, filter);
        }

        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        let mut query = audit_logs.into_boxed();
        for filter in &params.filters {
            query = Self::apply_audit_filter(query, filter);
        }

        query = Self::apply_audit_sort(query, &params.sort);

        let offset = (params.page.number - 1) * params.page.size;
        query = query.limit(params.page.size).offset(offset);

        let items = query.load::<AuditLog>(&mut conn).await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    fn apply_audit_filter(
        query: audit_logs::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        filter: &FilterCondition,
    ) -> audit_logs::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::audit_logs::dsl::*;

        match filter.field.as_str() {
            "entity" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(entity.eq(value)),
                        FilterOperator::Neq => query.filter(entity.ne(value)),
                        FilterOperator::Like => query.filter(entity.like(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "action" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(action.eq(value)),
                        FilterOperator::Neq => query.filter(action.ne(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "user_id" => {
                if let Some(ref value) = filter.value {
                    if let Ok(id_value) = value.parse::<i32>() {
                        match filter.operator {
                            FilterOperator::Eq => query.filter(user_id.eq(id_value)),
                            FilterOperator::Neq => query.filter(user_id.ne(id_value)),
                            _ => query,
                        }
                    } else {
                        query
                    }
                } else {
                    match filter.operator {
                        FilterOperator::Null => query.filter(user_id.is_null()),
                        FilterOperator::NotNull => query.filter(user_id.is_not_null()),
                        _ => query,
                    }
                }
            }
            "entity_id" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(entity_id.eq(value)),
                        FilterOperator::Neq => query.filter(entity_id.ne(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            _ => query,
        }
    }

    fn apply_audit_sort(
        query: audit_logs::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        sorts: &[(String, SortDirection)],
    ) -> audit_logs::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::audit_logs::dsl::*;

        let mut sorted_query = query;

        for (field, direction) in sorts {
            sorted_query = match (field.as_str(), direction) {
                ("entity", SortDirection::Ascending) => sorted_query.then_order_by(entity.asc()),
                ("entity", SortDirection::Descending) => sorted_query.then_order_by(entity.desc()),
                ("action", SortDirection::Ascending) => sorted_query.then_order_by(action.asc()),
                ("action", SortDirection::Descending) => sorted_query.then_order_by(action.desc()),
                ("created_at", SortDirection::Ascending) => {
                    sorted_query.then_order_by(created_at.asc())
                }
                ("created_at", SortDirection::Descending) => {
                    sorted_query.then_order_by(created_at.desc())
                }
                _ => sorted_query,
            };
        }

        sorted_query
    }

    pub async fn find_by_user(
        &self,
        user_id_param: i32,
        params: &QueryParams,
    ) -> Result<PaginatedResult<AuditLog>, diesel::result::Error> {
        use crate::database::schema::audit_logs::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let mut count_query = audit_logs.filter(user_id.eq(user_id_param)).into_boxed();
        for filter in &params.filters {
            count_query = Self::apply_audit_filter(count_query, filter);
        }

        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        let mut query = audit_logs.filter(user_id.eq(user_id_param)).into_boxed();
        for filter in &params.filters {
            query = Self::apply_audit_filter(query, filter);
        }

        query = Self::apply_audit_sort(query, &params.sort);

        let offset = (params.page.number - 1) * params.page.size;
        query = query.limit(params.page.size).offset(offset);

        let items = query.load::<AuditLog>(&mut conn).await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    pub async fn count(&self) -> Result<i64, diesel::result::Error> {
        use crate::database::schema::audit_logs::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        audit_logs.count().get_result(&mut conn).await
    }

    pub async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<AuditLog>, diesel::result::Error> {
        self.find_with_params(params).await
    }

    pub async fn update(
        &self,
        log_id: i32,
        log_update: AuditLogUpdate,
    ) -> Result<AuditLog, diesel::result::Error> {
        use crate::database::schema::audit_logs::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(audit_logs.filter(id.eq(log_id)))
            .set(&log_update)
            .get_result::<AuditLog>(&mut conn)
            .await
    }

    pub async fn delete(&self, log_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::audit_logs::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(audit_logs.filter(id.eq(log_id)))
            .execute(&mut conn)
            .await
    }
}

impl From<DbPool> for AuditLogRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<AuditLog> for AuditLogRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<AuditLog>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<AuditLog>, diesel::result::Error> {
        self.paginate(params).await
    }

    async fn create(&self, new_item: NewAuditLog) -> Result<AuditLog, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(
        &self,
        id: i32,
        update: AuditLogUpdate,
    ) -> Result<AuditLog, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<AuditLog>, diesel::result::Error> {
        use crate::database::schema::audit_logs;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        match foreign_key {
            "user_id" => {
                audit_logs::table
                    .filter(audit_logs::user_id.eq_any(&ids))
                    .load::<AuditLog>(&mut conn)
                    .await
            }
            _ => Ok(vec![]),
        }
    }
}
