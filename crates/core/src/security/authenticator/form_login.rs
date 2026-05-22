use crate::database::UserRepository;
use crate::error::{AppError, AppResult};
use crate::response::AppResponse;
use crate::security::authenticator::{AuthenticationResult, Authenticator};
use crate::security::csrf::CsrfTokenManager;

#[allow(unused_imports)]
use crate::security::authenticator::SessionAuthenticator;
use crate::security::token::{Token, UsernamePasswordToken};
use crate::security::user::User;
use async_trait::async_trait;
use axum::extract::Request;
use axum::http::Method;
use axum::response::Response;
use http_body_util::BodyExt;
use serde::Deserialize;

#[allow(unused_imports)]
use std::collections::HashMap;
use std::sync::Arc;
use tower_sessions::Session;

#[derive(Debug, Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub csrf_token: Option<String>,
}

pub trait BruteForceProtection: Send + Sync {
    fn is_locked(&self, identifier: &str) -> bool;

    fn record_failure(&mut self, identifier: &str);

    fn record_success(&mut self, identifier: &str);

    fn remaining_attempts(&self, identifier: &str) -> u32;
}

pub struct InMemoryBruteForceProtection {
    failures: Arc<tokio::sync::RwLock<HashMap<String, u32>>>,
    max_attempts: u32,
}

impl InMemoryBruteForceProtection {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            failures: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            max_attempts,
        }
    }
}

impl BruteForceProtection for InMemoryBruteForceProtection {
    fn is_locked(&self, identifier: &str) -> bool {
        let failures = self.failures.blocking_read();
        failures
            .get(identifier)
            .is_some_and(|count| *count >= self.max_attempts)
    }

    fn record_failure(&mut self, identifier: &str) {
        let mut failures = self.failures.blocking_write();
        let count = failures.entry(identifier.to_string()).or_insert(0);
        *count += 1;
        tracing::warn!(
            "Failed login attempt for {}: {} of {} attempts",
            identifier,
            count,
            self.max_attempts
        );
    }

    fn record_success(&mut self, identifier: &str) {
        let mut failures = self.failures.blocking_write();
        failures.remove(identifier);
        tracing::info!("Successful login for {}, cleared failure count", identifier);
    }

    fn remaining_attempts(&self, identifier: &str) -> u32 {
        let failures = self.failures.blocking_read();
        let count = failures.get(identifier).copied().unwrap_or(0);
        self.max_attempts.saturating_sub(count)
    }
}

pub struct FormLoginAuthenticator {
    user_repository: Arc<dyn UserRepository>,
    brute_force: Arc<tokio::sync::RwLock<Box<dyn BruteForceProtection>>>,
}

impl FormLoginAuthenticator {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        brute_force: Arc<tokio::sync::RwLock<Box<dyn BruteForceProtection>>>,
    ) -> Self {
        Self {
            user_repository,
            brute_force,
        }
    }

    async fn extract_credentials(request: Request) -> AppResult<(LoginCredentials, Request)> {
        use axum::body::Body;

        let (parts, body) = request.into_parts();

        let content_type = parts
            .headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let bytes = body
            .collect()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read body: {}", e)))?
            .to_bytes();

        let credentials = if content_type.contains("application/json") {
            serde_json::from_slice(&bytes)
                .map_err(|e| AppError::BadRequest(format!("Invalid JSON: {}", e)))?
        } else if content_type.contains("application/x-www-form-urlencoded") {
            let form_str = String::from_utf8(bytes.to_vec())
                .map_err(|e| AppError::BadRequest(format!("Invalid UTF-8 in form data: {}", e)))?;

            serde_urlencoded::from_str(&form_str)
                .map_err(|e| AppError::BadRequest(format!("Invalid form data: {}", e)))?
        } else {
            return Err(AppError::BadRequest(
                "Content-Type must be application/json or application/x-www-form-urlencoded"
                    .to_string(),
            ));
        };

        let request = Request::from_parts(parts, Body::empty());

        Ok((credentials, request))
    }

    async fn validate_csrf_token(
        session: Option<&Session>,
        credentials: &LoginCredentials,
    ) -> AppResult<()> {
        let provided_token = match &credentials.csrf_token {
            Some(token) => token,
            None => {
                tracing::debug!("No CSRF token provided, skipping validation");
                return Ok(());
            }
        };

        let session = match session {
            Some(s) => s,
            None => {
                tracing::warn!("CSRF token provided but no session exists");
                return Err(AppError::BadRequest("Invalid CSRF token".to_string()));
            }
        };

        CsrfTokenManager::validate_token(session, provided_token).await
    }
}

