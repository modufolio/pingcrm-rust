use appkit_core::error::AppError;
use appkit_core::response::AppResponse;
use axum::extract::Request;
use axum::response::Response;
use tower_sessions::Session;

pub fn is_logout_request(path: &str, method: &axum::http::Method) -> bool {
    use axum::http::Method;

    (method == Method::POST && path == "/logout") || (method == Method::GET && path == "/logout")
}

pub fn is_entry_point_page(path: &str) -> bool {
    matches!(
        path,
        "/login" | "/2fa" | "/register" | "/forgot-password" | "/reset-password"
    )
}

pub fn try_restore_session_token(
    _request: &Request,
    _firewall_name: &str,
) -> Option<appkit_core::security::token::Token> {
    None
}

pub async fn handle_logout(request: Request, _firewall_name: &str) -> Result<Response, AppError> {
    if let Some(session) = request.extensions().get::<Session>() {
        appkit_core::security::authenticator::invalidate_session(session).await?;
        tracing::info!("Session invalidated during logout");
    }

    Ok(AppResponse::redirect("/login"))
}
