use crate::database::models::User;
use crate::database::pool::DbPool;

use appkit_core::database::UserRepository as UserRepositoryTrait;
use appkit_core::error::{AppError, AppResult};
use appkit_core::security::user::User as SecurityUser;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

#[derive(Clone)]
pub struct DieselUserRepository {
    pool: DbPool,
}

impl DieselUserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryTrait for DieselUserRepository {
    async fn find_by_id(&self, user_id: i32) -> AppResult<Option<SecurityUser>> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::InternalServerError(format!("Database connection error: {}", e))
        })?;

        let user = users
            .filter(id.eq(user_id))
            .first::<User>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

        Ok(user.map(|u| u.to_security_user()))
    }

    async fn find_by_email(&self, user_email: &str) -> AppResult<Option<SecurityUser>> {
        use crate::database::schema::users::dsl::*;

        let mut conn = self.pool.get().await.map_err(|e| {
            AppError::InternalServerError(format!("Database connection error: {}", e))
        })?;

        let user = users
            .filter(email.eq(user_email))
            .first::<User>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

        Ok(user.map(|u| u.to_security_user()))
    }
}