#[async_trait]
impl Authenticator for FormLoginAuthenticator {
    fn supports(&self, request: &Request) -> bool {
        request.method() == Method::POST && request.uri().path() == "/login"
    }

    async fn authenticate(
        &self,
        request: Request,
    ) -> Result<(AuthenticationResult, Request), AppError> {
        let (credentials, request) = match Self::extract_credentials(request).await {
            Ok((creds, req)) => (creds, req),
            Err(e) => {
                tracing::debug!("Failed to extract credentials: {}", e);

                return Err(e);
            }
        };

        let session = request.extensions().get::<Session>();
        if let Err(e) = Self::validate_csrf_token(session, &credentials).await {
            tracing::warn!("CSRF validation failed: {}", e);
            return Ok((
                AuthenticationResult::Failed("Invalid CSRF token".to_string()),
                request,
            ));
        }

        {
            let bf = self.brute_force.read().await;
            if bf.is_locked(&credentials.email) {
                tracing::warn!(
                    "Account locked due to too many failures: {}",
                    credentials.email
                );
                return Ok((
                    AuthenticationResult::Failed(
                        "Account temporarily locked due to too many failed attempts".to_string(),
                    ),
                    request,
                ));
            }
        }

        let db_user = match self
            .user_repository
            .find_by_email(&credentials.email)
            .await?
        {
            Some(u) => u,
            None => {
                self.brute_force
                    .write()
                    .await
                    .record_failure(&credentials.email);
                return Ok((
                    AuthenticationResult::Failed("Invalid credentials".to_string()),
                    request,
                ));
            }
        };

        if !db_user.can_authenticate() {
            tracing::warn!(
                "User {} cannot authenticate (status: {:?})",
                db_user.email,
                db_user.status
            );
            return Ok((
                AuthenticationResult::Failed("Account is not active".to_string()),
                request,
            ));
        }

        if !db_user.verify_password(&credentials.password) {
            tracing::info!("Invalid password for user: {}", db_user.email);
            self.brute_force
                .write()
                .await
                .record_failure(&credentials.email);
            return Ok((
                AuthenticationResult::Failed("Invalid credentials".to_string()),
                request,
            ));
        }

        self.brute_force
            .write()
            .await
            .record_success(&credentials.email);

        if db_user.two_factor_enabled {
            tracing::info!("2FA required for user: {}", db_user.email);

            if let Some(session) = request.extensions().get::<Session>() {
                if let Err(e) = session.insert("2fa_user_id", db_user.id).await {
                    tracing::error!("Failed to store 2FA user ID in session: {}", e);
                    return Err(AppError::InternalError(
                        "Failed to initiate 2FA flow".to_string(),
                    ));
                }
            }

            return Ok((
                AuthenticationResult::Response(AppResponse::redirect("/2fa")),
                request,
            ));
        }

        tracing::info!("Form login successful for user: {}", db_user.email);
        Ok((AuthenticationResult::Success(db_user), request))
    }

    fn create_token(&self, user: User, firewall_name: String) -> Token {
        Token::UsernamePassword(UsernamePasswordToken::new(user, firewall_name))
    }

    fn on_authentication_failure(&self, _request: &Request, error: AppError) -> Response {
        AppResponse::unauthorized(error.to_string())
    }

    fn on_authentication_success(&self, _request: &Request, _token: &Token) -> Option<Response> {
        None
    }

    fn name(&self) -> &'static str {
        "form_login"
    }
}
