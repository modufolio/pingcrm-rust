use crate::app::App;
use appkit_core::response::AppResponse;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn maintenance_middleware(
    State(state): State<App>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if state.config.maintenance_mode {
        if request.uri().path() == "/health" {
            return Ok(next.run(request).await);
        }

        if let Some(bypass_token) = &state.config.maintenance_bypass_token {
            if let Some(query) = request.uri().query() {
                let params: std::collections::HashMap<String, String> =
                    url::form_urlencoded::parse(query.as_bytes())
                        .into_owned()
                        .collect();

                if let Some(request_token) = params.get("bypass") {
                    if request_token == bypass_token {
                        tracing::info!(
                            "Maintenance mode bypassed for request to {}",
                            request.uri().path()
                        );
                        return Ok(next.run(request).await);
                    } else {
                        tracing::warn!("Invalid bypass token attempt for {}", request.uri().path());
                    }
                }
            }
        }

        tracing::info!(
            "Request blocked due to maintenance mode: {}",
            request.uri().path()
        );
        return Ok(AppResponse::unavailable(
            "Application is currently under maintenance. Please try again later.",
        ));
    }

    Ok(next.run(request).await)
}
