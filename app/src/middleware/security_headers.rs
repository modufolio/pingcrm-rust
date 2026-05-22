use axum::{extract::Request, middleware::Next, response::Response};

pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());

    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());

    headers.insert("X-Frame-Options", "DENY".parse().unwrap());

    headers.insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    headers.insert(
        "Permissions-Policy",
        "geolocation=(), microphone=(), camera=()".parse().unwrap(),
    );

    response
}
