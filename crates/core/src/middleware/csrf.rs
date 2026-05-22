use crate::error::{AppError, AppResult};
use crate::security::csrf::CsrfTokenManager;
use axum::{
    extract::Request,
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower_sessions::Session;

const CSRF_HEADER: &str = "X-CSRF-Token";

#[allow(dead_code)]
const CSRF_FORM_FIELD: &str = "_csrf_token";

#[derive(Debug, Clone)]
pub struct CsrfConfig {
    pub protected_methods: Vec<Method>,

    pub exempt_paths: Vec<String>,

    pub allow_header: bool,

    pub allow_form: bool,
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            protected_methods: vec![Method::POST, Method::PUT, Method::PATCH, Method::DELETE],
            exempt_paths: Vec::new(),
            allow_header: true,
            allow_form: true,
        }
    }
}

impl CsrfConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exempt_path(mut self, path: impl Into<String>) -> Self {
        self.exempt_paths.push(path.into());
        self
    }

    pub fn allow_header(mut self, allow: bool) -> Self {
        self.allow_header = allow;
        self
    }

    pub fn allow_form(mut self, allow: bool) -> Self {
        self.allow_form = allow;
        self
    }

    pub fn protected_methods(mut self, methods: Vec<Method>) -> Self {
        self.protected_methods = methods;
        self
    }

    fn is_exempt(&self, path: &str) -> bool {
        self.exempt_paths.iter().any(|p| path.starts_with(p))
    }

    fn is_protected_method(&self, method: &Method) -> bool {
        self.protected_methods.contains(method)
    }
}

pub async fn csrf_middleware(
    req: Request,
    next: Next,
    config: CsrfConfig,
) -> Result<Response, AppError> {
    let method = req.method();
    let path = req.uri().path();

    if matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS) {
        return Ok(next.run(req).await);
    }

    if config.is_exempt(path) {
        return Ok(next.run(req).await);
    }

    if !config.is_protected_method(method) {
        return Ok(next.run(req).await);
    }

    let session = req
        .extensions()
        .get::<Session>()
        .ok_or_else(|| AppError::InternalError("Session not found in request".to_string()))?
        .clone();

    let token = extract_csrf_token(&req, &config).await?;

    CsrfTokenManager::validate_token(&session, &token).await?;

    Ok(next.run(req).await)
}

async fn extract_csrf_token(req: &Request, config: &CsrfConfig) -> AppResult<String> {
    if config.allow_header {
        if let Some(token) = extract_from_header(req.headers()) {
            return Ok(token);
        }
    }

    if config.allow_form {}

    Err(AppError::BadRequest("CSRF token not provided".to_string()))
}

fn extract_from_header(headers: &HeaderMap) -> Option<String> {
    headers
        .get(CSRF_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

pub async fn simple_csrf_middleware(req: Request, next: Next) -> Result<Response, AppError> {
    csrf_middleware(req, next, CsrfConfig::default()).await
}

pub fn csrf_error_response(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        format!("CSRF validation failed: {}", message),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_config_default() {
        let config = CsrfConfig::default();
        assert!(config.allow_header);
        assert!(config.allow_form);
        assert_eq!(config.protected_methods.len(), 4);
    }

    #[test]
    fn test_csrf_config_exempt_path() {
        let config = CsrfConfig::new()
            .exempt_path("/api/webhook")
            .exempt_path("/api/public/");

        assert!(config.is_exempt("/api/webhook"));
        assert!(config.is_exempt("/api/public/test"));
        assert!(!config.is_exempt("/admin"));
    }

    #[test]
    fn test_csrf_config_protected_methods() {
        let config = CsrfConfig::default();
        assert!(config.is_protected_method(&Method::POST));
        assert!(config.is_protected_method(&Method::PUT));
        assert!(config.is_protected_method(&Method::PATCH));
        assert!(config.is_protected_method(&Method::DELETE));
        assert!(!config.is_protected_method(&Method::GET));
        assert!(!config.is_protected_method(&Method::HEAD));
    }

    #[test]
    fn test_extract_from_header() {
        use axum::http::HeaderValue;

        let mut headers = HeaderMap::new();
        headers.insert(CSRF_HEADER, HeaderValue::from_static("test-token"));

        let token = extract_from_header(&headers);
        assert_eq!(token, Some("test-token".to_string()));
    }

    #[test]
    fn test_extract_from_header_missing() {
        let headers = HeaderMap::new();
        let token = extract_from_header(&headers);
        assert_eq!(token, None);
    }
}
