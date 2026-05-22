use super::rule::AccessControlConfig;
use crate::error::AppError;
use crate::security::user::User;
use axum::{extract::Request, middleware::Next, response::Response};
use std::sync::Arc;

#[tracing::instrument(
    name = "access_control",
    skip(config, request, next),
    fields(
        path = %request.uri().path(),
        has_rule = tracing::field::Empty,
        user_id = tracing::field::Empty,
        authorized = tracing::field::Empty
    )
)]
pub async fn access_control_middleware(
    config: Arc<AccessControlConfig>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path();

    let rule = match config.find_rule(path) {
        Some(rule) => {
            tracing::Span::current().record("has_rule", true);
            rule
        }
        None => {
            tracing::Span::current().record("has_rule", false);
            tracing::Span::current().record("authorized", true);
            tracing::debug!("Access control: No rule for path {}", path);
            return Ok(next.run(request).await);
        }
    };

    if rule.roles.is_empty() {
        tracing::Span::current().record("authorized", true);
        tracing::debug!("Access control: No roles required for {}", path);
        return Ok(next.run(request).await);
    }

    let user = request.extensions().get::<User>();

    let user = match user {
        Some(user) => {
            tracing::Span::current().record("user_id", user.id);
            user
        }
        None => {
            tracing::Span::current().record("authorized", false);
            tracing::warn!(
                "Access control: Authentication required for {} (requires roles: {:?})",
                path,
                rule.roles
            );
            return Err(AppError::AuthenticationFailed(
                "Authentication required".to_string(),
            ));
        }
    };

    let user_roles = user.role.inherited_roles();

    if rule.check_roles(&user_roles) {
        tracing::Span::current().record("authorized", true);
        tracing::debug!(
            "Access control: User id={} authorized for {} (role: {:?})",
            user.id,
            path,
            user.role
        );
        Ok(next.run(request).await)
    } else {
        tracing::Span::current().record("authorized", false);
        tracing::warn!(
            "Access control: User id={} forbidden from {} (has role: {:?}, requires: {:?})",
            user.id,
            path,
            user.role,
            rule.roles
        );
        Err(AppError::AuthorizationFailed(format!(
            "Insufficient permissions. Required roles: {:?}",
            rule.roles
        )))
    }
}
