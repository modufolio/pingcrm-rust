use appkit_core::jsonapi::pagination::{PaginationLinks, PaginationMeta};
use appkit_core::jsonapi::query_builder::PaginatedResult;

use appkit_core::jsonapi::{
    FilterCondition, FilterOperator, PageParams, ResourceObject, SortDirection,
};
use serde_json::json;

#[test]
fn test_pagination_meta_first_page() {
    let meta = PaginationMeta::new(100, 1, 10);

    assert_eq!(meta.total, 100);
    assert_eq!(meta.current_page, 1);
    assert_eq!(meta.per_page, 10);
    assert_eq!(meta.last_page, 10);
    assert_eq!(meta.from, 1);
    assert_eq!(meta.to, 10);
}

#[test]
fn test_pagination_meta_middle_page() {
    let meta = PaginationMeta::new(100, 5, 10);

    assert_eq!(meta.total, 100);
    assert_eq!(meta.current_page, 5);
    assert_eq!(meta.from, 41);
    assert_eq!(meta.to, 50);
}

#[test]
fn test_pagination_meta_last_page() {
    let meta = PaginationMeta::new(95, 10, 10);

    assert_eq!(meta.last_page, 10);
    assert_eq!(meta.from, 91);
    assert_eq!(meta.to, 95);
}

#[test]
fn test_pagination_meta_empty_results() {
    let meta = PaginationMeta::new(0, 1, 10);

    assert_eq!(meta.total, 0);
    assert_eq!(meta.last_page, 1);
    assert_eq!(meta.from, 0);
    assert_eq!(meta.to, 0);
}

#[test]
fn test_pagination_meta_single_item() {
    let meta = PaginationMeta::new(1, 1, 10);

    assert_eq!(meta.total, 1);
    assert_eq!(meta.last_page, 1);
    assert_eq!(meta.from, 1);
    assert_eq!(meta.to, 1);
}

#[test]
fn test_pagination_meta_exact_page_boundary() {
    let meta = PaginationMeta::new(50, 5, 10);

    assert_eq!(meta.last_page, 5);
    assert_eq!(meta.from, 41);
    assert_eq!(meta.to, 50);
}

#[test]
fn test_pagination_meta_large_per_page() {
    let meta = PaginationMeta::new(100, 1, 200);

    assert_eq!(meta.last_page, 1);
    assert_eq!(meta.from, 1);
    assert_eq!(meta.to, 100);
}

#[test]
fn test_pagination_meta_to_value() {
    let meta = PaginationMeta::new(100, 2, 25);
    let value = meta.to_value();

    assert_eq!(value["total"], 100);
    assert_eq!(value["current_page"], 2);
    assert_eq!(value["per_page"], 25);
    assert_eq!(value["last_page"], 4);
    assert_eq!(value["from"], 26);
    assert_eq!(value["to"], 50);
}

#[test]
fn test_pagination_meta_to_map() {
    let meta = PaginationMeta::new(100, 3, 20);
    let map = meta.to_map();

    assert_eq!(map.get("total"), Some(&json!(100)));
    assert_eq!(map.get("current_page"), Some(&json!(3)));
    assert_eq!(map.get("per_page"), Some(&json!(20)));
    assert_eq!(map.len(), 6);
}

#[test]
fn test_pagination_links_structure() {
    let links = PaginationLinks {
        first: "/users?page[number]=1&page[size]=10".to_string(),
        last: "/users?page[number]=10&page[size]=10".to_string(),
        prev: Some("/users?page[number]=4&page[size]=10".to_string()),
        next: Some("/users?page[number]=6&page[size]=10".to_string()),
        self_link: "/users?page[number]=5&page[size]=10".to_string(),
    };

    assert!(links.first.contains("page[number]=1"));
    assert!(links.last.contains("page[number]=10"));
    assert!(links.prev.is_some());
    assert!(links.next.is_some());
}

#[test]
fn test_pagination_links_first_page_no_prev() {
    let links = PaginationLinks {
        first: "/users?page[number]=1&page[size]=10".to_string(),
        last: "/users?page[number]=5&page[size]=10".to_string(),
        prev: None,
        next: Some("/users?page[number]=2&page[size]=10".to_string()),
        self_link: "/users?page[number]=1&page[size]=10".to_string(),
    };

    assert!(links.prev.is_none());
    assert!(links.next.is_some());
}

