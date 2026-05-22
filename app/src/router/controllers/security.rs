use axum::{body::Body, extract::State, http::Request, response::Response, Json};
use serde::Deserialize;
use serde_json::json;
use tower_sessions::Session;

use crate::app::App;
use crate::database::UserRepository;
use appkit_core::{error::AppResult, inertia::SharedProps, response::AppResponse};

#[derive(Debug, Deserialize)]
pub struct FormLoginCredentials {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub remember: bool,
    #[serde(rename = "_csrf_token")]
    pub csrf_token: Option<String>,
}

const SESSION_USER_ID_KEY: &str = "user_id";

pub async fn login(
    State(state): State<App>,
    session: Session,
    request: Request<Body>,
) -> AppResult<Response> {
    let has_2fa_token: bool = session
        .get::<i32>("2fa_user_id")
        .await
        .ok()
        .flatten()
        .is_some();

    if has_2fa_token {
        return Ok(AppResponse::redirect("/2fa"));
    }

    let user_id: Option<i32> = session.get::<i32>(SESSION_USER_ID_KEY).await.ok().flatten();

    if let Some(uid) = user_id {
        let repo = UserRepository::new(state.db_pool.clone());
        if let Ok(Some(_user)) = repo.find_by_id(uid).await {
            return Ok(AppResponse::redirect("/"));
        }
    }

    let mut shared_props = SharedProps::new();

    if let Ok(Some(error)) = session.get::<String>("flash_error").await {
        shared_props = shared_props.with_field_error("email".to_string(), error);

        let _ = session.remove::<String>("flash_error").await;
    }

    if let Ok(Some(info)) = session.get::<String>("flash_info").await {
        let _ = info;
        let _ = session.remove::<String>("flash_info").await;
    }

    // Reuse the session-bound CSRF token across renders (matches the
    // Symfony CsrfTokenManager pattern). Without this, every GET /login
    // would overwrite the session token and invalidate any token already
    // captured in a mounted Vue form (e.g. on back-nav or re-render).
    let csrf_token = match session.get::<String>("csrf_token").await {
        Ok(Some(existing)) if !existing.is_empty() => existing,
        _ => {
            let token = generate_csrf_token();
            let _ = session.insert("csrf_token", token.clone()).await;
            token
        }
    };

    let props = shared_props.merge_with(json!({
        "csrf_token": csrf_token,
    }));

    let response = AppResponse::inertia("Auth/Login")
        .with_props(props)
        .render(&request);

    Ok(response)
}

pub async fn form_login(
    State(state): State<App>,
    session: Session,
    Json(credentials): Json<FormLoginCredentials>,
) -> AppResult<Response> {
    tracing::info!(
        "form_login received: email={} password_len={}",
        credentials.email,
        credentials.password.len()
    );

    // CSRF: the GET /login handler issued a token bound to this session.
    // Reject submissions that omit it or send a value that no longer matches.
    // The browser's SameSite=Strict cookie policy is the primary defence; this
    // is a server-side belt-and-braces check.
    let submitted_token = credentials.csrf_token.as_deref().unwrap_or("");
    if !validate_csrf_token(&session, submitted_token).await {
        tracing::warn!("form_login: CSRF token mismatch for email={}", credentials.email);
        let _ = session
            .insert("flash_error", "Your session expired. Please try again.")
            .await;
        return Ok(AppResponse::redirect("/login"));
    }

    let repo = UserRepository::new(state.db_pool.clone());
    let db_user = match repo.find_by_email(&credentials.email).await {
        Ok(Some(user)) => {
            tracing::info!("form_login: found user id={} email={}", user.id, user.email);
            user
        }
        Ok(None) => {
            tracing::warn!("form_login: NO USER for email={}", credentials.email);
            let _ = session.insert("flash_error", "Invalid credentials").await;
            return Ok(AppResponse::redirect("/login"));
        }
        Err(e) => {
            tracing::error!("Database error during login: {}", e);
            let _ = session.insert("flash_error", "An error occurred").await;
            return Ok(AppResponse::redirect("/login"));
        }
    };

    let user = db_user.to_security_user();

    if !user.can_authenticate() {
        tracing::warn!(
            "form_login: can_authenticate=false status={:?}",
            user.status
        );
        let _ = session.insert("flash_error", "Account is not active").await;
        return Ok(AppResponse::redirect("/login"));
    }

    if !user.verify_password(&credentials.password) {
        tracing::warn!(
            "form_login: password verify FAILED for email={}",
            user.email
        );
        let _ = session.insert("flash_error", "Invalid credentials").await;
        return Ok(AppResponse::redirect("/login"));
    }

    if user.two_factor_enabled {
        let _ = session.insert("2fa_user_id", user.id).await;
        tracing::info!("2FA required for user: {}", user.email);
        return Ok(AppResponse::redirect("/2fa"));
    }

    appkit_core::security::authenticator::store_user_in_session(&session, user.id).await?;

    tracing::info!("User logged in successfully: {}", user.email);

    Ok(AppResponse::redirect("/"))
}

pub async fn logout(session: Session) -> AppResult<Response> {
    session.flush().await.map_err(|e| {
        appkit_core::error::AppError::InternalServerError(format!(
            "Failed to invalidate session: {}",
            e
        ))
    })?;

    tracing::info!("User logged out, session invalidated");

    Ok(AppResponse::redirect("/login"))
}

pub async fn register(
    State(_state): State<App>,
    session: Session,
    request: Request<Body>,
) -> AppResult<Response> {
    let user_id: Option<i32> = session.get::<i32>(SESSION_USER_ID_KEY).await.ok().flatten();

    if user_id.is_some() {
        return Ok(AppResponse::redirect("/"));
    }

    let mut shared_props = SharedProps::new();

    if let Ok(Some(error)) = session.get::<String>("flash_error").await {
        shared_props = shared_props.with_field_error("email".to_string(), error);
        let _ = session.remove::<String>("flash_error").await;
    }

    let csrf_token = match session.get::<String>("csrf_token").await {
        Ok(Some(existing)) if !existing.is_empty() => existing,
        _ => {
            let token = generate_csrf_token();
            let _ = session.insert("csrf_token", token.clone()).await;
            token
        }
    };

    let props = shared_props.merge_with(json!({
        "csrf_token": csrf_token,
    }));

    let response = AppResponse::inertia("Auth/Register")
        .with_props(props)
        .render(&request);

    Ok(response)
}

fn generate_csrf_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
}

pub async fn validate_csrf_token(session: &Session, token: &str) -> bool {
    if let Ok(Some(stored_token)) = session.get::<String>("csrf_token").await {
        stored_token.len() == token.len()
            && stored_token
                .as_bytes()
                .iter()
                .zip(token.as_bytes())
                .all(|(a, b)| a == b)
    } else {
        false
    }
}
