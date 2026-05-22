use crate::negotiation::{AcceptHeader, Negotiator};
use axum::{extract::FromRequestParts, http::request::Parts};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResponseFormat {
    #[default]
    JsonApi,

    Json,

    Text,
}

impl ResponseFormat {
    pub fn from_accept_header(accept: &str) -> Self {
        let priorities = &["application/vnd.api+json", "application/json", "text/plain"];

        let negotiator = Negotiator::new();

        if let Some(best_match) = negotiator.negotiate(accept, priorities) {
            let media_type = best_match.get_type();

            if media_type.eq_ignore_ascii_case("application/vnd.api+json") {
                Self::JsonApi
            } else if media_type.eq_ignore_ascii_case("application/json") {
                Self::Json
            } else if media_type.eq_ignore_ascii_case("text/plain") {
                Self::Text
            } else {
                Self::JsonApi
            }
        } else {
            Self::JsonApi
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::JsonApi => "application/vnd.api+json",
            Self::Json => "application/json",
            Self::Text => "text/plain; charset=utf-8",
        }
    }
}

impl<S> FromRequestParts<S> for ResponseFormat
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let format = parts
            .headers
            .get(axum::http::header::ACCEPT)
            .and_then(|v| v.to_str().ok())
            .map(Self::from_accept_header)
            .unwrap_or_default();

        Ok(format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonapi_format() {
        let format = ResponseFormat::from_accept_header("application/vnd.api+json");
        assert_eq!(format, ResponseFormat::JsonApi);
    }

    #[test]
    fn test_json_format() {
        let format = ResponseFormat::from_accept_header("application/json");
        assert_eq!(format, ResponseFormat::Json);
    }

    #[test]
    fn test_text_format() {
        let format = ResponseFormat::from_accept_header("text/plain");
        assert_eq!(format, ResponseFormat::Text);
    }

    #[test]
    fn test_wildcard_defaults_to_jsonapi() {
        let format = ResponseFormat::from_accept_header("*/*");
        assert_eq!(format, ResponseFormat::JsonApi);
    }

    #[test]
    fn test_complex_accept_header() {
        let format = ResponseFormat::from_accept_header(
            "text/html,application/xhtml+xml,application/xml;q=0.9,application/json;q=0.8,*/*;q=0.7"
        );

        assert_eq!(format, ResponseFormat::Json);
    }

    #[test]
    fn test_quality_negotiation() {
        let format = ResponseFormat::from_accept_header(
            "application/json;q=0.9, application/vnd.api+json;q=1.0",
        );
        assert_eq!(format, ResponseFormat::JsonApi);
    }

    #[test]
    fn test_wildcard_with_specific() {
        let format = ResponseFormat::from_accept_header("application/json, */*;q=0.1");
        assert_eq!(format, ResponseFormat::Json);
    }

    #[test]
    fn test_no_match_defaults_to_jsonapi() {
        let format = ResponseFormat::from_accept_header("image/png, video/mp4");
        assert_eq!(format, ResponseFormat::JsonApi);
    }
}
