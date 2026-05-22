use appkit_core::jsonapi::document::JsonApiDocument;
use appkit_core::jsonapi::error::{ErrorObject, ErrorSource};
use appkit_core::jsonapi::resource::ResourceObject;
use appkit_core::jsonapi::{FilterCondition, FilterOperator, PageParams, SortDirection};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_resource_with_empty_id() {
    let resource = ResourceObject::new("users".to_string(), "".to_string());
    assert_eq!(resource.id, Some("".to_string()));
    assert_eq!(resource.resource_type, "users");
}

#[test]
fn test_resource_with_special_chars_in_id() {
    let special_ids = vec![
        "user@123",
        "user#456",
        "user$789",
        "user%abc",
        "user&def",
        "user with spaces",
        "user/with/slashes",
    ];

    for id in special_ids {
        let resource = ResourceObject::new("users".to_string(), id.to_string());
        assert_eq!(resource.id, Some(id.to_string()));
    }
}

#[test]
fn test_resource_with_unicode_in_type_and_id() {
    let resource = ResourceObject::new("用户".to_string(), "用户123".to_string());
    assert_eq!(resource.resource_type, "用户");
    assert_eq!(resource.id, Some("用户123".to_string()));
}

#[test]
fn test_resource_with_many_attributes() {
    let mut resource = ResourceObject::new("users".to_string(), "1".to_string());

    for i in 0..100 {
        resource = resource.set_attribute(&format!("field_{}", i), json!(i));
    }

    let attrs = resource.attributes.as_ref().unwrap();
    assert_eq!(attrs.len(), 100);
    assert_eq!(attrs.get("field_50"), Some(&json!(50)));
}

#[test]
fn test_resource_attribute_with_null_value() {
    let resource = ResourceObject::new("users".to_string(), "1".to_string())
        .set_attribute("nullable_field", json!(null));

    let attrs = resource.attributes.as_ref().unwrap();
    assert_eq!(attrs.get("nullable_field"), Some(&json!(null)));
}

#[test]
fn test_resource_attribute_with_complex_nested_object() {
    let complex_value = json!({
        "nested": {
            "deeply": {
                "nested": {
                    "value": [1, 2, 3, {"key": "value"}]
                }
            }
        },
        "array": [null, true, false, "string", 123, {"obj": "in array"}]
    });

    let resource = ResourceObject::new("users".to_string(), "1".to_string())
        .set_attribute("complex", complex_value.clone());

    let attrs = resource.attributes.as_ref().unwrap();
    assert_eq!(attrs.get("complex"), Some(&complex_value));
}

#[test]
fn test_resource_without_attributes() {
    let resource = ResourceObject::new("users".to_string(), "1".to_string());

    assert!(resource.attributes.is_none() || resource.attributes.as_ref().unwrap().is_empty());
}

#[test]
fn test_empty_document_serialization() {
    let doc = JsonApiDocument::new();
    let json = serde_json::to_string(&doc).unwrap();

    assert!(!json.is_empty());
}

#[test]
fn test_document_with_empty_data_array() {
    let doc = JsonApiDocument::new().with_data(Vec::<ResourceObject>::new());
    let json = serde_json::to_string(&doc).unwrap();
    assert!(json.contains("\"data\":[]"));
}

#[test]
fn test_document_with_single_resource_as_array() {
    let resource = ResourceObject::new("users".to_string(), "1".to_string());
    let doc = JsonApiDocument::new().with_data(vec![resource]);
    let json = serde_json::to_string(&doc).unwrap();

    assert!(json.contains("\"data\":["));
    assert!(json.contains("\"type\":\"users\""));
    assert!(json.contains("\"id\":\"1\""));
}

#[test]
fn test_document_with_large_collection() {
    let resources: Vec<ResourceObject> = (0..1000)
        .map(|i| ResourceObject::new("users".to_string(), i.to_string()))
        .collect();

    let doc = JsonApiDocument::new().with_data(resources);
    let json = serde_json::to_string(&doc).unwrap();

    assert!(json.contains("\"data\":["));
    assert!(json.len() > 10000);
}

#[test]
fn test_document_with_meta_special_characters() {
    let doc = JsonApiDocument::new()
        .with_meta("key_with_<special>&chars", json!("value with \"quotes\""));

    let json = serde_json::to_string(&doc).unwrap();
    assert!(json.contains("key_with_<special>&chars"));
}

#[test]
fn test_document_with_multiple_meta_fields() {
    let doc = JsonApiDocument::new()
        .with_meta("total", json!(100))
        .with_meta("page", json!(1))
        .with_meta("per_page", json!(10))
        .with_meta("custom", json!({"nested": "value"}));

    let serialized = serde_json::to_string(&doc).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["meta"]["total"], 100);
    assert_eq!(parsed["meta"]["page"], 1);
    assert_eq!(parsed["meta"]["per_page"], 10);
    assert_eq!(parsed["meta"]["custom"]["nested"], "value");
}

