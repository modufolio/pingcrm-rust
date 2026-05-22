use crate::error::AppError;
use crate::security::token::Token;
use tower_sessions::Session;

const SESSION_USER_ID_KEY: &str = "user_id";

pub async fn extract_user_id_from_session(session: Session) -> Option<i32> {
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
        .cycle_id()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to cycle session id: {}", e)))?;
    session
        .insert(SESSION_USER_ID_KEY, user_id)
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to store user in session: {}", e)))?;
    Ok(())
}

pub async fn store_token_in_session(session: &Session, token: &Token) -> Result<(), AppError> {
    let user_id = token.get_user().id;
    store_user_in_session(session, user_id).await
}

pub async fn invalidate_session(session: &Session) -> Result<(), AppError> {
    session
        .flush()
        .await
        .map_err(|e| AppError::InternalError(format!("Failed to invalidate session: {}", e)))?;
    Ok(())
}