#[test]
fn test_pagination_links_last_page_no_next() {
    let links = PaginationLinks {
        first: "/users?page[number]=1&page[size]=10".to_string(),
        last: "/users?page[number]=5&page[size]=10".to_string(),
        prev: Some("/users?page[number]=4&page[size]=10".to_string()),
        next: None,
        self_link: "/users?page[number]=5&page[size]=10".to_string(),
    };

    assert!(links.prev.is_some());
    assert!(links.next.is_none());
}

#[test]
fn test_pagination_links_serialization() {
    let links = PaginationLinks {
        first: "/api/users?page[number]=1&page[size]=25".to_string(),
        last: "/api/users?page[number]=4&page[size]=25".to_string(),
        prev: Some("/api/users?page[number]=1&page[size]=25".to_string()),
        next: Some("/api/users?page[number]=3&page[size]=25".to_string()),
        self_link: "/api/users?page[number]=2&page[size]=25".to_string(),
    };

    let json = serde_json::to_value(&links).unwrap();

    assert_eq!(json["first"], "/api/users?page[number]=1&page[size]=25");
    assert_eq!(json["last"], "/api/users?page[number]=4&page[size]=25");
    assert_eq!(json["self"], "/api/users?page[number]=2&page[size]=25");
    assert!(json["prev"].is_string());
    assert!(json["next"].is_string());
}

#[test]
fn test_pagination_links_skip_serializing_none() {
    let links = PaginationLinks {
        first: "/users?page[number]=1&page[size]=10".to_string(),
        last: "/users?page[number]=1&page[size]=10".to_string(),
        prev: None,
        next: None,
        self_link: "/users?page[number]=1&page[size]=10".to_string(),
    };

    let json = serde_json::to_value(&links).unwrap();

    assert!(json["prev"].is_null());
    assert!(json["next"].is_null());
}

#[test]
fn test_paginated_result_creation() {
    let items = vec![1, 2, 3, 4, 5];
    let result = PaginatedResult::new(items.clone(), 50, 1, 5);

    assert_eq!(result.items, items);
    assert_eq!(result.total, 50);
    assert_eq!(result.page, 1);
    assert_eq!(result.per_page, 5);
}

#[test]
fn test_paginated_result_last_page_calculation() {
    let result = PaginatedResult::new(vec![1, 2, 3], 100, 1, 10);
    assert_eq!(result.last_page(), 10);

    let result = PaginatedResult::new(vec![1, 2, 3], 95, 1, 10);
    assert_eq!(result.last_page(), 10);

    let result = PaginatedResult::new(vec![1], 1, 1, 10);
    assert_eq!(result.last_page(), 1);
}

#[test]
fn test_paginated_result_empty() {
    let result: PaginatedResult<i32> = PaginatedResult::new(vec![], 0, 1, 10);

    assert_eq!(result.items.len(), 0);
    assert_eq!(result.total, 0);
    assert_eq!(result.last_page(), 1);
}

#[test]
fn test_paginated_result_zero_per_page() {
    let result = PaginatedResult::new(vec![1, 2, 3], 10, 1, 0);

    assert_eq!(result.last_page(), 1);
}

#[test]
fn test_page_params_creation() {
    let params = PageParams {
        number: 2,
        size: 25,
    };

    assert_eq!(params.number, 2);
    assert_eq!(params.size, 25);
}

#[test]
fn test_page_params_offset_calculation() {
    let params = PageParams {
        number: 1,
        size: 10,
    };
    assert_eq!((params.number - 1) * params.size, 0);

    let params = PageParams {
        number: 3,
        size: 25,
    };
    assert_eq!((params.number - 1) * params.size, 50);

    let params = PageParams {
        number: 10,
        size: 100,
    };
    assert_eq!((params.number - 1) * params.size, 900);
}

#[test]
fn test_filter_condition_equality() {
    let filter = FilterCondition {
        field: "email".to_string(),
        operator: FilterOperator::Eq,
        value: Some("test@example.com".to_string()),
    };

    assert_eq!(filter.field, "email");
    assert_eq!(filter.operator, FilterOperator::Eq);
    assert_eq!(filter.value, Some("test@example.com".to_string()));
}

