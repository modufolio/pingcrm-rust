use crate::database::UserRepository;
use crate::error::{AppError, AppResult};
use crate::response::AppResponse;
use crate::security::authenticator::{AuthenticationResult, Authenticator};
use crate::security::token::{Token, UsernamePasswordToken};
use crate::security::user::{User, UserRole};
use async_trait::async_trait;
use axum::extract::Request;
use axum::response::Response;
use chrono::Duration;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtClaims {
    pub sub: i32,

    pub email: String,

    pub role: UserRole,

    pub iat: i64,

    pub exp: i64,
}

impl JwtClaims {
    pub fn new(user: &User, expiration_hours: i64) -> Self {
        let now = chrono::Utc::now();
        let exp = now + Duration::hours(expiration_hours);

        Self {
            sub: user.id,
            email: user.email.clone(),
            role: user.role.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.exp < now
    }
}

pub struct JwtAuthenticator<R> {
    secret: String,
    user_repository: R,
    _phantom: PhantomData<R>,
}

impl<R> JwtAuthenticator<R>
where
    R: UserRepository + Clone + Send + Sync + 'static,
{
    pub fn new(secret: String, user_repository: R) -> Self {
        Self {
            secret,
            user_repository,
            _phantom: PhantomData,
        }
    }

    pub fn generate_token(&self, user: &User) -> AppResult<String> {
        let claims = JwtClaims::new(user, 24);

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalServerError(format!("Failed to generate token: {}", e)))?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> AppResult<JwtClaims> {
        let mut validation = Validation::default();
        validation.validate_aud = false;

        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| AppError::AuthenticationFailed(format!("Invalid token: {}", e)))?;

        if token_data.claims.is_expired() {
            return Err(AppError::AuthenticationFailed("Token expired".to_string()));
        }

        Ok(token_data.claims)
    }
}

impl<R> JwtAuthenticator<R> {
    fn extract_bearer_token(request: &Request) -> Option<String> {
        request
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| {
                if s.starts_with("Bearer ") {
                    Some(s[7..].to_string())
                } else {
                    None
                }
            })
    }

    #[allow(dead_code)]
    fn extract_token_owned(request: &Request) -> Option<String> {
        Self::extract_bearer_token(request)
    }
}

#[async_trait]
impl<R> Authenticator for JwtAuthenticator<R>
where
    R: UserRepository + Clone + Send + Sync + 'static,
{
    fn supports(&self, request: &Request) -> bool {
        Self::extract_bearer_token(request).is_some()
    }

    async fn authenticate(
        &self,
        request: Request,
    ) -> Result<(AuthenticationResult, Request), AppError> {
        let token = match Self::extract_bearer_token(&request) {
            Some(t) => t,
            None => return Ok((AuthenticationResult::NoCredentials, request)),
        };

        let claims = match self.verify_token(&token) {
            Ok(c) => c,
            Err(e) => return Ok((AuthenticationResult::Failed(e.to_string()), request)),
        };

        let db_user = self.user_repository.find_by_id(claims.sub).await?;

        let result = match db_user {
            Some(security_user) => {
                if security_user.can_authenticate() {
                    AuthenticationResult::Success(security_user)
                } else {
                    AuthenticationResult::Failed("User account is not active".to_string())
                }
            }
            None => AuthenticationResult::Failed("User not found".to_string()),
        };

        Ok((result, request))
    }

    fn create_token(&self, user: User, firewall_name: String) -> Token {
        Token::UsernamePassword(UsernamePasswordToken::new(user, firewall_name))
    }

    fn on_authentication_failure(&self, _request: &Request, error: AppError) -> Response {
        AppResponse::unauthorized(error.to_string())
    }

    fn name(&self) -> &'static str {
        "jwt"
    }
}
