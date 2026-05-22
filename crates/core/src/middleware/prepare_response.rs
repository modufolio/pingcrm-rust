use axum::body::{Body, HttpBody};
use axum::extract::Request;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use http_body_util::BodyExt;

pub async fn prepare_response_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let is_head = method == Method::HEAD;
    let has_inertia = request.headers().get("X-Inertia").is_some();
    let is_inertia_redirect_method = matches!(method, Method::PUT | Method::PATCH | Method::DELETE);

    let mut response = next.run(request).await;

    if response.headers().contains_key("Transfer-Encoding") {
        response.headers_mut().remove("Content-Length");
    }

    if !response.headers().contains_key("Content-Length")
        && !response.headers().contains_key("Transfer-Encoding")
    {
        if let Some(size) = response.body().size_hint().exact() {
            if let Ok(size_str) = HeaderValue::from_str(&size.to_string()) {
                response.headers_mut().insert("Content-Length", size_str);
            }
        }
    }

    if is_head {
        let content_length = response.headers().get("Content-Length").cloned();

        let (parts, _body) = response.into_parts();
        response = Response::from_parts(parts, Body::empty());

        if let Some(length) = content_length {
            response.headers_mut().insert("Content-Length", length);
        }
    }

    if has_inertia {
        if response.status() == StatusCode::FOUND && is_inertia_redirect_method {
            *response.status_mut() = StatusCode::SEE_OTHER;
        }

        response
            .headers_mut()
            .insert("X-Inertia", HeaderValue::from_static("true"));

        response
            .headers_mut()
            .insert("Vary", HeaderValue::from_static("Accept"));
    }

    if !response.headers().contains_key("X-Request-Id") {
        if let Ok(request_id) = HeaderValue::from_str(&uuid::Uuid::new_v4().to_string()) {
            response.headers_mut().insert("X-Request-Id", request_id);
        }
    }

    response
}

pub async fn prepare_response_buffered_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let is_head = method == Method::HEAD;
    let has_inertia = request.headers().get("X-Inertia").is_some();
    let is_inertia_redirect_method = matches!(method, Method::PUT | Method::PATCH | Method::DELETE);

    let mut response = next.run(request).await;

    if response.headers().contains_key("Transfer-Encoding") {
        response.headers_mut().remove("Content-Length");
    }

    if !response.headers().contains_key("Content-Length")
        && !response.headers().contains_key("Transfer-Encoding")
    {
        let (parts, body) = response.into_parts();

        let bytes = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Failed to process response"))
                    .unwrap();
            }
        };

        let content_length = bytes.len();

        response = Response::from_parts(parts, Body::from(bytes));

        if let Ok(length_str) = HeaderValue::from_str(&content_length.to_string()) {
            response.headers_mut().insert("Content-Length", length_str);
        }
    }

    if is_head {
        let content_length = response.headers().get("Content-Length").cloned();
        let (parts, _body) = response.into_parts();
        response = Response::from_parts(parts, Body::empty());

        if let Some(length) = content_length {
            response.headers_mut().insert("Content-Length", length);
        }
    }

    if has_inertia {
        if response.status() == StatusCode::FOUND && is_inertia_redirect_method {
            *response.status_mut() = StatusCode::SEE_OTHER;
        }

        response
            .headers_mut()
            .insert("X-Inertia", HeaderValue::from_static("true"));

        response
            .headers_mut()
            .insert("Vary", HeaderValue::from_static("Accept"));
    }

    if !response.headers().contains_key("X-Request-Id") {
        if let Ok(request_id) = HeaderValue::from_str(&uuid::Uuid::new_v4().to_string()) {
            response.headers_mut().insert("X-Request-Id", request_id);
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use axum::{middleware, routing::get, Router};
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "Hello, World!"
    }

    #[tokio::test]
    async fn test_head_request_strips_body() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(prepare_response_middleware));

        let request = Request::builder()
            .method(Method::HEAD)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert!(response.headers().contains_key("Content-Length"));

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body_bytes.len(), 0);
    }

    #[tokio::test]
    async fn test_inertia_redirect_conversion() {
        async fn redirect_handler() -> Response {
            Response::builder()
                .status(StatusCode::FOUND)
                .body(Body::empty())
                .unwrap()
        }

        let app = Router::new()
            .route("/test", axum::routing::delete(redirect_handler))
            .layer(middleware::from_fn(prepare_response_middleware));

        let request = Request::builder()
            .method(Method::DELETE)
            .uri("/test")
            .header("X-Inertia", "true")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        assert_eq!(response.headers().get("X-Inertia").unwrap(), "true");
        assert_eq!(response.headers().get("Vary").unwrap(), "Accept");
    }

    #[tokio::test]
    async fn test_request_id_added() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(prepare_response_middleware));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert!(response.headers().contains_key("X-Request-Id"));
    }
}