#[test]
fn test_filter_condition_greater_than() {
    let filter = FilterCondition {
        field: "age".to_string(),
        operator: FilterOperator::Gt,
        value: Some("18".to_string()),
    };

    assert_eq!(filter.operator, FilterOperator::Gt);
    assert_eq!(filter.value, Some("18".to_string()));
}

#[test]
fn test_filter_condition_like() {
    let filter = FilterCondition {
        field: "name".to_string(),
        operator: FilterOperator::Like,
        value: Some("John%".to_string()),
    };

    assert_eq!(filter.operator, FilterOperator::Like);
}

#[test]
fn test_filter_condition_null() {
    let filter = FilterCondition {
        field: "deleted_at".to_string(),
        operator: FilterOperator::Null,
        value: None,
    };

    assert_eq!(filter.operator, FilterOperator::Null);
    assert!(filter.value.is_none());
}

#[test]
fn test_filter_condition_in_operator() {
    let filter = FilterCondition {
        field: "status".to_string(),
        operator: FilterOperator::In,
        value: Some("active,pending,approved".to_string()),
    };

    assert_eq!(filter.operator, FilterOperator::In);
    assert!(filter.value.as_ref().unwrap().contains(','));
}

#[test]
fn test_filter_condition_with_special_chars() {
    let filter = FilterCondition {
        field: "description".to_string(),
        operator: FilterOperator::Like,
        value: Some("100% pure".to_string()),
    };

    assert_eq!(filter.value, Some("100% pure".to_string()));
}

#[test]
fn test_sort_direction_ascending() {
    let sort = SortDirection::Ascending;
    assert_eq!(sort, SortDirection::Ascending);
}

#[test]
fn test_sort_direction_descending() {
    let sort = SortDirection::Descending;
    assert_eq!(sort, SortDirection::Descending);
}

#[test]
fn test_resource_collection_with_pagination() {
    let mut resources = vec![];
    for i in 1..=25 {
        let resource = ResourceObject::new("users".to_string(), i.to_string())
            .set_attribute("name", json!(format!("User {}", i)));
        resources.push(resource);
    }

    assert_eq!(resources.len(), 25);
    assert_eq!(resources[0].id, Some("1".to_string()));
    assert_eq!(resources[24].id, Some("25".to_string()));
}

#[test]
fn test_empty_resource_collection() {
    let resources: Vec<ResourceObject> = vec![];
    assert_eq!(resources.len(), 0);
}

#[test]
fn test_multiple_filter_conditions() {
    let filters = vec![
        FilterCondition {
            field: "status".to_string(),
            operator: FilterOperator::Eq,
            value: Some("active".to_string()),
        },
        FilterCondition {
            field: "age".to_string(),
            operator: FilterOperator::Gte,
            value: Some("18".to_string()),
        },
        FilterCondition {
            field: "name".to_string(),
            operator: FilterOperator::Like,
            value: Some("%john%".to_string()),
        },
    ];

    assert_eq!(filters.len(), 3);
    assert_eq!(filters[0].operator, FilterOperator::Eq);
    assert_eq!(filters[1].operator, FilterOperator::Gte);
    assert_eq!(filters[2].operator, FilterOperator::Like);
}

#[test]
fn test_filter_with_null_and_not_null() {
    let filters = vec![
        FilterCondition {
            field: "deleted_at".to_string(),
            operator: FilterOperator::Null,
            value: None,
        },
        FilterCondition {
            field: "email".to_string(),
            operator: FilterOperator::NotNull,
            value: None,
        },
    ];

    assert!(filters[0].value.is_none());
    assert!(filters[1].value.is_none());
}

#[test]
fn test_pagination_with_filters_and_sorts() {
    let page_params = PageParams {
        number: 2,
        size: 10,
    };

    let filters = vec![
        FilterCondition {
            field: "role".to_string(),
            operator: FilterOperator::Eq,
            value: Some("admin".to_string()),
        },
        FilterCondition {
            field: "status".to_string(),
            operator: FilterOperator::In,
            value: Some("active,pending".to_string()),
        },
    ];

    let sorts = vec![
        ("created_at".to_string(), SortDirection::Descending),
        ("name".to_string(), SortDirection::Ascending),
    ];

    assert_eq!(page_params.number, 2);
    assert_eq!(filters.len(), 2);
    assert_eq!(sorts.len(), 2);
}

