use crate::database::UserRepository;
use crate::error::{AppError, AppResult};
use crate::http::cookie::{Cookie, SameSite};
use crate::security::authenticator::{AuthenticationResult, Authenticator};
use crate::security::token::{Token, UsernamePasswordToken};
use crate::security::user::User;
use async_trait::async_trait;
use axum::extract::Request;
use axum::http::HeaderMap;
use axum::response::Response;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;
use tower_sessions::Session;
use uuid::Uuid;

const REMEMBER_ME_COOKIE: &str = "remember_me";

pub const DEFAULT_REMEMBER_ME_DURATION: i64 = 30 * 24 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RememberMeToken {
    pub id: Uuid,

    pub user_id: i32,

    pub selector: String,

    pub validator_hash: String,

    pub expires_at: chrono::DateTime<Utc>,

    pub created_at: chrono::DateTime<Utc>,

    pub last_used_at: Option<chrono::DateTime<Utc>>,
}

impl RememberMeToken {
    pub fn new(user_id: i32, duration_seconds: i64) -> (Self, String) {
        let selector = Self::generate_random_string(16);
        let validator = Self::generate_random_string(32);
        let validator_hash = Self::hash_validator(&validator);

        let token = Self {
            id: Uuid::new_v4(),
            user_id,
            selector: selector.clone(),
            validator_hash,
            expires_at: Utc::now() + Duration::seconds(duration_seconds),
            created_at: Utc::now(),
            last_used_at: None,
        };

        let cookie_value = format!("{}:{}", selector, validator);
        (token, cookie_value)
    }

