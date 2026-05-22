use crate::database::UserRepository;
use crate::error::AppError;
use crate::response::AppResponse;
use crate::security::authenticator::{AuthenticationResult, Authenticator};
use crate::security::token::{Token, UsernamePasswordToken};
use crate::security::user::User;
use async_trait::async_trait;
use axum::extract::Request;
use axum::response::Response;
use std::marker::PhantomData;
use tower_sessions::Session;

const SESSION_USER_ID_KEY: &str = "user_id";

pub struct SessionAuthenticator<R> {
    user_repository: R,
    _phantom: PhantomData<R>,
}

impl<R> SessionAuthenticator<R>
where
    R: UserRepository + Clone + Send + Sync + 'static,
{
    pub fn new(user_repository: R) -> Self {
        Self {
            user_repository,
            _phantom: PhantomData,
        }
    }
}

impl<R> SessionAuthenticator<R> {
    fn extract_session(request: &Request) -> Option<Session> {
        request.extensions().get::<Session>().cloned()
    }

    async fn extract_user_id_from_session(session: Session) -> Option<i32> {
        match session.get::<i32>(SESSION_USER_ID_KEY).await {
            Ok(user_id) => user_id,
            Err(e) => {
                tracing::debug!("Failed to extract user_id from session: {}", e);
                None
            }
        }
    }

    pub async fn store_user_in_session(session: &Session, user_id: i32) -> Result<(), AppError> {
        session
            .insert(SESSION_USER_ID_KEY, user_id)
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Failed to store user in session: {}", e))
            })?;
        Ok(())
    }

    pub async fn store_token_in_session(session: &Session, token: &Token) -> Result<(), AppError> {
        let user_id = token.get_user().id;
        Self::store_user_in_session(session, user_id).await
    }

    pub async fn invalidate_session(session: &Session) -> Result<(), AppError> {
        session
            .flush()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to invalidate session: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl<R> Authenticator for SessionAuthenticator<R>
where
    R: UserRepository + Clone + Send + Sync + 'static,
{
    fn supports(&self, request: &Request) -> bool {
        request.extensions().get::<Session>().is_some()
    }

    async fn authenticate(
        &self,
        request: Request,
    ) -> Result<(AuthenticationResult, Request), AppError> {
        let session = match Self::extract_session(&request) {
            Some(s) => s,
            None => return Ok((AuthenticationResult::NoCredentials, request)),
        };

        let user_id = match Self::extract_user_id_from_session(session).await {
            Some(id) => id,
            None => return Ok((AuthenticationResult::NoCredentials, request)),
        };

        let db_user = self.user_repository.find_by_id(user_id).await?;

        let result = match db_user {
            Some(security_user) => {
                if security_user.can_authenticate() {
                    tracing::debug!(
                        "Session authentication successful for user: {}",
                        security_user.email
                    );
                    AuthenticationResult::Success(security_user)
                } else {
                    AuthenticationResult::Failed("User account is not active".to_string())
                }
            }
            None => AuthenticationResult::Failed("Session user not found".to_string()),
        };

        Ok((result, request))
    }

    fn create_token(&self, user: User, firewall_name: String) -> Token {
        Token::UsernamePassword(UsernamePasswordToken::new(user, firewall_name))
    }

    fn on_authentication_failure(&self, _request: &Request, _error: AppError) -> Response {
        AppResponse::redirect("/login")
    }

    fn name(&self) -> &'static str {
        "session"
    }
}
