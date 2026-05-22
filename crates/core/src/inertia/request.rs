use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};

#[derive(Debug, Clone, Default)]
pub struct InertiaRequest {
    pub is_inertia: bool,

    pub version: Option<String>,

    pub partial_data: Option<Vec<String>>,

    pub partial_component: Option<String>,
}

impl InertiaRequest {
    pub fn is_inertia(&self) -> bool {
        self.is_inertia
    }

    pub fn is_partial(&self) -> bool {
        self.partial_data.is_some()
    }

    pub fn should_include_prop(&self, key: &str) -> bool {
        match &self.partial_data {
            Some(keys) => keys.iter().any(|k| k == key),
            None => true,
        }
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn from_headers(headers: &HeaderMap) -> Self {
        let is_inertia = headers
            .get("X-Inertia")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "true")
            .unwrap_or(false);

        let version = headers
            .get("X-Inertia-Version")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let partial_data = headers
            .get("X-Inertia-Partial-Data")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').map(|p| p.trim().to_string()).collect());

        let partial_component = headers
            .get("X-Inertia-Partial-Component")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Self {
            is_inertia,
            version,
            partial_data,
            partial_component,
        }
    }
}

impl<S> FromRequestParts<S> for InertiaRequest
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_headers(&parts.headers))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
    JsonApi,

    Inertia,

    Web,
}

impl RequestType {
    pub fn detect(headers: &HeaderMap, path: &str) -> Self {
        if headers
            .get("X-Inertia")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "true")
            .unwrap_or(false)
        {
            return Self::Inertia;
        }

        let accept = headers
            .get("Accept")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if path.starts_with("/api/") || accept.contains("application/vnd.api+json") {
            return Self::JsonApi;
        }

        Self::Web
    }

    pub fn is_api(&self) -> bool {
        matches!(self, Self::JsonApi)
    }

    pub fn is_inertia(&self) -> bool {
        matches!(self, Self::Inertia)
    }

    pub fn is_web(&self) -> bool {
        matches!(self, Self::Web)
    }
}

impl<S> FromRequestParts<S> for RequestType
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::detect(&parts.headers, parts.uri.path()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_detect_inertia_request() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Inertia", HeaderValue::from_static("true"));

        let req = InertiaRequest::from_headers(&headers);
        assert!(req.is_inertia());
    }

    #[test]
    fn test_detect_partial_request() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Inertia", HeaderValue::from_static("true"));
        headers.insert(
            "X-Inertia-Partial-Data",
            HeaderValue::from_static("users,posts"),
        );

        let req = InertiaRequest::from_headers(&headers);
        assert!(req.is_partial());
        assert!(req.should_include_prop("users"));
        assert!(req.should_include_prop("posts"));
        assert!(!req.should_include_prop("comments"));
    }

    #[test]
    fn test_request_type_detection() {
        let mut headers = HeaderMap::new();

        headers.insert("X-Inertia", HeaderValue::from_static("true"));
        assert_eq!(
            RequestType::detect(&headers, "/users"),
            RequestType::Inertia
        );

        headers.clear();
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.api+json"),
        );
        assert_eq!(
            RequestType::detect(&headers, "/users"),
            RequestType::JsonApi
        );

        headers.clear();
        assert_eq!(
            RequestType::detect(&headers, "/api/users"),
            RequestType::JsonApi
        );

        headers.clear();
        assert_eq!(RequestType::detect(&headers, "/users"), RequestType::Web);
    }
}
