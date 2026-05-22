use crate::extractors::FilterInterface;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilter {
    pub search: Option<String>,

    pub role: Option<String>,

    pub trashed: Option<String>,
}

impl SearchFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_filters(&self) -> bool {
        self.search.is_some() || self.role.is_some() || self.trashed.is_some()
    }

    fn normalize_role(role: Option<String>) -> Option<String> {
        match role.as_deref() {
            Some("admin") | Some("editor") | Some("user") => role,
            _ => None,
        }
    }

    fn normalize_trashed(trashed: Option<String>) -> Option<String> {
        match trashed.as_deref() {
            Some("only") | Some("with") => trashed,
            _ => None,
        }
    }

    fn normalize_search(search: Option<String>) -> Option<String> {
        search.and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    }
}

impl FilterInterface for SearchFilter {
    fn from_query_params(params: HashMap<String, String>) -> Self {
        Self {
            search: Self::normalize_search(params.get("search").cloned()),
            role: Self::normalize_role(params.get("role").cloned()),
            trashed: Self::normalize_trashed(params.get("trashed").cloned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_filter() {
        let params = HashMap::new();
        let filter = SearchFilter::from_query_params(params);

        assert_eq!(filter.search, None);
        assert_eq!(filter.role, None);
        assert_eq!(filter.trashed, None);
        assert!(!filter.has_filters());
    }

    #[test]
    fn test_search_filter() {
        let mut params = HashMap::new();
        params.insert("search".to_string(), "john doe  ".to_string());

        let filter = SearchFilter::from_query_params(params);

        assert_eq!(filter.search, Some("john doe".to_string()));
        assert!(filter.has_filters());
    }

    #[test]
    fn test_empty_search_is_none() {
        let mut params = HashMap::new();
        params.insert("search".to_string(), " ".to_string());

        let filter = SearchFilter::from_query_params(params);

        assert_eq!(filter.search, None);
    }

    #[test]
    fn test_valid_roles() {
        for role in ["admin", "editor", "user"] {
            let mut params = HashMap::new();
            params.insert("role".to_string(), role.to_string());

            let filter = SearchFilter::from_query_params(params);

            assert_eq!(filter.role, Some(role.to_string()));
            assert!(filter.has_filters());
        }
    }

    #[test]
    fn test_invalid_role() {
        let mut params = HashMap::new();
        params.insert("role".to_string(), "invalid".to_string());

        let filter = SearchFilter::from_query_params(params);

        assert_eq!(filter.role, None);
    }

    #[test]
    fn test_valid_trashed() {
        for trashed in ["only", "with"] {
            let mut params = HashMap::new();
            params.insert("trashed".to_string(), trashed.to_string());

            let filter = SearchFilter::from_query_params(params);

            assert_eq!(filter.trashed, Some(trashed.to_string()));
            assert!(filter.has_filters());
        }
    }

    #[test]
    fn test_invalid_trashed() {
        let mut params = HashMap::new();
        params.insert("trashed".to_string(), "invalid".to_string());

        let filter = SearchFilter::from_query_params(params);

        assert_eq!(filter.trashed, None);
    }

    #[test]
    fn test_combined_filters() {
        let mut params = HashMap::new();
        params.insert("search".to_string(), "john".to_string());
        params.insert("role".to_string(), "admin".to_string());
        params.insert("trashed".to_string(), "with".to_string());

        let filter = SearchFilter::from_query_params(params);

        assert_eq!(filter.search, Some("john".to_string()));
        assert_eq!(filter.role, Some("admin".to_string()));
        assert_eq!(filter.trashed, Some("with".to_string()));
        assert!(filter.has_filters());
    }
}
