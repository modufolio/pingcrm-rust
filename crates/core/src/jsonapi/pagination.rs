use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,

    pub per_page: i64,

    pub current_page: i64,

    pub last_page: i64,

    pub from: i64,

    pub to: i64,
}

impl PaginationMeta {
    pub fn new(total: i64, current_page: i64, per_page: i64) -> Self {
        let last_page = if total == 0 || per_page == 0 {
            1
        } else {
            ((total as f64) / (per_page as f64)).ceil() as i64
        };

        let from = if total > 0 {
            ((current_page - 1) * per_page) + 1
        } else {
            0
        };

        let to = std::cmp::min(current_page * per_page, total);

        Self {
            total,
            per_page,
            current_page,
            last_page,
            from,
            to,
        }
    }

    pub fn to_value(&self) -> Value {
        json!(self)
    }

    pub fn to_map(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("total".to_string(), json!(self.total));
        map.insert("per_page".to_string(), json!(self.per_page));
        map.insert("current_page".to_string(), json!(self.current_page));
        map.insert("last_page".to_string(), json!(self.last_page));
        map.insert("from".to_string(), json!(self.from));
        map.insert("to".to_string(), json!(self.to));
        map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationLinks {
    pub first: String,
    pub last: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    #[serde(rename = "self")]
    pub self_link: String,
}

impl PaginationLinks {
    pub fn new(base_url: &str, current_page: i64, last_page: i64, per_page: i64) -> Self {
        let first = Self::build_page_url(base_url, 1, per_page);
        let last = Self::build_page_url(base_url, last_page, per_page);
        let self_link = Self::build_page_url(base_url, current_page, per_page);

        let prev = if current_page > 1 {
            Some(Self::build_page_url(base_url, current_page - 1, per_page))
        } else {
            None
        };

        let next = if current_page < last_page {
            Some(Self::build_page_url(base_url, current_page + 1, per_page))
        } else {
            None
        };

        Self {
            first,
            last,
            prev,
            next,
            self_link,
        }
    }

    fn build_page_url(base_url: &str, page: i64, per_page: i64) -> String {
        let parts: Vec<&str> = base_url.splitn(2, '?').collect();
        let path = parts[0];
        let existing_query = parts.get(1).unwrap_or(&"");

        let mut params: Vec<(String, String)> = if !existing_query.is_empty() {
            existing_query
                .split('&')
                .filter_map(|param| {
                    let kv: Vec<&str> = param.splitn(2, '=').collect();
                    if kv.len() == 2 {
                        let key = kv[0].to_string();
                        let value = kv[1].to_string();

                        if key != "page[number]"
                            && key != "page[size]"
                            && key != "page"
                            && key != "per_page"
                        {
                            Some((key, value))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        params.push(("page[number]".to_string(), page.to_string()));
        params.push(("page[size]".to_string(), per_page.to_string()));

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", path, query_string)
    }

    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("first".to_string(), self.first.clone());
        map.insert("last".to_string(), self.last.clone());
        map.insert("self".to_string(), self.self_link.clone());

        if let Some(ref prev) = self.prev {
            map.insert("prev".to_string(), prev.clone());
        }

        if let Some(ref next) = self.next {
            map.insert("next".to_string(), next.clone());
        }

        map
    }
}

pub struct Pagination {
    pub meta: PaginationMeta,
    pub links: PaginationLinks,
}

impl Pagination {
    pub fn new(total: i64, current_page: i64, per_page: i64, base_url: &str) -> Self {
        let meta = PaginationMeta::new(total, current_page, per_page);
        let links = PaginationLinks::new(base_url, current_page, meta.last_page, per_page);

        Self { meta, links }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,

    pub meta: PaginationMeta,

    pub links: PaginationLinks,
}

impl<T> PaginatedResponse<T> {
    pub fn new(
        items: Vec<T>,
        total: i64,
        current_page: i64,
        per_page: i64,
        base_url: &str,
    ) -> Self {
        let meta = PaginationMeta::new(total, current_page, per_page);
        let links = PaginationLinks::new(base_url, current_page, meta.last_page, per_page);

        Self {
            data: items,
            meta,
            links,
        }
    }

    pub fn from_query_params(
        items: Vec<T>,
        total: i64,
        params: &crate::jsonapi::query::QueryParams,
        base_url: &str,
    ) -> Self {
        Self::new(items, total, params.page.number, params.page.size, base_url)
    }

    pub fn map<U, F>(self, f: F) -> PaginatedResponse<U>
    where
        F: Fn(T) -> U,
    {
        PaginatedResponse {
            data: self.data.into_iter().map(f).collect(),
            meta: self.meta,
            links: self.links,
        }
    }

    pub fn try_map<U, E, F>(self, f: F) -> Result<PaginatedResponse<U>, E>
    where
        F: Fn(T) -> Result<U, E>,
    {
        let data: Result<Vec<U>, E> = self.data.into_iter().map(f).collect();

        Ok(PaginatedResponse {
            data: data?,
            meta: self.meta,
            links: self.links,
        })
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn is_first_page(&self) -> bool {
        self.meta.current_page == 1
    }

    pub fn is_last_page(&self) -> bool {
        self.meta.current_page == self.meta.last_page
    }

    pub fn total_pages(&self) -> i64 {
        self.meta.last_page
    }

    pub fn to_value(&self) -> Value
    where
        T: Serialize,
    {
        json!({
            "data": self.data,
            "meta": self.meta,
            "links": self.links,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(100, 3, 25);

        assert_eq!(meta.total, 100);
        assert_eq!(meta.per_page, 25);
        assert_eq!(meta.current_page, 3);
        assert_eq!(meta.last_page, 4);
        assert_eq!(meta.from, 51);
        assert_eq!(meta.to, 75);
    }

    #[test]
    fn test_pagination_meta_empty() {
        let meta = PaginationMeta::new(0, 1, 25);

        assert_eq!(meta.total, 0);
        assert_eq!(meta.last_page, 1);
        assert_eq!(meta.from, 0);
        assert_eq!(meta.to, 0);
    }

    #[test]
    fn test_pagination_links() {
        let links = PaginationLinks::new("/api/users", 2, 4, 10);

        assert_eq!(links.first, "/api/users?page[number]=1&page[size]=10");
        assert_eq!(links.last, "/api/users?page[number]=4&page[size]=10");
        assert_eq!(links.self_link, "/api/users?page[number]=2&page[size]=10");
        assert_eq!(
            links.prev,
            Some("/api/users?page[number]=1&page[size]=10".to_string())
        );
        assert_eq!(
            links.next,
            Some("/api/users?page[number]=3&page[size]=10".to_string())
        );
    }

    #[test]
    fn test_pagination_links_first_page() {
        let links = PaginationLinks::new("/api/users", 1, 4, 10);

        assert!(links.prev.is_none());
        assert!(links.next.is_some());
    }

    #[test]
    fn test_pagination_links_last_page() {
        let links = PaginationLinks::new("/api/users", 4, 4, 10);

        assert!(links.prev.is_some());
        assert!(links.next.is_none());
    }

    #[test]
    fn test_pagination_links_preserve_query_params() {
        let links = PaginationLinks::new("/api/users?foo=bar&baz=qux", 1, 2, 10);

        assert!(links.first.contains("foo=bar"));
        assert!(links.first.contains("baz=qux"));
        assert!(links.first.contains("page[number]=1"));
        assert!(links.first.contains("page[size]=10"));
    }

    #[test]
    fn test_complete_pagination() {
        let pagination = Pagination::new(50, 2, 10, "/api/users");

        assert_eq!(pagination.meta.total, 50);
        assert_eq!(pagination.meta.current_page, 2);
        assert_eq!(
            pagination.links.self_link,
            "/api/users?page[number]=2&page[size]=10"
        );
    }

    #[test]
    fn test_pagination_preserves_include() {
        let links = PaginationLinks::new("/api/users?include=account,organization", 2, 4, 10);

        assert!(links.first.contains("include=account,organization"));
        assert!(links.last.contains("include=account,organization"));
        assert!(links.self_link.contains("include=account,organization"));
        assert!(links
            .prev
            .as_ref()
            .unwrap()
            .contains("include=account,organization"));
        assert!(links
            .next
            .as_ref()
            .unwrap()
            .contains("include=account,organization"));
    }

    #[test]
    fn test_pagination_preserves_fields() {
        let links = PaginationLinks::new(
            "/api/users?fields[users]=name,email&fields[accounts]=name",
            1,
            2,
            25,
        );

        assert!(links.first.contains("fields[users]=name,email"));
        assert!(links.first.contains("fields[accounts]=name"));
        assert!(links
            .next
            .as_ref()
            .unwrap()
            .contains("fields[users]=name,email"));
    }

    #[test]
    fn test_pagination_preserves_filters() {
        let links = PaginationLinks::new(
            "/api/users?filter[email][contains]=john&filter[active][eq]=true",
            1,
            3,
            15,
        );

        assert!(links.first.contains("filter[email][contains]=john"));
        assert!(links.first.contains("filter[active][eq]=true"));
        assert!(links.last.contains("filter[email][contains]=john"));
    }

    #[test]
    fn test_pagination_preserves_sort() {
        let links = PaginationLinks::new("/api/users?sort=-created_at,name", 2, 5, 20);

        assert!(links.first.contains("sort=-created_at,name"));
        assert!(links
            .prev
            .as_ref()
            .unwrap()
            .contains("sort=-created_at,name"));
        assert!(links
            .next
            .as_ref()
            .unwrap()
            .contains("sort=-created_at,name"));
    }

    #[test]
    fn test_pagination_preserves_all_jsonapi_params() {
        let url = "/api/users?include=account&fields[users]=name,email&filter[active][eq]=true&sort=-created_at";
        let links = PaginationLinks::new(url, 3, 10, 25);

        assert!(links.self_link.contains("include=account"));
        assert!(links.self_link.contains("fields[users]=name,email"));
        assert!(links.self_link.contains("filter[active][eq]=true"));
        assert!(links.self_link.contains("sort=-created_at"));
        assert!(links.self_link.contains("page[number]=3"));
        assert!(links.self_link.contains("page[size]=25"));
    }

    #[test]
    fn test_pagination_replaces_existing_page_params() {
        let links = PaginationLinks::new(
            "/api/users?page[number]=5&page[size]=50&include=account",
            2,
            4,
            25,
        );

        assert!(links.self_link.contains("page[number]=2"));
        assert!(links.self_link.contains("page[size]=25"));
        assert!(!links.self_link.contains("page[number]=5"));
        assert!(!links.self_link.contains("page[size]=50"));

        assert!(links.self_link.contains("include=account"));
    }
}