#[test]
fn test_error_object_minimal() {
    let error = ErrorObject {
        id: None,
        links: None,
        status: Some("400".to_string()),
        code: None,
        title: None,
        detail: None,
        source: None,
        meta: None,
    };

    let doc = JsonApiDocument::new().with_errors(vec![error]);
    let json = serde_json::to_string(&doc).unwrap();

    assert!(json.contains("\"errors\":["));
    assert!(json.contains("\"status\":\"400\""));
}

#[test]
fn test_error_object_complete() {
    let error = ErrorObject {
        id: Some("err_123".to_string()),
        links: None,
        status: Some("422".to_string()),
        code: Some("validation_failed".to_string()),
        title: Some("Validation Failed".to_string()),
        detail: Some("The email field is required".to_string()),
        source: Some(ErrorSource {
            pointer: Some("/data/attributes/email".to_string()),
            parameter: None,
            header: None,
        }),
        meta: None,
    };

    let doc = JsonApiDocument::new().with_errors(vec![error]);
    let json = serde_json::to_string(&doc).unwrap();

    assert!(json.contains("\"id\":\"err_123\""));
    assert!(json.contains("\"status\":\"422\""));
    assert!(json.contains("\"code\":\"validation_failed\""));
    assert!(json.contains("\"title\":\"Validation Failed\""));
    assert!(json.contains("\"detail\":\"The email field is required\""));
    assert!(json.contains("\"pointer\":\"/data/attributes/email\""));
}

#[test]
fn test_multiple_errors() {
    let errors = vec![
        ErrorObject {
            id: Some("err_1".to_string()),
            links: None,
            status: Some("422".to_string()),
            code: Some("required".to_string()),
            title: Some("Field Required".to_string()),
            detail: Some("Email is required".to_string()),
            source: Some(ErrorSource {
                pointer: Some("/data/attributes/email".to_string()),
                parameter: None,
                header: None,
            }),
            meta: None,
        },
        ErrorObject {
            id: Some("err_2".to_string()),
            links: None,
            status: Some("422".to_string()),
            code: Some("too_short".to_string()),
            title: Some("Value Too Short".to_string()),
            detail: Some("Password must be at least 8 characters".to_string()),
            source: Some(ErrorSource {
                pointer: Some("/data/attributes/password".to_string()),
                parameter: None,
                header: None,
            }),
            meta: None,
        },
    ];

    let doc = JsonApiDocument::new().with_errors(errors);
    let serialized = serde_json::to_string(&doc).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["errors"].as_array().unwrap().len(), 2);
}

#[test]
fn test_error_source_with_parameter() {
    let error = ErrorObject {
        id: None,
        links: None,
        status: Some("400".to_string()),
        code: Some("invalid_parameter".to_string()),
        title: Some("Invalid Query Parameter".to_string()),
        detail: Some("Invalid filter parameter".to_string()),
        source: Some(ErrorSource {
            pointer: None,
            parameter: Some("filter[email]".to_string()),
            header: None,
        }),
        meta: None,
    };

    let doc = JsonApiDocument::new().with_errors(vec![error]);
    let json = serde_json::to_string(&doc).unwrap();

    assert!(json.contains("\"parameter\":\"filter[email]\""));
}

#[test]
fn test_error_with_meta() {
    let mut meta = HashMap::new();
    meta.insert("timestamp".to_string(), json!("2024-01-01T00:00:00Z"));
    meta.insert("request_id".to_string(), json!("req_abc123"));

    let error = ErrorObject {
        id: None,
        links: None,
        status: Some("500".to_string()),
        code: Some("internal_error".to_string()),
        title: Some("Internal Server Error".to_string()),
        detail: None,
        source: None,
        meta: Some(meta),
    };

    let doc = JsonApiDocument::new().with_errors(vec![error]);
    let serialized = serde_json::to_string(&doc).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["errors"][0]["meta"]["request_id"], "req_abc123");
}

#[test]
fn test_document_with_included_resources() {
    let user = ResourceObject::new("users".to_string(), "1".to_string())
        .set_attribute("name", json!("John Doe"));

    let org = ResourceObject::new("organizations".to_string(), "42".to_string())
        .set_attribute("name", json!("ACME Corp"));

    let doc = JsonApiDocument::new()
        .with_data(vec![user])
        .with_included(vec![org]);

    let json = serde_json::to_string(&doc).unwrap();

    assert!(json.contains("\"included\":["));
    assert!(json.contains("\"type\":\"organizations\""));
    assert!(json.contains("\"id\":\"42\""));
}

#[test]
fn test_document_with_empty_included() {
    let user = ResourceObject::new("users".to_string(), "1".to_string());
    let doc = JsonApiDocument::new()
        .with_data(vec![user])
        .with_included(Vec::<ResourceObject>::new());

    let json = serde_json::to_string(&doc).unwrap();
    assert!(json.contains("\"included\":[]"));
}

