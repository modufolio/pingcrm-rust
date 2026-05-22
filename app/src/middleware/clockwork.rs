use axum::{
    body::{Body, Bytes},
    extract::{Request, State},
    http::{HeaderValue, Response},
    middleware::Next,
};
use http_body_util::BodyExt;
use serde_json::json;
use std::collections::HashMap;

use crate::app::App;
use crate::clockwork::{start_request_tracking, ClockworkController};

pub async fn clockwork_middleware(
    State(state): State<App>,
    request: Request,
    next: Next,
) -> Response<Body> {
    let uri = request.uri().path().to_string();
    if uri.starts_with("/__clockwork") {
        return next.run(request).await;
    }

    if !state.config.debug {
        return next.run(request).await;
    }

    let debug_stack = state.debug_stack.clone();

    let method = request.method().to_string();
    let url = request.uri().to_string();

    let headers: HashMap<String, String> = request
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let query_params: HashMap<String, String> = request
        .uri()
        .query()
        .map(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_default();

    let (parts, body) = request.into_parts();
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => Bytes::new(),
    };

    let post_data = if !bytes.is_empty() {
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or(json!({}))
    } else {
        json!({})
    };

    let request = Request::from_parts(parts, Body::from(bytes));

    start_request_tracking(
        &debug_stack,
        &method,
        &url,
        &uri,
        json!(headers),
        json!(query_params),
        post_data,
    );

    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    let response_status = response.status().as_u16() as i32;

    let controller = ClockworkController::new(state.db_pool.clone(), debug_stack);

    if let Some(request_id) = controller.store_request(response_status, duration_ms).await {
        let (mut parts, body) = response.into_parts();

        if let Ok(id_value) = HeaderValue::from_str(&request_id) {
            parts.headers.insert("X-Clockwork-Id", id_value);
        }
        parts
            .headers
            .insert("X-Clockwork-Version", HeaderValue::from_static("5.1"));
        parts.headers.insert(
            "X-Clockwork-Path",
            HeaderValue::from_static("/__clockwork/"),
        );

        Response::from_parts(parts, body)
    } else {
        response
    }
}
