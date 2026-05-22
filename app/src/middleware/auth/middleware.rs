use super::helpers::{
    handle_logout, is_entry_point_page, is_logout_request, try_restore_session_token,
};
use crate::app::App;
use appkit_core::error::AppError;
use appkit_core::response::AppResponse;
use appkit_core::security::authenticator::AuthenticatorChain;
use appkit_core::security::firewall::FirewallRule;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tower_sessions::Session;

pub async fn auth_middleware(
    State(_state): State<App>,
    authenticator_chain: Arc<AuthenticatorChain<crate::database::DieselUserRepository>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path().to_string();
    let method = request.method().clone();

    let firewall_rule = request.extensions().get::<FirewallRule>().cloned();

    let firewall_rule = match firewall_rule {
        Some(rule) => rule,
        None => {
            tracing::error!(
                "No firewall rule found in request extensions for path: {}",
                path
            );
            return Ok(next.run(request).await);
        }
    };

    if !firewall_rule.requires_auth() {
        tracing::debug!("Public route - no authentication required: {}", path);
        let response = next.run(request).await;
        return Ok(response);
    }

    let firewall_name = firewall_rule.name();
    let stateless = firewall_rule.is_stateless();

    if is_logout_request(&path, &method) {
        tracing::info!("Logout request detected for path: {}", path);
        return handle_logout(request, firewall_name).await;
    }

    if let Some(token) = try_restore_session_token(&request, firewall_name) {
        tracing::debug!("Session token restored for firewall: {}", firewall_name);

        let user = token.get_user().clone();

        let mut request = request;
        request.extensions_mut().insert(user);
        return Ok(next.run(request).await);
    }

    if is_entry_point_page(&path) {
        tracing::debug!("Entry point page accessed: {}", path);
        return Ok(next.run(request).await);
    }

    match authenticator_chain
        .try_authenticate(request, firewall_name.to_string())
        .await?
    {
        Some((_token, Some(response), _request)) => Ok(response),
        Some((token, None, mut request)) => {
            let user = token.get_user().clone();

            if !stateless {
                if let Some(session) = request.extensions().get::<Session>() {
                    if let Err(e) = appkit_core::security::authenticator::store_token_in_session(
                        session, &token,
                    )
                    .await
                    {
                        tracing::warn!("Failed to store token in session: {}", e);
                    }
                }
            }

            request.extensions_mut().insert(user);

            Ok(next.run(request).await)
        }
        None => {
            tracing::debug!("No credentials found for protected path {}", path);

            if stateless {
                Err(AppError::AuthenticationFailed(
                    "Authentication required".to_string(),
                ))
            } else {
                Ok(AppResponse::redirect("/login"))
            }
        }
    }
}
