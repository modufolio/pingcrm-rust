use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use urlencoding::encode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterOperator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Like,
    In,
    Null,
    NotNull,
}

impl FilterOperator {
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Neq => "!=",
            Self::Gt => ">",
            Self::Gte => ">=",
            Self::Lt => "<",
            Self::Lte => "<=",
            Self::Like => "LIKE",
            Self::In => "IN",
            Self::Null => "IS NULL",
            Self::NotNull => "IS NOT NULL",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "eq" => Some(Self::Eq),
            "neq" | "not" => Some(Self::Neq),
            "gt" => Some(Self::Gt),
            "gte" => Some(Self::Gte),
            "lt" => Some(Self::Lt),
            "lte" => Some(Self::Lte),
            "like" => Some(Self::Like),
            "in" => Some(Self::In),
            "null" => Some(Self::Null),
            "not_null" => Some(Self::NotNull),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: Option<String>,
}

impl FilterCondition {
    pub fn new(field: impl Into<String>, operator: FilterOperator, value: Option<String>) -> Self {
        Self {
            field: field.into(),
            operator,
            value,
        }
    }

    pub fn eq(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(field, FilterOperator::Eq, Some(value.into()))
    }

    pub fn like(field: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::new(field, FilterOperator::Like, Some(pattern.into()))
    }

    pub fn in_values(field: impl Into<String>, values: impl Into<String>) -> Self {
        Self::new(field, FilterOperator::In, Some(values.into()))
    }