#[test]
fn test_document_with_many_included_resources() {
    let user = ResourceObject::new("users".to_string(), "1".to_string());

    let included: Vec<ResourceObject> = (0..50)
        .map(|i| {
            ResourceObject::new("tags".to_string(), i.to_string())
                .set_attribute("name", json!(format!("Tag {}", i)))
        })
        .collect();

    let doc = JsonApiDocument::new()
        .with_data(vec![user])
        .with_included(included);

    let serialized = serde_json::to_string(&doc).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    assert_eq!(parsed["included"].as_array().unwrap().len(), 50);
}

#[test]
fn test_filter_condition_equals() {
    let filter = FilterCondition {
        field: "email".to_string(),
        operator: FilterOperator::Eq,
        value: Some("test@example.com".to_string()),
    };

    assert_eq!(filter.field, "email");
    assert_eq!(filter.value, Some("test@example.com".to_string()));
}

#[test]
fn test_filter_condition_with_special_characters() {
    let filter = FilterCondition {
        field: "email".to_string(),
        operator: FilterOperator::Eq,
        value: Some("user+test@example.com".to_string()),
    };

    assert_eq!(filter.value, Some("user+test@example.com".to_string()));
}

#[test]
fn test_filter_condition_with_sql_injection_attempt() {
    let dangerous_value = "' OR '1'='1";
    let filter = FilterCondition {
        field: "username".to_string(),
        operator: FilterOperator::Eq,
        value: Some(dangerous_value.to_string()),
    };

    assert_eq!(filter.value, Some(dangerous_value.to_string()));
}

#[test]
fn test_filter_condition_with_unicode() {
    let filter = FilterCondition {
        field: "name".to_string(),
        operator: FilterOperator::Like,
        value: Some("José García 日本語".to_string()),
    };

    assert_eq!(filter.value, Some("José García 日本語".to_string()));
}

#[test]
fn test_filter_condition_null_value() {
    let filter = FilterCondition {
        field: "deleted_at".to_string(),
        operator: FilterOperator::Eq,
        value: None,
    };

    assert!(filter.value.is_none());
}

#[test]
fn test_filter_operators() {
    let _eq = FilterOperator::Eq;
    let _neq = FilterOperator::Neq;
    let _gt = FilterOperator::Gt;
    let _gte = FilterOperator::Gte;
    let _lt = FilterOperator::Lt;
    let _lte = FilterOperator::Lte;
    let _like = FilterOperator::Like;
}

#[test]
fn test_page_params_defaults() {
    let page = PageParams::default();

    assert!(page.number > 0);
    assert!(page.size > 0);
}

#[test]
fn test_page_params_custom_values() {
    let page = PageParams {
        number: 5,
        size: 50,
    };

    assert_eq!(page.number, 5);
    assert_eq!(page.size, 50);
}

#[test]
fn test_page_params_large_page_numbers() {
    let page = PageParams {
        number: 1000000,
        size: 100,
    };

    assert_eq!(page.number, 1000000);
    assert_eq!(page.size, 100);
}

#[test]
fn test_sort_direction_ascending() {
    let _dir = SortDirection::Ascending;
}

#[test]
fn test_sort_direction_descending() {
    let _dir = SortDirection::Descending;
}

#[test]
fn test_document_serialization_roundtrip() {
    let original = ResourceObject::new("users".to_string(), "1".to_string())
        .set_attribute("email", json!("test@example.com"))
        .set_attribute("name", json!("Test User"));

    let doc = JsonApiDocument::new().with_data(vec![original.clone()]);

    let json = serde_json::to_string(&doc).unwrap();
    let parsed: JsonApiDocument = serde_json::from_str(&json).unwrap();

    assert!(parsed.data.is_some());
}

#[test]
fn test_error_document_serialization_roundtrip() {
    let error = ErrorObject {
        id: Some("err_123".to_string()),
        links: None,
        status: Some("404".to_string()),
        code: Some("not_found".to_string()),
        title: Some("Not Found".to_string()),
        detail: Some("Resource not found".to_string()),
        source: None,
        meta: None,
    };

    let doc = JsonApiDocument::new().with_errors(vec![error]);

    let json = serde_json::to_string(&doc).unwrap();
    let parsed: JsonApiDocument = serde_json::from_str(&json).unwrap();

    assert!(parsed.errors.is_some());
    assert_eq!(parsed.errors.as_ref().unwrap().len(), 1);
}

#[test]
fn test_resource_with_unicode_roundtrip() {
    let original = ResourceObject::new("users".to_string(), "123".to_string())
        .set_attribute("name", json!("José García"))
        .set_attribute("bio", json!("Hello 世界 🌍"));

    let doc = JsonApiDocument::new().with_data(vec![original]);

    let json = serde_json::to_string(&doc).unwrap();
    let parsed: JsonApiDocument = serde_json::from_str(&json).unwrap();

    assert!(parsed.data.is_some());
}
