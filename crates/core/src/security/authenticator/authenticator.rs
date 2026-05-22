use super::{JwtAuthenticator, SessionAuthenticator};
use crate::database::UserRepository;
use crate::error::AppError;
use crate::security::token::Token;
use crate::security::user::User;
use async_trait::async_trait;
use axum::extract::Request;
use axum::response::Response;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum AuthenticationResult {
    Success(User),

    Failed(String),

    NoCredentials,

    Response(Response),
}

#[async_trait]
pub trait Authenticator: Send + Sync {
    fn supports(&self, request: &Request) -> bool;

    async fn authenticate(
        &self,
        request: Request,
    ) -> Result<(AuthenticationResult, Request), AppError>;

    fn create_token(&self, user: User, firewall_name: String) -> Token;

    fn on_authentication_failure(&self, request: &Request, error: AppError) -> Response;

    fn on_authentication_success(&self, _request: &Request, _token: &Token) -> Option<Response> {
        None
    }

    fn name(&self) -> &'static str;
}

pub enum AuthenticatorType<R>
where
    R: UserRepository + Clone + Send + Sync + 'static,
{
    Jwt(JwtAuthenticator<R>),
    Session(SessionAuthenticator<R>),
}

impl<R> AuthenticatorType<R>
where
    R: UserRepository + Clone + Send + Sync + 'static,
{
    pub fn supports(&self, request: &Request) -> bool {
        match self {
            AuthenticatorType::Jwt(auth) => auth.supports(request),
            AuthenticatorType::Session(auth) => auth.supports(request),
        }
    }

    pub async fn authenticate(
        &self,
        request: Request,
    ) -> Result<(AuthenticationResult, Request), AppError> {
        match self {
            AuthenticatorType::Jwt(auth) => auth.authenticate(request).await,
            AuthenticatorType::Session(auth) => auth.authenticate(request).await,
        }
    }

    pub fn create_token(&self, user: User, firewall_name: String) -> Token {
        match self {
            AuthenticatorType::Jwt(auth) => auth.create_token(user, firewall_name),
            AuthenticatorType::Session(auth) => auth.create_token(user, firewall_name),
        }
    }

    pub fn on_authentication_failure(&self, request: &Request, error: AppError) -> Response {
        match self {
            AuthenticatorType::Jwt(auth) => auth.on_authentication_failure(request, error),
            AuthenticatorType::Session(auth) => auth.on_authentication_failure(request, error),
        }
    }

    pub fn on_authentication_success(&self, request: &Request, token: &Token) -> Option<Response> {
        match self {
            AuthenticatorType::Jwt(auth) => auth.on_authentication_success(request, token),
            AuthenticatorType::Session(auth) => auth.on_authentication_success(request, token),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AuthenticatorType::Jwt(auth) => auth.name(),
            AuthenticatorType::Session(auth) => auth.name(),
        }
    }
}

pub struct AuthenticatorChain<R>
where
    R: UserRepository + Clone + Send + Sync + 'static,
{
    authenticators: Vec<AuthenticatorType<R>>,
    _phantom: PhantomData<R>,
}

impl<R> AuthenticatorChain<R>
where
    R: UserRepository + Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            authenticators: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn add_jwt(mut self, authenticator: JwtAuthenticator<R>) -> Self {
        self.authenticators
            .push(AuthenticatorType::Jwt(authenticator));
        self
    }

    pub fn add_session(mut self, authenticator: SessionAuthenticator<R>) -> Self {
        self.authenticators
            .push(AuthenticatorType::Session(authenticator));
        self
    }

    pub async fn try_authenticate(
        &self,
        mut request: Request,
        firewall_name: String,
    ) -> Result<Option<(Token, Option<Response>, Request)>, AppError> {
        for authenticator in &self.authenticators {
            if authenticator.supports(&request) {
                tracing::debug!("Trying authenticator: {}", authenticator.name());

                let (result, req) = authenticator.authenticate(request).await?;
                request = req;

                match result {
                    AuthenticationResult::Success(user) => {
                        let token = authenticator.create_token(user, firewall_name.clone());

                        let response = authenticator.on_authentication_success(&request, &token);

                        return Ok(Some((token, response, request)));
                    }
                    AuthenticationResult::Response(response) => {
                        return Ok(Some((
                            Token::UsernamePassword(
                                crate::security::token::UsernamePasswordToken {
                                    user: crate::security::user::User {
                                        id: 0,
                                        email: String::new(),
                                        password_hash: String::new(),
                                        first_name: String::new(),
                                        last_name: String::new(),
                                        role: crate::security::user::UserRole::Guest,
                                        status: crate::security::user::UserStatus::Active,
                                        created_at: chrono::Utc::now(),
                                        updated_at: chrono::Utc::now(),
                                        last_login_at: None,
                                        failed_login_attempts: 0,
                                        totp_secret: None,
                                        two_factor_enabled: false,
                                        account_id: None,
                                    },
                                    firewall_name: firewall_name.clone(),
                                    roles: vec![],
                                    authenticated: false,
                                },
                            ),
                            Some(response),
                            request,
                        )));
                    }
                    AuthenticationResult::Failed(msg) => {
                        tracing::debug!("Authenticator {} failed: {}", authenticator.name(), msg);
                        continue;
                    }
                    AuthenticationResult::NoCredentials => continue,
                }
            }
        }

        Ok(None)
    }
}

impl<R> Default for AuthenticatorChain<R>
where
    R: UserRepository + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
