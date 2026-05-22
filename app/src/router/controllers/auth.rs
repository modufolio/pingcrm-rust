use crate::app::App;
use crate::application::auth::{CreateUserAction, LoginAction};
use crate::presenter::UserPresenter;
use appkit_core::error::AppResult;
use appkit_core::extractors::ValidatedJson;
use appkit_core::response::AppResponse;
use appkit_core::security::user::{CreateUserDto, LoginCredentials};
use axum::{extract::State, response::Response};

pub async fn login(
    State(state): State<App>,
    ValidatedJson(credentials): ValidatedJson<LoginCredentials>,
) -> AppResult<Response> {
    let action = LoginAction::new(state.db_pool.clone(), state.config.jwt_secret.clone());
    let result = action.execute(credentials).await?;

    if result.is_success() {
        Ok(AppResponse::json(result))
    } else {
        Ok(AppResponse::bad_request(
            result.message.unwrap_or_else(|| "Login failed".to_string()),
        ))
    }
}

pub async fn register(
    State(state): State<App>,
    ValidatedJson(dto): ValidatedJson<CreateUserDto>,
) -> AppResult<Response> {
    let action = CreateUserAction::new(state.db_pool.clone());
    let result = action.execute(dto).await?;

    if result.is_success() {
        let user = result.data.unwrap();
        let user_data = UserPresenter::from(&user);

        let jwt_secret = state.config.jwt_secret.clone();
        let user_repo = crate::database::DieselUserRepository::new(state.db_pool.clone());
        let jwt_auth =
            appkit_core::security::authenticator::JwtAuthenticator::new(jwt_secret, user_repo);
        let token = jwt_auth.generate_token(&user)?;

        Ok(AppResponse::created(
            serde_json::json!({
                "success": true,
                "token": token,
                "user": {
                    "id": user_data.id.to_string(),
                    "email": user_data.email,
                    "first_name": user_data.first_name,
                    "last_name": user_data.last_name,
                    "role": user_data.role,
                    "status": user_data.status,
                },
                "message": result.message,
            }),
            Some(format!("/api/users/{}", user.id)),
        ))
    } else {
        Ok(AppResponse::bad_request(
            result
                .message
                .unwrap_or_else(|| "Registration failed".to_string()),
        ))
    }
}
