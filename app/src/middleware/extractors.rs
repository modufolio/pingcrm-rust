use appkit_core::error::AppError;
use appkit_core::security::user::User;

pub struct CurrentUser(pub User);

impl<S> axum::extract::FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<User>()
            .cloned()
            .map(CurrentUser)
            .ok_or_else(|| AppError::AuthorizationFailed("Not authenticated".to_string()))
    }
}
