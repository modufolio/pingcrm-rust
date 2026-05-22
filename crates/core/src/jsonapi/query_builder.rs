use crate::jsonapi::{FilterCondition, SortDirection};

#[allow(unused_imports)]
use diesel::prelude::*;

#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

impl<T> PaginatedResult<T> {
    pub fn new(items: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        Self {
            items,
            total,
            page,
            per_page,
        }
    }

    pub fn last_page(&self) -> i64 {
        if self.total == 0 || self.per_page == 0 {
            1
        } else {
            ((self.total as f64) / (self.per_page as f64)).ceil() as i64
        }
    }
}

pub trait ApplyJsonApiFilter: Sized {
    fn apply_filter(self, filter: &FilterCondition) -> Self;
    fn apply_filters(self, filters: &[FilterCondition]) -> Self;
}

pub trait ApplyJsonApiSort: Sized {
    fn apply_sort(self, field: &str, direction: SortDirection) -> Self;
    fn apply_sorts(self, sorts: &[(String, SortDirection)]) -> Self;
}

pub trait ApplyJsonApiPagination: Sized {
    fn apply_pagination(self, page: i64, per_page: i64) -> (Self, i64);
}

#[allow(unused_macros)]
macro_rules! diesel_filter {
    ($query:expr, $field:expr, $op:expr, $value:expr) => {
        match $op {
            FilterOperator::Eq => $query.filter($field.eq($value)),
            FilterOperator::Neq => $query.filter($field.ne($value)),
            FilterOperator::Gt => $query.filter($field.gt($value)),
            FilterOperator::Gte => $query.filter($field.ge($value)),
            FilterOperator::Lt => $query.filter($field.lt($value)),
            FilterOperator::Lte => $query.filter($field.le($value)),
            FilterOperator::Like => $query.filter($field.like($value)),
            _ => $query,
        }
    };
}

pub fn apply_text_filter<'a, ST, QS, DB, GB>(
    query: diesel::query_builder::BoxedSelectStatement<'a, ST, QS, DB, GB>,
    _field_name: &str,
    _condition: &FilterCondition,
) -> diesel::query_builder::BoxedSelectStatement<'a, ST, QS, DB, GB>
where
    DB: diesel::backend::Backend,
{
    query
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_result() {
        let result = PaginatedResult::new(vec![1, 2, 3], 100, 2, 25);

        assert_eq!(result.items.len(), 3);
        assert_eq!(result.total, 100);
        assert_eq!(result.page, 2);
        assert_eq!(result.per_page, 25);
        assert_eq!(result.last_page(), 4);
    }

    #[test]
    fn test_pagination_offset() {
        let offset = (2 - 1) * 25;
        assert_eq!(offset, 25);

        let offset = (1 - 1) * 25;
        assert_eq!(offset, 0);
    }
}
