use axum::{
    extract::{FromRequestParts, Query},
    http::{request::Parts, StatusCode},
};
use serde::Deserialize;

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub page: i64,

    pub per_page: i64,

    pub offset: i64,
}

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    #[serde(default = "default_page")]
    page: Option<i64>,
    #[serde(default = "default_per_page")]
    per_page: Option<i64>,
}

fn default_page() -> Option<i64> {
    Some(1)
}

fn default_per_page() -> Option<i64> {
    Some(25)
}

impl Pagination {
    pub fn new() -> Self {
        Self {
            page: 1,
            per_page: 25,
            offset: 0,
        }
    }

    pub fn with_params(page: i64, per_page: i64) -> Result<Self, String> {
        if page < 1 {
            return Err("Page must be >= 1".to_string());
        }
        if per_page < 1 || per_page > 100 {
            return Err("Per page must be between 1 and 100".to_string());
        }

        let offset = (page - 1) * per_page;
        Ok(Self {
            page,
            per_page,
            offset,
        })
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> FromRequestParts<S> for Pagination
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(query) = Query::<PaginationQuery>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Invalid query parameters: {}", e),
                )
            })?;

        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(25);

        if page < 1 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Page number must be >= 1".to_string(),
            ));
        }

        if per_page < 1 {
            return Err((StatusCode::BAD_REQUEST, "Per page must be >= 1".to_string()));
        }

        if per_page > 100 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Per page must be <= 100".to_string(),
            ));
        }

        let offset = (page - 1) * per_page;

        Ok(Pagination {
            page,
            per_page,
            offset,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, Uri};

    fn make_parts() -> Parts {
        let req = Request::builder().body(()).unwrap();
        let (parts, _) = req.into_parts();
        parts
    }

    #[tokio::test]
    async fn test_default_pagination() {
        let mut parts = make_parts();
        parts.uri = Uri::from_static("http://localhost/users");

        let pagination = Pagination::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 25);
        assert_eq!(pagination.offset, 0);
    }

    #[tokio::test]
    async fn test_custom_pagination() {
        let mut parts = make_parts();
        parts.uri = Uri::from_static("http://localhost/users?page=3&per_page=50");

        let pagination = Pagination::from_request_parts(&mut parts, &())
            .await
            .unwrap();
        assert_eq!(pagination.page, 3);
        assert_eq!(pagination.per_page, 50);
        assert_eq!(pagination.offset, 100);
    }

    #[tokio::test]
    async fn test_invalid_page() {
        let mut parts = make_parts();
        parts.uri = Uri::from_static("http://localhost/users?page=0");

        let result = Pagination::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_per_page_too_large() {
        let mut parts = make_parts();
        parts.uri = Uri::from_static("http://localhost/users?per_page=150");

        let result = Pagination::from_request_parts(&mut parts, &()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_offset_calculation() {
        let pagination = Pagination::with_params(5, 20).unwrap();
        assert_eq!(pagination.offset, 80);

        let pagination = Pagination::with_params(1, 10).unwrap();
        assert_eq!(pagination.offset, 0);
    }
}
