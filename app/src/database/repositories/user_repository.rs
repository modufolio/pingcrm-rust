use crate::database::models::*;
use crate::database::pool::DbPool;
use crate::database::schema::users;
use appkit_core::jsonapi::{
    FilterCondition, FilterOperator, PaginatedResult, QueryParams, SortDirection,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, user_id: i32) -> Result<Option<User>, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        users
            .filter(id.eq(user_id))
            .first::<User>(&mut conn)
            .await
            .optional()
    }

    pub async fn find_by_email(
        &self,
        user_email: &str,
    ) -> Result<Option<User>, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        users
            .filter(email.eq(user_email))
            .first::<User>(&mut conn)
            .await
            .optional()
    }

    pub async fn create(&self, new_user: NewUser) -> Result<User, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::insert_into(users)
            .values(&new_user)
            .get_result::<User>(&mut conn)
            .await
    }

    pub async fn update(
        &self,
        user_id: i32,
        user_update: UserUpdate,
    ) -> Result<User, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(users.filter(id.eq(user_id)))
            .set(&user_update)
            .get_result::<User>(&mut conn)
            .await
    }

    pub async fn delete(&self, user_id: i32) -> Result<usize, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::delete(users.filter(id.eq(user_id)))
            .execute(&mut conn)
            .await
    }

    pub async fn find_all(&self) -> Result<Vec<User>, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        users.load::<User>(&mut conn).await
    }

    pub async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<User>, diesel::result::Error> {
        self.find_with_params(params).await
    }

    pub async fn find_with_params(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<User>, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        let mut count_query = users.into_boxed();
        for filter in &params.filters {
            count_query = Self::apply_user_filter(count_query, filter);
        }

        let total = count_query.count().get_result::<i64>(&mut conn).await?;

        let mut query = users.into_boxed();
        for filter in &params.filters {
            query = Self::apply_user_filter(query, filter);
        }

        query = Self::apply_user_sort(query, &params.sort);

        let offset = (params.page.number - 1) * params.page.size;
        query = query.limit(params.page.size).offset(offset);

        let items = query.load::<User>(&mut conn).await?;

        Ok(PaginatedResult::new(
            items,
            total,
            params.page.number,
            params.page.size,
        ))
    }

    fn apply_user_filter(
        query: users::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        filter: &FilterCondition,
    ) -> users::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::users::dsl::*;

        match filter.field.as_str() {
            "email" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(email.eq(value)),
                        FilterOperator::Neq => query.filter(email.ne(value)),
                        FilterOperator::Like => query.filter(email.like(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "first_name" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(first_name.eq(value)),
                        FilterOperator::Neq => query.filter(first_name.ne(value)),
                        FilterOperator::Like => query.filter(first_name.like(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "last_name" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(last_name.eq(value)),
                        FilterOperator::Neq => query.filter(last_name.ne(value)),
                        FilterOperator::Like => query.filter(last_name.like(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "account_status" => {
                if let Some(ref value) = filter.value {
                    let value = value.clone();
                    match filter.operator {
                        FilterOperator::Eq => query.filter(account_status.eq(value)),
                        FilterOperator::Neq => query.filter(account_status.ne(value)),
                        _ => query,
                    }
                } else {
                    query
                }
            }
            "enabled" => {
                if let Some(ref value) = filter.value {
                    if let Ok(bool_value) = value.parse::<bool>() {
                        match filter.operator {
                            FilterOperator::Eq => query.filter(enabled.eq(bool_value)),
                            FilterOperator::Neq => query.filter(enabled.ne(bool_value)),
                            _ => query,
                        }
                    } else {
                        query
                    }
                } else {
                    query
                }
            }
            "locked" => {
                if let Some(ref value) = filter.value {
                    if let Ok(bool_value) = value.parse::<bool>() {
                        match filter.operator {
                            FilterOperator::Eq => query.filter(locked.eq(bool_value)),
                            FilterOperator::Neq => query.filter(locked.ne(bool_value)),
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

    fn apply_user_sort(
        query: users::BoxedQuery<'static, diesel::sqlite::Sqlite>,
        sorts: &[(String, SortDirection)],
    ) -> users::BoxedQuery<'static, diesel::sqlite::Sqlite> {
        use crate::database::schema::users::dsl::*;

        let mut sorted_query = query;

        for (field, direction) in sorts {
            sorted_query = match (field.as_str(), direction) {
                ("email", SortDirection::Ascending) => sorted_query.then_order_by(email.asc()),
                ("email", SortDirection::Descending) => sorted_query.then_order_by(email.desc()),
                ("first_name", SortDirection::Ascending) => {
                    sorted_query.then_order_by(first_name.asc())
                }
                ("first_name", SortDirection::Descending) => {
                    sorted_query.then_order_by(first_name.desc())
                }
                ("last_name", SortDirection::Ascending) => {
                    sorted_query.then_order_by(last_name.asc())
                }
                ("last_name", SortDirection::Descending) => {
                    sorted_query.then_order_by(last_name.desc())
                }
                ("created_at", SortDirection::Ascending) => {
                    sorted_query.then_order_by(created_at.asc())
                }
                ("created_at", SortDirection::Descending) => {
                    sorted_query.then_order_by(created_at.desc())
                }
                ("updated_at", SortDirection::Ascending) => {
                    sorted_query.then_order_by(updated_at.asc())
                }
                ("updated_at", SortDirection::Descending) => {
                    sorted_query.then_order_by(updated_at.desc())
                }
                _ => sorted_query,
            };
        }

        sorted_query
    }

    pub async fn count(&self) -> Result<i64, diesel::result::Error> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        users.count().get_result(&mut conn).await
    }

    pub async fn lock_user(
        &self,
        user_id: i32,
        reason: Option<String>,
    ) -> Result<(), diesel::result::Error> {
        use crate::database::schema::users::dsl::*;
        use chrono::Utc;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(users.filter(id.eq(user_id)))
            .set((
                locked.eq(true),
                locked_at.eq(Some(Utc::now().naive_utc())),
                locked_reason.eq(reason),
                updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn unlock_user(&self, user_id: i32) -> Result<(), diesel::result::Error> {
        use crate::database::schema::users::dsl::*;
        use chrono::Utc;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(users.filter(id.eq(user_id)))
            .set((
                locked.eq(false),
                locked_at.eq(None::<chrono::NaiveDateTime>),
                locked_reason.eq(None::<String>),
                updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .await?;

        Ok(())
    }

    pub async fn find(&self, user_id: i32) -> Result<Option<User>, diesel::result::Error> {
        self.find_by_id(user_id).await
    }
}

impl From<DbPool> for UserRepository {
    fn from(pool: DbPool) -> Self {
        Self::new(pool)
    }
}

#[async_trait::async_trait]
impl appkit_core::database::UserRepository for UserRepository {
    async fn find_by_id(
        &self,
        user_id: i32,
    ) -> appkit_core::error::AppResult<Option<appkit_core::security::user::User>> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            appkit_core::error::AppError::InternalServerError(format!(
                "Database connection error: {}",
                e
            ))
        })?;

        let db_user: Option<User> = users
            .filter(id.eq(user_id))
            .first::<User>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                appkit_core::error::AppError::InternalServerError(format!("Database error: {}", e))
            })?;

        Ok(db_user.map(|u| u.to_security_user()))
    }

    async fn find_by_email(
        &self,
        user_email: &str,
    ) -> appkit_core::error::AppResult<Option<appkit_core::security::user::User>> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            appkit_core::error::AppError::InternalServerError(format!(
                "Database connection error: {}",
                e
            ))
        })?;

        let db_user: Option<User> = users
            .filter(email.eq(user_email))
            .first::<User>(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                appkit_core::error::AppError::InternalServerError(format!("Database error: {}", e))
            })?;

        Ok(db_user.map(|u| u.to_security_user()))
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiRepository<User> for UserRepository {
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, diesel::result::Error> {
        self.find_by_id(id).await
    }

    async fn paginate(
        &self,
        params: &QueryParams,
    ) -> Result<PaginatedResult<User>, diesel::result::Error> {
        self.paginate(params).await
    }

    async fn create(&self, new_item: NewUser) -> Result<User, diesel::result::Error> {
        self.create(new_item).await
    }

    async fn update(&self, id: i32, update: UserUpdate) -> Result<User, diesel::result::Error> {
        self.update(id, update).await
    }

    async fn delete(&self, id: i32) -> Result<(), diesel::result::Error> {
        self.delete(id).await.map(|_| ())
    }

    async fn load_by_foreign_key_in(
        &self,
        foreign_key: &str,
        ids: Vec<i32>,
    ) -> Result<Vec<User>, diesel::result::Error> {
        use crate::database::schema::users;

        let mut conn = self.pool.get().await.map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UnableToSendCommand,
                Box::new(e.to_string()),
            )
        })?;

        match foreign_key {
            "account_id" => {
                users::table
                    .filter(users::account_id.eq_any(&ids))
                    .load::<User>(&mut conn)
                    .await
            }
            _ => Ok(vec![]),
        }
    }
}
