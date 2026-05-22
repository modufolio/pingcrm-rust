use crate::error::{AppError, AppResult};
use tower_sessions::Session;
use uuid::Uuid;

const CSRF_TOKEN_KEY: &str = "csrf_token";

pub struct CsrfTokenManager;

impl CsrfTokenManager {
    pub async fn generate_token(session: &Session) -> AppResult<String> {
        let token = Uuid::new_v4().to_string();

        session
            .insert(CSRF_TOKEN_KEY, token.clone())
            .await
            .map_err(|e| {
                AppError::InternalError(format!("Failed to store CSRF token in session: {}", e))
            })?;

        Ok(token)
    }

    pub async fn get_token(session: &Session) -> AppResult<Option<String>> {
        session.get::<String>(CSRF_TOKEN_KEY).await.map_err(|e| {
            AppError::InternalError(format!("Failed to retrieve CSRF token from session: {}", e))
        })
    }

    pub async fn validate_token(session: &Session, provided_token: &str) -> AppResult<()> {
        let stored_token = Self::get_token(session).await?;

        match stored_token {
            Some(token) if token == provided_token => {
                Self::rotate_token(session).await?;
                Ok(())
            }
            Some(_) => Err(AppError::BadRequest("Invalid CSRF token".to_string())),
            None => Err(AppError::BadRequest("No CSRF token in session".to_string())),
        }
    }

    async fn rotate_token(session: &Session) -> AppResult<()> {
        let new_token = Uuid::new_v4().to_string();

        session
            .insert(CSRF_TOKEN_KEY, new_token)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to rotate CSRF token: {}", e)))?;

        Ok(())
    }

    pub async fn get_or_create_token(session: &Session) -> AppResult<String> {
        match Self::get_token(session).await? {
            Some(token) => Ok(token),
            None => Self::generate_token(session).await,
        }
    }

    pub async fn clear_token(session: &Session) -> AppResult<()> {
        session
            .remove::<String>(CSRF_TOKEN_KEY)
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to clear CSRF token: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_generate_token() {}

    #[tokio::test]
    async fn test_validate_token() {}

    #[tokio::test]
    async fn test_invalid_token() {}
}