    fn generate_random_string(length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();

        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    fn hash_validator(validator: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(validator.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_validator(&self, validator: &str) -> bool {
        let hash = Self::hash_validator(validator);
        hash == self.validator_hash
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn parse_cookie_value(cookie_value: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = cookie_value.split(':').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }
}

#[async_trait]
pub trait RememberMeRepository: Send + Sync {
    async fn store(&self, token: RememberMeToken) -> AppResult<()>;

    async fn find_by_selector(&self, selector: &str) -> AppResult<Option<RememberMeToken>>;

    async fn delete(&self, token_id: Uuid) -> AppResult<()>;

    async fn delete_by_user(&self, user_id: i32) -> AppResult<()>;

    async fn delete_expired(&self) -> AppResult<usize>;

    async fn update_last_used(&self, token_id: Uuid) -> AppResult<()>;
}

#[derive(Clone)]
pub struct InMemoryRememberMeRepository {
    tokens: std::sync::Arc<tokio::sync::RwLock<Vec<RememberMeToken>>>,
}

impl InMemoryRememberMeRepository {
    pub fn new() -> Self {
        Self {
            tokens: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryRememberMeRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RememberMeRepository for InMemoryRememberMeRepository {
    async fn store(&self, token: RememberMeToken) -> AppResult<()> {
        let mut tokens = self.tokens.write().await;
        tokens.push(token);
        Ok(())
    }

    async fn find_by_selector(&self, selector: &str) -> AppResult<Option<RememberMeToken>> {
        let tokens = self.tokens.read().await;
        Ok(tokens.iter().find(|t| t.selector == selector).cloned())
    }

    async fn delete(&self, token_id: Uuid) -> AppResult<()> {
        let mut tokens = self.tokens.write().await;
        tokens.retain(|t| t.id != token_id);
        Ok(())
    }

    async fn delete_by_user(&self, user_id: i32) -> AppResult<()> {
        let mut tokens = self.tokens.write().await;
        tokens.retain(|t| t.user_id != user_id);
        Ok(())
    }

    async fn delete_expired(&self) -> AppResult<usize> {
        let mut tokens = self.tokens.write().await;
        let before = tokens.len();
        tokens.retain(|t| !t.is_expired());
        Ok(before - tokens.len())
    }

    async fn update_last_used(&self, token_id: Uuid) -> AppResult<()> {
        let mut tokens = self.tokens.write().await;
        if let Some(token) = tokens.iter_mut().find(|t| t.id == token_id) {
            token.last_used_at = Some(Utc::now());
        }
        Ok(())
    }
}

pub struct RememberMeAuthenticator {
    user_repository: std::sync::Arc<dyn UserRepository>,
    remember_me_repository: std::sync::Arc<dyn RememberMeRepository>,
    cookie_name: String,
    duration_seconds: i64,
}

impl RememberMeAuthenticator {
    pub fn new(
        user_repository: std::sync::Arc<dyn UserRepository>,
        remember_me_repository: std::sync::Arc<dyn RememberMeRepository>,
    ) -> Self {
        Self {
            user_repository,
            remember_me_repository,
            cookie_name: REMEMBER_ME_COOKIE.to_string(),
            duration_seconds: DEFAULT_REMEMBER_ME_DURATION,
        }
    }

    pub fn with_cookie_name(mut self, name: impl Into<String>) -> Self {
        self.cookie_name = name.into();
        self
    }

    pub fn with_duration(mut self, duration_seconds: i64) -> Self {
        self.duration_seconds = duration_seconds;
        self
    }

    fn extract_cookie(headers: &HeaderMap, cookie_name: &str) -> Option<String> {
        crate::http::cookie::CookieParser::get(headers, cookie_name)
    }

    pub async fn create_token(&self, user_id: i32) -> AppResult<String> {
        let (token, cookie_value) = RememberMeToken::new(user_id, self.duration_seconds);
        self.remember_me_repository.store(token).await?;
        Ok(cookie_value)
    }

    pub fn create_cookie(&self, cookie_value: &str) -> Cookie {
        Cookie::new(&self.cookie_name, cookie_value)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(StdDuration::from_secs(self.duration_seconds as u64))
    }

    pub async fn delete_user_tokens(&self, user_id: i32) -> AppResult<()> {
        self.remember_me_repository.delete_by_user(user_id).await
    }

    pub async fn cleanup_expired(&self) -> AppResult<usize> {
        self.remember_me_repository.delete_expired().await
    }
}

#[async_trait]
impl Authenticator for RememberMeAuthenticator {
    fn supports(&self, request: &Request) -> bool {
        Self::extract_cookie(request.headers(), &self.cookie_name).is_some()
    }

    async fn authenticate(
        &self,
        request: Request,
    ) -> Result<(AuthenticationResult, Request), AppError> {
        let cookie_value = match Self::extract_cookie(request.headers(), &self.cookie_name) {
            Some(value) => value,
            None => return Ok((AuthenticationResult::NoCredentials, request)),
        };

        let (selector, validator) = match RememberMeToken::parse_cookie_value(&cookie_value) {
            Some((s, v)) => (s, v),
            None => {
                tracing::warn!("Invalid remember me cookie format");
                return Ok((
                    AuthenticationResult::Failed("Invalid cookie format".to_string()),
                    request,
                ));
            }
        };

        let token = match self
            .remember_me_repository
            .find_by_selector(&selector)
            .await?
        {
            Some(t) => t,
            None => {
                tracing::warn!("Remember me token not found for selector: {}", selector);
                return Ok((
                    AuthenticationResult::Failed("Token not found".to_string()),
                    request,
                ));
            }
        };

        if token.is_expired() {
            tracing::warn!("Remember me token expired for user: {}", token.user_id);
            self.remember_me_repository.delete(token.id).await?;
            return Ok((
                AuthenticationResult::Failed("Token expired".to_string()),
                request,
            ));
        }

        if !token.verify_validator(&validator) {
            tracing::warn!("Invalid remember me validator for user: {}", token.user_id);

            self.remember_me_repository.delete(token.id).await?;
            return Ok((
                AuthenticationResult::Failed("Invalid token".to_string()),
                request,
            ));
        }

        let db_user = self.user_repository.find_by_id(token.user_id).await?;

        let result = match db_user {
            Some(security_user) => {
                if security_user.can_authenticate() {
                    tracing::info!(
                        "Remember me authentication successful for user: {}",
                        security_user.email
                    );

                    self.remember_me_repository
                        .update_last_used(token.id)
                        .await?;

                    AuthenticationResult::Success(security_user)
                } else {
                    tracing::warn!("User cannot authenticate: {}", security_user.email);
                    AuthenticationResult::Failed("User account is not active".to_string())
                }
            }
            None => {
                tracing::warn!("User not found for remember me token: {}", token.user_id);
                self.remember_me_repository.delete(token.id).await?;
                AuthenticationResult::Failed("User not found".to_string())
            }
        };

        Ok((result, request))
    }

    fn create_token(&self, user: User, firewall_name: String) -> Token {
        Token::UsernamePassword(UsernamePasswordToken::new(user, firewall_name))
    }

    fn on_authentication_failure(&self, _request: &Request, _error: AppError) -> Response {
        crate::response::AppResponse::unauthorized("Remember me authentication failed")
    }

    fn name(&self) -> &'static str {
        "remember_me"
    }
}

pub struct RememberMeHelper;

impl RememberMeHelper {
    pub async fn setup_remember_me(
        authenticator: &RememberMeAuthenticator,
        user_id: i32,
        session: &Session,
    ) -> AppResult<Cookie> {
        let cookie_value = authenticator.create_token(user_id).await?;

        let cookie = authenticator.create_cookie(&cookie_value);

        super::session_helpers::store_user_in_session(session, user_id).await?;

        Ok(cookie)
    }

    pub async fn clear_remember_me(
        authenticator: &RememberMeAuthenticator,
        user_id: i32,
    ) -> AppResult<Cookie> {
        authenticator.delete_user_tokens(user_id).await?;

        let cookie = Cookie::deleted(&authenticator.cookie_name);

        Ok(cookie)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let user_id = 1i32;
        let (token, cookie_value) = RememberMeToken::new(user_id, 3600);

        assert_eq!(token.user_id, user_id);
        assert!(!token.is_expired());
        assert!(cookie_value.contains(':'));
    }

    #[test]
    fn test_validator_verification() {
        let user_id = 1i32;
        let (token, cookie_value) = RememberMeToken::new(user_id, 3600);

        let (_, validator) = RememberMeToken::parse_cookie_value(&cookie_value).unwrap();
        assert!(token.verify_validator(&validator));
        assert!(!token.verify_validator("wrong"));
    }

    #[test]
    fn test_cookie_parsing() {
        let cookie_value = "selector123:validator456";
        let (selector, validator) = RememberMeToken::parse_cookie_value(cookie_value).unwrap();

        assert_eq!(selector, "selector123");
        assert_eq!(validator, "validator456");
    }

    #[tokio::test]
    async fn test_in_memory_repository() {
        let repo = InMemoryRememberMeRepository::new();
        let user_id = 1i32;
        let (token, _) = RememberMeToken::new(user_id, 3600);

        repo.store(token.clone()).await.unwrap();

        let found = repo.find_by_selector(&token.selector).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, token.id);

        repo.delete(token.id).await.unwrap();
        let found = repo.find_by_selector(&token.selector).await.unwrap();
        assert!(found.is_none());
    }
}
