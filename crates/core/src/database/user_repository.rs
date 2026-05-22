use crate::error::AppResult;
use crate::security::user::User;

use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, user_id: i32) -> AppResult<Option<User>>;

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
}