    pub fn is_null(field: impl Into<String>) -> Self {
        Self::new(field, FilterOperator::Null, None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    #[serde(default)]
    pub fields: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub filters: Vec<FilterCondition>,

    #[serde(default)]
    pub includes: Vec<String>,

    #[serde(default)]
    pub sort: Vec<(String, SortDirection)>,

    #[serde(default)]
    pub page: PageParams,

    #[serde(default = "default_include_count")]
    pub include_count: bool,

    #[serde(default)]
    pub debug: bool,
}

fn default_include_count() -> bool {
    true
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
            filters: Vec::new(),
            includes: Vec::new(),
            sort: Vec::new(),
            page: PageParams::default(),
            include_count: true,
            debug: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchStrategy {
    Exact,
    Partial,
    StartsWith,
    EndsWith,
}

impl SearchStrategy {
    pub fn apply(&self, raw: &str) -> (FilterOperator, String) {
        match self {
            Self::Exact => (FilterOperator::Eq, raw.to_string()),
            Self::Partial => (FilterOperator::Like, format!("%{raw}%")),
            Self::StartsWith => (FilterOperator::Like, format!("{raw}%")),
            Self::EndsWith => (FilterOperator::Like, format!("%{raw}")),
        }
    }
}

pub const MIN_PAGE_SIZE: i64 = 1;
pub const DEFAULT_PAGE_SIZE: i64 = 25;
pub const MAX_PAGE_SIZE: i64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageParams {
    pub number: i64,

    pub size: i64,
}

impl Default for PageParams {
    fn default() -> Self {
        Self {
            number: 1,
            size: DEFAULT_PAGE_SIZE,
        }
    }
}

impl PageParams {
    pub fn normalize(&mut self) {
        if self.number < 1 {
            self.number = 1;
        }
        if self.size < MIN_PAGE_SIZE {
            self.size = MIN_PAGE_SIZE;
        }
        if self.size > MAX_PAGE_SIZE {
            self.size = MAX_PAGE_SIZE;
        }
    }
}

impl QueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_query_map(params: &HashMap<String, String>) -> Self {
        let mut query = Self::new();

        if let Some(page_number) = params.get("page[number]") {
            query.page.number = page_number.parse().unwrap_or(1);
        }
        if let Some(page_size) = params.get("page[size]") {
            query.page.size = page_size.parse().unwrap_or(25);
        }

        if let Some(page) = params.get("page") {
            query.page.number = page.parse().unwrap_or(1);
        }
        if let Some(per_page) = params.get("per_page") {
            query.page.size = per_page.parse().unwrap_or(25);
        }

        for (key, value) in params {
            if let Some(filter_part) = key.strip_prefix("filter[") {
                if let Some(field_end) = filter_part.find(']') {
                    let field = &filter_part[..field_end];
                    let remainder = &filter_part[field_end + 1..];

                    let (operator, filter_value) = if remainder.is_empty() {
                        (FilterOperator::Eq, Some(value.clone()))
                    } else if let Some(op_part) = remainder.strip_prefix('[') {
                        if let Some(op_end) = op_part.find(']') {
                            let op_str = &op_part[..op_end];
                            let operator =
                                FilterOperator::from_str(op_str).unwrap_or(FilterOperator::Eq);
                            let filter_value = if operator == FilterOperator::Null
                                || operator == FilterOperator::NotNull
                            {
                                None
                            } else {
                                Some(value.clone())
                            };
                            (operator, filter_value)
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };

                    query.filters.push(FilterCondition::new(
                        field.to_string(),
                        operator,
                        filter_value,
                    ));
                }
            }
        }

        if let Some(sort_str) = params.get("sort") {
            query.sort = sort_str
                .split(',')
                .filter_map(|s| {
                    let s = s.trim();
                    if s.is_empty() {
                        return None;
                    }

                    if let Some(field) = s.strip_prefix('-') {
                        Some((field.to_string(), SortDirection::Descending))
                    } else {
                        Some((s.to_string(), SortDirection::Ascending))
                    }
                })
                .collect();
        }

        if let Some(include_str) = params.get("include") {
            query.includes = include_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        for (key, value) in params {
            if let Some(type_part) = key.strip_prefix("fields[") {
                if let Some(type_end) = type_part.find(']') {
                    let resource_type = &type_part[..type_end];
                    let fields: Vec<String> = value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    query.fields.insert(resource_type.to_string(), fields);
                }
            }
        }

        if let Some(include_count_str) = params.get("include_count") {
            query.include_count = match include_count_str.to_lowercase().as_str() {
                "false" | "0" | "no" => false,
                _ => true,
            };
        }

        if let Some(debug_str) = params.get("debug") {
            query.debug = match debug_str.to_lowercase().as_str() {
                "true" | "1" | "yes" => true,
                _ => false,
            };
        }

        query.page.normalize();
        query
    }

    pub fn add_filter(mut self, condition: FilterCondition) -> Self {
        self.filters.push(condition);
        self
    }

    pub fn add_sort(mut self, field: impl Into<String>, direction: SortDirection) -> Self {
        self.sort.push((field.into(), direction));
        self
    }

    pub fn add_include(mut self, relationship: impl Into<String>) -> Self {
        self.includes.push(relationship.into());
        self
    }

    pub fn with_page(mut self, number: i64, size: i64) -> Self {
        self.page.number = number;
        self.page.size = size;
        self
    }

    pub fn filter(
        mut self,
        field: impl Into<String>,
        operator: FilterOperator,
        value: Option<String>,
    ) -> Self {
        self.filters
            .push(FilterCondition::new(field, operator, value));
        self
    }

    pub fn sort(mut self, sort_str: &str) -> Self {
        for s in sort_str.split(',') {
            let s = s.trim();
            if s.is_empty() {
                continue;
            }

            if let Some(field) = s.strip_prefix('-') {
                self.sort
                    .push((field.to_string(), SortDirection::Descending));
            } else {
                self.sort.push((s.to_string(), SortDirection::Ascending));
            }
        }
        self
    }

    pub fn include(mut self, relationships: &[&str]) -> Self {
        for rel in relationships {
            self.includes.push(rel.to_string());
        }
        self
    }

    pub fn fields(mut self, resource_type: &str, field_names: &[&str]) -> Self {
        self.fields.insert(
            resource_type.to_string(),
            field_names.iter().map(|s| s.to_string()).collect(),
        );
        self
    }

    pub fn page(self, number: i64, size: i64) -> Self {
        self.with_page(number, size)
    }

    pub fn with_debug(mut self, enabled: bool) -> Self {
        self.debug = enabled;
        self
    }

    pub fn without_count(mut self) -> Self {
        self.include_count = false;
        self
    }

    pub fn to_query_string_without_pagination(&self) -> String {
        let mut parts = Vec::new();

        if !self.includes.is_empty() {
            let includes_str = self.includes.join(",");
            let includes_encoded = encode(&includes_str);
            parts.push(format!("include={}", includes_encoded));
        }

        for (resource_type, fields) in &self.fields {
            if !fields.is_empty() {
                let resource_type_encoded = encode(resource_type);
                let fields_str = fields.join(",");
                let fields_encoded = encode(&fields_str);
                parts.push(format!(
                    "fields[{}]={}",
                    resource_type_encoded, fields_encoded
                ));
            }
        }

        for filter in &self.filters {
            let operator_str = match filter.operator {
                FilterOperator::Eq => "eq",
                FilterOperator::Neq => "neq",
                FilterOperator::Gt => "gt",
                FilterOperator::Gte => "gte",
                FilterOperator::Lt => "lt",
                FilterOperator::Lte => "lte",
                FilterOperator::Like => "like",
                FilterOperator::In => "in",
                FilterOperator::Null => "null",
                FilterOperator::NotNull => "not_null",
            };

            let field_encoded = encode(&filter.field);
            if let Some(ref value) = filter.value {
                let value_encoded = encode(value);
                parts.push(format!(
                    "filter[{}][{}]={}",
                    field_encoded, operator_str, value_encoded
                ));
            } else {
                parts.push(format!("filter[{}][{}]=", field_encoded, operator_str));
            }
        }

        if !self.sort.is_empty() {
            let sort_str = self
                .sort
                .iter()
                .map(|(field, direction)| match direction {
                    SortDirection::Ascending => field.clone(),
                    SortDirection::Descending => format!("-{}", field),
                })
                .collect::<Vec<_>>()
                .join(",");
            let sort_encoded = encode(&sort_str);
            parts.push(format!("sort={}", sort_encoded));
        }

        parts.join("&")
    }

    pub fn to_query_string(&self) -> String {
        let mut parts = Vec::new();

        let base = self.to_query_string_without_pagination();
        if !base.is_empty() {
            parts.push(base);
        }

        parts.push(format!("page[number]={}", self.page.number));
        parts.push(format!("page[size]={}", self.page.size));

        parts.join("&")
    }

    pub fn build_url(&self, base_path: &str) -> String {
        let query_string = self.to_query_string();
        if query_string.is_empty() {
            base_path.to_string()
        } else {
            format!("{}?{}", base_path, query_string)
        }
    }

    pub fn build_url_with_page(&self, base_path: &str, page_number: i64, page_size: i64) -> String {
        let base = self.to_query_string_without_pagination();
        let query_string = if base.is_empty() {
            format!("page[number]={}&page[size]={}", page_number, page_size)
        } else {
            format!(
                "{}&page[number]={}&page[size]={}",
                base, page_number, page_size
            )
        };
        format!("{}?{}", base_path, query_string)
    }

    pub fn validate_includes(&self, allowed: &[&str]) -> Result<(), crate::error::AppError> {
        for include in &self.includes {
            if !allowed.contains(&include.as_str()) {
                return Err(crate::error::AppError::invalid_include(include));
            }
        }
        Ok(())
    }

    pub fn validate_fields(
        &self,
        resource_type: &str,
        allowed: &[&str],
    ) -> Result<(), crate::error::AppError> {
        if let Some(fields) = self.fields.get(resource_type) {
            for field in fields {
                if !allowed.contains(&field.as_str()) {
                    return Err(crate::error::AppError::invalid_field(field, resource_type));
                }
            }
        }
        Ok(())
    }

    pub fn validate_sorts(&self, allowed: &[&str]) -> Result<(), crate::error::AppError> {
        for (field, _) in &self.sort {
            if !allowed.contains(&field.as_str()) {
                return Err(crate::error::AppError::invalid_sort_field(field));
            }
        }
        Ok(())
    }

    pub fn validate_filters(&self, allowed: &[&str]) -> Result<(), crate::error::AppError> {
        for filter in &self.filters {
            if !allowed.contains(&filter.field.as_str()) {
                return Err(crate::error::AppError::invalid_filter_field(&filter.field));
            }
        }
        Ok(())
    }

    pub fn validate(
        &self,
        resource_type: &str,
        allowed_fields: &[&str],
        allowed_includes: &[&str],
    ) -> Result<(), crate::error::AppError> {
        self.validate_includes(allowed_includes)?;
        self.validate_fields(resource_type, allowed_fields)?;
        self.validate_sorts(allowed_fields)?;
        self.validate_filters(allowed_fields)?;
        Ok(())
    }
}

use axum::{extract::FromRequestParts, http::request::Parts};

impl<S> FromRequestParts<S> for QueryParams
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query_string = parts.uri.query().unwrap_or("");

        let mut params = HashMap::new();

        for pair in query_string.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                let key = urlencoding::decode(key).unwrap_or_default().into_owned();
                let value = urlencoding::decode(value).unwrap_or_default().into_owned();
                params.insert(key, value);
            }
        }

        Ok(QueryParams::from_query_map(&params))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pagination() {
        let mut params = HashMap::new();
        params.insert("page[number]".to_string(), "3".to_string());
        params.insert("page[size]".to_string(), "50".to_string());

        let query = QueryParams::from_query_map(&params);

        assert_eq!(query.page.number, 3);
        assert_eq!(query.page.size, 50);
    }

    #[test]
    fn test_parse_simple_filter() {
        let mut params = HashMap::new();
        params.insert("filter[name]".to_string(), "John".to_string());

        let query = QueryParams::from_query_map(&params);

        assert_eq!(query.filters.len(), 1);
        assert_eq!(query.filters[0].field, "name");
        assert_eq!(query.filters[0].operator, FilterOperator::Eq);
        assert_eq!(query.filters[0].value, Some("John".to_string()));
    }

    #[test]
    fn test_parse_operator_filter() {
        let mut params = HashMap::new();
        params.insert("filter[age][gt]".to_string(), "18".to_string());
        params.insert(
            "filter[email][like]".to_string(),
            "%@example.com".to_string(),
        );

        let query = QueryParams::from_query_map(&params);

        assert_eq!(query.filters.len(), 2);

        let age_filter = query.filters.iter().find(|f| f.field == "age").unwrap();
        assert_eq!(age_filter.operator, FilterOperator::Gt);
        assert_eq!(age_filter.value, Some("18".to_string()));

        let email_filter = query.filters.iter().find(|f| f.field == "email").unwrap();
        assert_eq!(email_filter.operator, FilterOperator::Like);
    }

    #[test]
    fn test_parse_sort() {
        let mut params = HashMap::new();
        params.insert("sort".to_string(), "name,-created_at,title".to_string());

        let query = QueryParams::from_query_map(&params);

        assert_eq!(query.sort.len(), 3);
        assert_eq!(
            query.sort[0],
            ("name".to_string(), SortDirection::Ascending)
        );
        assert_eq!(
            query.sort[1],
            ("created_at".to_string(), SortDirection::Descending)
        );
        assert_eq!(
            query.sort[2],
            ("title".to_string(), SortDirection::Ascending)
        );
    }

    #[test]
    fn test_parse_include() {
        let mut params = HashMap::new();
        params.insert(
            "include".to_string(),
            "author,comments.author,tags".to_string(),
        );

        let query = QueryParams::from_query_map(&params);

        assert_eq!(query.includes.len(), 3);
        assert!(query.includes.contains(&"author".to_string()));
        assert!(query.includes.contains(&"comments.author".to_string()));
    }

    #[test]
    fn test_parse_sparse_fieldsets() {
        let mut params = HashMap::new();
        params.insert("fields[users]".to_string(), "name,email".to_string());
        params.insert(
            "fields[posts]".to_string(),
            "title,body,created_at".to_string(),
        );

        let query = QueryParams::from_query_map(&params);

        assert_eq!(query.fields.len(), 2);
        assert_eq!(
            query.fields.get("users").unwrap(),
            &vec!["name".to_string(), "email".to_string()]
        );
        assert_eq!(query.fields.get("posts").unwrap().len(), 3);
    }

    #[test]
    fn test_builder_pattern() {
        let query = QueryParams::new()
            .add_filter(FilterCondition::eq("name", "John"))
            .add_sort("created_at", SortDirection::Descending)
            .add_include("author")
            .with_page(2, 10);

        assert_eq!(query.filters.len(), 1);
        assert_eq!(query.sort.len(), 1);
        assert_eq!(query.includes.len(), 1);
        assert_eq!(query.page.number, 2);
        assert_eq!(query.page.size, 10);
    }

    #[test]
    fn test_validate_includes_valid() {
        let query = QueryParams::new()
            .add_include("account")
            .add_include("profile");
        let result = query.validate_includes(&["account", "profile", "posts"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_includes_invalid() {
        let query = QueryParams::new()
            .add_include("account")
            .add_include("invalid");
        let result = query.validate_includes(&["account", "profile"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid include"));
    }

    #[test]
    fn test_validate_fields_valid() {
        let mut query = QueryParams::new();
        query.fields.insert(
            "users".to_string(),
            vec!["name".to_string(), "email".to_string()],
        );
        let result = query.validate_fields("users", &["id", "name", "email", "created_at"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_fields_invalid() {
        let mut query = QueryParams::new();
        query.fields.insert(
            "users".to_string(),
            vec!["name".to_string(), "invalid".to_string()],
        );
        let result = query.validate_fields("users", &["id", "name", "email"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid field"));
    }

    #[test]
    fn test_validate_sorts_valid() {
        let query = QueryParams::new()
            .add_sort("created_at", SortDirection::Descending)
            .add_sort("name", SortDirection::Ascending);
        let result = query.validate_sorts(&["id", "name", "created_at", "updated_at"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_sorts_invalid() {
        let query = QueryParams::new().add_sort("invalid_field", SortDirection::Ascending);
        let result = query.validate_sorts(&["id", "name", "created_at"]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid sort field"));
    }

    #[test]
    fn test_validate_all_valid() {
        let mut query = QueryParams::new()
            .add_include("account")
            .add_sort("created_at", SortDirection::Descending);
        query
            .fields
            .insert("users".to_string(), vec!["name".to_string()]);

        let result = query.validate(
            "users",
            &["id", "name", "email", "created_at"],
            &["account", "profile"],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_all_invalid_include() {
        let query = QueryParams::new().add_include("invalid");
        let result = query.validate("users", &["id", "name"], &["account"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid include"));
    }

    #[test]
    fn test_fluent_filter() {
        let params = QueryParams::new()
            .filter("status", FilterOperator::Eq, Some("active".to_string()))
            .filter("price", FilterOperator::Gt, Some("100".to_string()))
            .filter("deleted_at", FilterOperator::Null, None);

        assert_eq!(params.filters.len(), 3);
        assert_eq!(params.filters[0].field, "status");
        assert_eq!(params.filters[0].operator, FilterOperator::Eq);
        assert_eq!(params.filters[1].field, "price");
        assert_eq!(params.filters[1].operator, FilterOperator::Gt);
        assert_eq!(params.filters[2].field, "deleted_at");
        assert_eq!(params.filters[2].operator, FilterOperator::Null);
    }

    #[test]
    fn test_fluent_sort_string() {
        let params = QueryParams::new().sort("-created_at,name,-price");

        assert_eq!(params.sort.len(), 3);
        assert_eq!(
            params.sort[0],
            ("created_at".to_string(), SortDirection::Descending)
        );
        assert_eq!(
            params.sort[1],
            ("name".to_string(), SortDirection::Ascending)
        );
        assert_eq!(
            params.sort[2],
            ("price".to_string(), SortDirection::Descending)
        );
    }

    #[test]
    fn test_fluent_sort_empty_parts() {
        let params = QueryParams::new().sort("-created_at, , name");

        assert_eq!(params.sort.len(), 2);
        assert_eq!(
            params.sort[0],
            ("created_at".to_string(), SortDirection::Descending)
        );
        assert_eq!(
            params.sort[1],
            ("name".to_string(), SortDirection::Ascending)
        );
    }

    #[test]
    fn test_fluent_include_multiple() {
        let params = QueryParams::new().include(&["brand", "categories", "account"]);

        assert_eq!(params.includes.len(), 3);
        assert!(params.includes.contains(&"brand".to_string()));
        assert!(params.includes.contains(&"categories".to_string()));
        assert!(params.includes.contains(&"account".to_string()));
    }

    #[test]
    fn test_fluent_fields() {
        let params = QueryParams::new()
            .fields("products", &["id", "name", "price"])
            .fields("brands", &["id", "name"]);

        assert_eq!(params.fields.len(), 2);
        assert_eq!(
            params.fields.get("products").unwrap(),
            &vec!["id".to_string(), "name".to_string(), "price".to_string()]
        );
        assert_eq!(
            params.fields.get("brands").unwrap(),
            &vec!["id".to_string(), "name".to_string()]
        );
    }

    #[test]
    fn test_fluent_page() {
        let params = QueryParams::new().page(3, 50);

        assert_eq!(params.page.number, 3);
        assert_eq!(params.page.size, 50);
    }

    #[test]
    fn test_with_debug() {
        let params = QueryParams::new().with_debug(true);
        assert_eq!(params.debug, true);

        let params2 = QueryParams::new().with_debug(false);
        assert_eq!(params2.debug, false);
    }

    #[test]
    fn test_without_count() {
        let params = QueryParams::new().without_count();
        assert_eq!(params.include_count, false);
    }

    #[test]
    fn test_fluent_chaining_complex() {
        let params = QueryParams::new()
            .filter("status", FilterOperator::Eq, Some("active".to_string()))
            .filter("price", FilterOperator::Gte, Some("50".to_string()))
            .sort("-created_at,name")
            .include(&["brand", "categories"])
            .fields("products", &["id", "name", "price"])
            .page(2, 25)
            .with_debug(true)
            .without_count();

        assert_eq!(params.filters.len(), 2);
        assert_eq!(params.sort.len(), 2);
        assert_eq!(params.includes.len(), 2);
        assert_eq!(params.fields.len(), 1);
        assert_eq!(params.page.number, 2);
        assert_eq!(params.page.size, 25);
        assert_eq!(params.debug, true);
        assert_eq!(params.include_count, false);
    }

    #[test]
    fn test_to_query_string_complete() {
        let params = QueryParams::new()
            .filter("status", FilterOperator::Eq, Some("active".to_string()))
            .sort("-created_at")
            .page(2, 25);

        let query_string = params.to_query_string();

        assert!(query_string.contains("filter[status][eq]=active"));
        assert!(
            query_string.contains("sort=-created_at")
                || query_string.contains("sort=%2Dcreated_at")
        );
        assert!(query_string.contains("page[number]=2"));
        assert!(query_string.contains("page[size]=25"));
    }

    #[test]
    fn test_url_encoding_in_query_string() {
        let params = QueryParams::new()
            .filter("name", FilterOperator::Like, Some("John Doe".to_string()))
            .filter(
                "email",
                FilterOperator::Eq,
                Some("test@example.com".to_string()),
            )
            .page(1, 10);

        let query_string = params.to_query_string();

        assert!(query_string.contains("John%20Doe"));

        assert!(query_string.contains("test%40example.com"));
    }

    #[test]
    fn test_build_url() {
        let params = QueryParams::new()
            .filter("status", FilterOperator::Eq, Some("active".to_string()))
            .sort("-created_at")
            .page(1, 25);

        let url = params.build_url("/api/v1/products");

        assert!(url.starts_with("/api/v1/products?"));
        assert!(url.contains("filter[status][eq]=active"));
        assert!(url.contains("sort=-created_at"));
        assert!(url.contains("page[number]=1"));
        assert!(url.contains("page[size]=25"));
    }

    #[test]
    fn test_build_url_with_page() {
        let params = QueryParams::new()
            .filter("status", FilterOperator::Eq, Some("active".to_string()))
            .page(1, 25);

        let next_url = params.build_url_with_page("/api/v1/products", 2, 25);

        assert!(next_url.contains("page[number]=2"));
        assert!(next_url.contains("page[size]=25"));
        assert!(next_url.contains("filter[status][eq]=active"));
    }

    #[test]
    fn test_build_url_with_includes_and_fields() {
        let params = QueryParams::new()
            .include(&["brand", "categories"])
            .fields("products", &["id", "name", "price"])
            .page(1, 10);

        let url = params.build_url("/api/v1/products");

        assert!(
            url.contains("include=brand%2Ccategories") || url.contains("include=brand,categories")
        );
        assert!(
            url.contains("fields[products]=id%2Cname%2Cprice")
                || url.contains("fields[products]=id,name,price")
        );
        assert!(url.contains("page[number]=1"));
        assert!(url.contains("page[size]=10"));
    }

    #[test]
    fn test_parse_debug_parameter() {
        let mut params = HashMap::new();
        params.insert("debug".to_string(), "true".to_string());

        let query = QueryParams::from_query_map(&params);
        assert_eq!(query.debug, true);

        let mut params2 = HashMap::new();
        params2.insert("debug".to_string(), "1".to_string());
        let query2 = QueryParams::from_query_map(&params2);
        assert_eq!(query2.debug, true);

        let mut params3 = HashMap::new();
        params3.insert("debug".to_string(), "false".to_string());
        let query3 = QueryParams::from_query_map(&params3);
        assert_eq!(query3.debug, false);
    }

    #[test]
    fn test_default_has_debug_false() {
        let params = QueryParams::default();
        assert_eq!(params.debug, false);
    }
}