#[test]
fn test_pagination_metadata_consistency() {
    let scenarios = vec![
        (100, 1, 10),
        (100, 10, 10),
        (95, 10, 10),
        (0, 1, 10),
        (5, 1, 100),
    ];

    for (total, page, per_page) in scenarios {
        let meta = PaginationMeta::new(total, page, per_page);

        if total == 0 {
            assert_eq!(meta.from, 0);
            assert_eq!(meta.to, 0);
        } else {
            assert!(meta.from > 0);
            assert!(meta.to >= meta.from);
            assert!(meta.to <= total);
        }

        assert!(meta.last_page >= 1);
    }
}

#[test]
fn test_large_dataset_pagination() {
    let total = 1_000_000;
    let per_page = 100;
    let page = 5000;

    let meta = PaginationMeta::new(total, page, per_page);

    assert_eq!(meta.total, 1_000_000);
    assert_eq!(meta.last_page, 10_000);
    assert_eq!(meta.from, 499_901);
    assert_eq!(meta.to, 500_000);
}

#[test]
fn test_filter_field_name_variations() {
    let filters = vec![
        FilterCondition {
            field: "simple_field".to_string(),
            operator: FilterOperator::Eq,
            value: Some("value".to_string()),
        },
        FilterCondition {
            field: "camelCaseField".to_string(),
            operator: FilterOperator::Eq,
            value: Some("value".to_string()),
        },
        FilterCondition {
            field: "dot.notation.field".to_string(),
            operator: FilterOperator::Eq,
            value: Some("value".to_string()),
        },
        FilterCondition {
            field: "field_with_numbers_123".to_string(),
            operator: FilterOperator::Eq,
            value: Some("value".to_string()),
        },
    ];

    assert_eq!(filters.len(), 4);
    assert!(filters[2].field.contains('.'));
}

#[test]
fn test_pagination_boundary_conditions() {
    let meta = PaginationMeta::new(100, 0, 10);

    assert_eq!(meta.from, -9);
    assert_eq!(meta.to, 0);

    let meta = PaginationMeta::new(100, 1000, 10);
    assert!(meta.from > 0);
}

#[test]
fn test_resource_with_complex_id_formats() {
    let id_formats = vec![
        "123",
        "uuid-1234-5678",
        "composite:user:123",
        "user_456",
        "UPPER_CASE_789",
    ];

    for id in id_formats {
        let resource = ResourceObject::new("items".to_string(), id.to_string());
        assert_eq!(resource.id, Some(id.to_string()));
    }
}

#[test]
fn test_filter_value_encoding() {
    let filter = FilterCondition {
        field: "query".to_string(),
        operator: FilterOperator::Like,
        value: Some("search term with spaces & symbols!".to_string()),
    };

    assert!(filter.value.as_ref().unwrap().contains("&"));
    assert!(filter.value.as_ref().unwrap().contains(" "));
    assert!(filter.value.as_ref().unwrap().contains("!"));
}

#[test]
fn test_pagination_meta_clone() {
    let meta1 = PaginationMeta::new(100, 5, 20);
    let meta2 = meta1.clone();

    assert_eq!(meta1.total, meta2.total);
    assert_eq!(meta1.current_page, meta2.current_page);
    assert_eq!(meta1.per_page, meta2.per_page);
}

#[test]
fn test_filter_condition_clone() {
    let filter1 = FilterCondition {
        field: "test".to_string(),
        operator: FilterOperator::Eq,
        value: Some("value".to_string()),
    };
    let filter2 = filter1.clone();

    assert_eq!(filter1.field, filter2.field);
    assert_eq!(filter1.operator, filter2.operator);
    assert_eq!(filter1.value, filter2.value);
}

#[test]
fn test_pagination_serialization_roundtrip() {
    let meta = PaginationMeta::new(100, 3, 25);
    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: PaginationMeta = serde_json::from_str(&json).unwrap();

    assert_eq!(meta.total, deserialized.total);
    assert_eq!(meta.current_page, deserialized.current_page);
    assert_eq!(meta.per_page, deserialized.per_page);
    assert_eq!(meta.last_page, deserialized.last_page);
    assert_eq!(meta.from, deserialized.from);
    assert_eq!(meta.to, deserialized.to);
}
