use axum::{extract::FromRequestParts, http::request::Parts};
use std::collections::HashMap;

pub trait FilterInterface: Sized {
    fn from_query_params(params: HashMap<String, String>) -> Self;
}

pub struct FilterExtractor<T>(pub T);

impl<T, S> FromRequestParts<S> for FilterExtractor<T>
where
    T: FilterInterface,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query_string = parts.uri.query().unwrap_or("");

        let params: HashMap<String, String> =
            serde_urlencoded::from_str(query_string).unwrap_or_default();

        let filter = T::from_query_params(params);

        Ok(FilterExtractor(filter))
    }
}

impl FilterInterface for crate::jsonapi::QueryParams {
    fn from_query_params(params: HashMap<String, String>) -> Self {
        Self::from_query_map(&params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{extract::FromRequestParts, http::Request};

    #[derive(Debug, Default, PartialEq)]
    struct TestFilter {
        name: Option<String>,
        age: Option<u32>,
        active: bool,
    }

    impl FilterInterface for TestFilter {
        fn from_query_params(params: HashMap<String, String>) -> Self {
            Self {
                name: params.get("name").cloned(),
                age: params.get("age").and_then(|v| v.parse().ok()),
                active: params
                    .get("active")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(false),
            }
        }
    }

    #[tokio::test]
    async fn test_filter_extractor_with_params() {
        let uri = "http://example.com/test?name=John&age=30&active=true"
            .parse::<axum::http::Uri>()
            .unwrap();

        let mut parts = Request::builder().uri(uri).body(()).unwrap().into_parts().0;

        let FilterExtractor(filter) =
            FilterExtractor::<TestFilter>::from_request_parts(&mut parts, &())
                .await
                .unwrap();

        assert_eq!(filter.name, Some("John".to_string()));
        assert_eq!(filter.age, Some(30));
        assert!(filter.active);
    }

    #[tokio::test]
    async fn test_filter_extractor_without_params() {
        let uri = "http://example.com/test"
            .parse::<axum::http::Uri>()
            .unwrap();

        let mut parts = Request::builder().uri(uri).body(()).unwrap().into_parts().0;

        let FilterExtractor(filter) =
            FilterExtractor::<TestFilter>::from_request_parts(&mut parts, &())
                .await
                .unwrap();

        assert_eq!(filter.name, None);
        assert_eq!(filter.age, None);
        assert!(!filter.active);
    }

    #[tokio::test]
    async fn test_filter_extractor_with_json_api_query_params() {
        use crate::jsonapi::QueryParams;

        let uri = "http://example.com/test?filter[name]=John&filter[age][gt]=18&sort=-created_at&page[number]=2"
            .parse::<axum::http::Uri>()
            .unwrap();

        let mut parts = Request::builder().uri(uri).body(()).unwrap().into_parts().0;

        let FilterExtractor(query) =
            FilterExtractor::<QueryParams>::from_request_parts(&mut parts, &())
                .await
                .unwrap();

        assert_eq!(query.filters.len(), 2);
        assert_eq!(query.sort.len(), 1);
        assert_eq!(query.page.number, 2);
    }
}
