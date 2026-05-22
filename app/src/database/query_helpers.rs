use crate::database::pool::{DbConnection, DbPool};

use appkit_core::error::AppError;
use diesel::query_dsl::methods::{LimitDsl, OffsetDsl};

pub async fn get_conn(
    pool: &DbPool,
) -> Result<impl std::ops::DerefMut<Target = DbConnection> + '_, AppError> {
    pool.get()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database connection error: {}", e)))
}

pub fn sanitize_like_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[derive(Debug, Clone)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub page_number: i64,
    pub page_size: i64,
}

impl<T> Paginated<T> {
    pub fn total_pages(&self) -> i64 {
        if self.total_count == 0 {
            return 1;
        }
        ((self.total_count as f64 / self.page_size as f64).ceil() as i64).max(1)
    }

    pub fn offset(&self) -> i64 {
        (self.page_number - 1) * self.page_size
    }
}

pub trait Paginatable: Sized {
    fn paginate_query(self, page_number: i64, page_size: i64) -> PaginationHelper<Self> {
        let offset = (page_number - 1) * page_size;
        PaginationHelper {
            query: self,
            page_number,
            page_size,
            offset,
        }
    }
}

impl<T> Paginatable for T {}

pub struct PaginationHelper<T> {
    query: T,
    #[allow(dead_code)]
    page_number: i64,
    page_size: i64,
    offset: i64,
}

impl<T> PaginationHelper<T>
where
    T: LimitDsl,
{
    pub fn apply_limit(self) -> <<T as LimitDsl>::Output as OffsetDsl>::Output
    where
        <T as LimitDsl>::Output: OffsetDsl,
    {
        self.query.limit(self.page_size).offset(self.offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_total_pages() {
        let paginated = Paginated {
            items: vec![1, 2, 3],
            total_count: 100,
            page_number: 1,
            page_size: 10,
        };

        assert_eq!(paginated.total_pages(), 10);
    }

    #[test]
    fn test_paginated_total_pages_with_remainder() {
        let paginated = Paginated {
            items: vec![1, 2, 3],
            total_count: 95,
            page_number: 1,
            page_size: 10,
        };

        assert_eq!(paginated.total_pages(), 10);
    }

    #[test]
    fn test_paginated_total_pages_empty() {
        let paginated: Paginated<i32> = Paginated {
            items: vec![],
            total_count: 0,
            page_number: 1,
            page_size: 10,
        };

        assert_eq!(paginated.total_pages(), 1);
    }

    #[test]
    fn test_paginated_offset() {
        let paginated: Paginated<i32> = Paginated {
            items: vec![],
            total_count: 100,
            page_number: 3,
            page_size: 10,
        };

        assert_eq!(paginated.offset(), 20);
    }

    #[test]
    fn test_sanitize_like_value() {
        assert_eq!(sanitize_like_value("100%"), "100\\%");

        assert_eq!(sanitize_like_value("test_value"), "test\\_value");

        assert_eq!(sanitize_like_value("100%_pure"), "100\\%\\_pure");

        assert_eq!(sanitize_like_value("%%%%%%%%"), "\\%\\%\\%\\%\\%\\%\\%\\%");

        assert_eq!(sanitize_like_value("normal"), "normal");

        assert_eq!(sanitize_like_value("path\\to\\file"), "path\\\\to\\\\file");

        assert_eq!(sanitize_like_value("\\%_"), "\\\\\\%\\_");
    }
}

#[macro_export]
macro_rules! apply_filter_to_query {
    ($query:expr, $filter:expr, $( $field_name:literal => $field:expr ),* $(,)?) => {{
        use appkit_core::jsonapi::FilterOperator;
        use diesel::prelude::*;

        let mut query = $query;
        match $filter.field.as_str() {
            $(
                $field_name => {
                    query = match $filter.operator {
                        FilterOperator::Eq => {
                            if let Some(ref value) = $filter.value {
                                query.filter($field.eq(value))
                            } else {
                                query
                            }
                        }
                        FilterOperator::Neq => {
                            if let Some(ref value) = $filter.value {
                                query.filter($field.ne(value))
                            } else {
                                query
                            }
                        }
                        FilterOperator::Like => {
                            if let Some(ref value) = $filter.value {

                                let sanitized = $crate::database::query_helpers::sanitize_like_value(value);

                                query.filter($field.like(format!("%{}%", sanitized)).escape('\\'))
                            } else {
                                query
                            }
                        }
                        FilterOperator::Null => {
                            query.filter($field.is_null())
                        }
                        FilterOperator::NotNull => {
                            query.filter($field.is_not_null())
                        }
                        _ => query,
                    };
                }
            )*
            _ => {

            }
        }
        query
    }};
}

#[macro_export]
macro_rules! apply_sort_to_query {
    ($query:expr, $field:expr, $direction:expr, $( $field_name:literal => $field_expr:expr ),* $(,)?) => {{
        use appkit_core::jsonapi::SortDirection;
        use diesel::prelude::*;

        let mut query = $query;
        match $field.as_str() {
            $(
                $field_name => {
                    query = match $direction {
                        SortDirection::Ascending => query.then_order_by($field_expr.asc()),
                        SortDirection::Descending => query.then_order_by($field_expr.desc()),
                    };
                }
            )*
            _ => {

            }
        }
        query
    }};
}

#[macro_export]
macro_rules! apply_filters {
    ($query:expr, $filters:expr, $( $field_name:literal => $field:expr ),* $(,)?) => {{
        let mut query = $query;
        for filter in $filters {
            query = $crate::apply_filter_to_query!(query, filter, $( $field_name => $field ),*);
        }
        query
    }};
}

#[macro_export]
macro_rules! apply_sorts {
    ($query:expr, $sorts:expr, $( $field_name:literal => $field:expr ),* $(,)?) => {{
        let mut query = $query;
        for (field, direction) in $sorts {
            query = $crate::apply_sort_to_query!(query, field, direction, $( $field_name => $field ),*);
        }
        query
    }};
}
