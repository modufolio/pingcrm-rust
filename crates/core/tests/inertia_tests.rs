use appkit_core::inertia::{
    AccountData, InertiaRequest, InertiaResponse, InertiaVersion, RequestType, SharedProps,
    UserData,
};
use axum::http::{HeaderMap, HeaderValue};
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_shared_props_new() {
    let props = SharedProps::new();

    assert!(props.auth.user.is_none());
    assert!(props.auth.impersonation.is_none());
    assert!(props.flash.success.is_none());
    assert!(props.flash.error.is_none());
    assert!(props.errors.is_empty());
}

#[test]
fn test_shared_props_default() {
    let props = SharedProps::default();

    assert!(props.auth.user.is_none());
    assert!(props.auth.impersonation.is_none());
    assert!(props.flash.success.is_none());
    assert!(props.flash.error.is_none());
    assert!(props.errors.is_empty());
}

#[test]
fn test_shared_props_with_user() {
    let user = UserData {
        id: "123".to_string(),
        email: "test@example.com".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        account: AccountData {
            id: 1,
            name: "Test Account".to_string(),
        },
        role: Some("admin".to_string()),
        two_factor_enabled: false,
    };

    let props = SharedProps::new().with_user(user.clone());

    assert!(props.auth.user.is_some());
    let stored_user = props.auth.user.unwrap();
    assert_eq!(stored_user.id, "123");
    assert_eq!(stored_user.email, "test@example.com");
    assert_eq!(stored_user.first_name, "Test");
    assert_eq!(stored_user.last_name, "User");
    assert_eq!(stored_user.account.id, 1);
    assert_eq!(stored_user.account.name, "Test Account");
    assert_eq!(stored_user.role, Some("admin".to_string()));
    assert!(!stored_user.two_factor_enabled);
}

#[test]
fn test_shared_props_with_flash_success() {
    let props = SharedProps::new().with_success("Operation successful!".to_string());

    assert_eq!(
        props.flash.success,
        Some("Operation successful!".to_string())
    );
    assert!(props.flash.error.is_none());
}

#[test]
fn test_shared_props_with_flash_error() {
    let props = SharedProps::new().with_error("Operation failed!".to_string());

    assert!(props.flash.success.is_none());
    assert_eq!(props.flash.error, Some("Operation failed!".to_string()));
}

#[test]
fn test_shared_props_with_field_error() {
    let props =
        SharedProps::new().with_field_error("email".to_string(), "Invalid email".to_string());

    assert_eq!(props.errors.len(), 1);
    assert_eq!(
        props.errors.get("email"),
        Some(&"Invalid email".to_string())
    );
}

#[test]
fn test_shared_props_with_multiple_errors() {
    let mut errors = HashMap::new();
    errors.insert("email".to_string(), "Invalid email".to_string());
    errors.insert("password".to_string(), "Too short".to_string());

    let props = SharedProps::new().with_errors(errors.clone());

    assert_eq!(props.errors.len(), 2);
    assert_eq!(
        props.errors.get("email"),
        Some(&"Invalid email".to_string())
    );
    assert_eq!(props.errors.get("password"), Some(&"Too short".to_string()));
}

#[test]
fn test_shared_props_chaining() {
    let user = UserData {
        id: "123".to_string(),
        email: "test@example.com".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        account: AccountData {
            id: 1,
            name: "Test Account".to_string(),
        },
        role: Some("admin".to_string()),
        two_factor_enabled: true,
    };

    let props = SharedProps::new()
        .with_user(user)
        .with_success("Saved!".to_string())
        .with_field_error("name".to_string(), "Required".to_string());

    assert!(props.auth.user.is_some());
    assert_eq!(props.flash.success, Some("Saved!".to_string()));
    assert_eq!(props.errors.get("name"), Some(&"Required".to_string()));
}

#[test]
fn test_shared_props_to_value() {
    let props = SharedProps::new().with_success("Success!".to_string());

    let value = props.to_value();

    assert!(value.is_object());
    assert!(value["auth"].is_object());
    assert!(value["flash"].is_object());
    assert!(value["errors"].is_object());
    assert_eq!(value["flash"]["success"], json!("Success!"));
}

#[test]
fn test_shared_props_merge_with() {
    let props = SharedProps::new().with_success("Merged!".to_string());

    let custom_props = json!({
        "users": ["Alice", "Bob"],
        "count": 42
    });

    let merged = props.merge_with(custom_props);

    assert!(merged["auth"].is_object());
    assert!(merged["flash"].is_object());
    assert!(merged["errors"].is_object());
    assert_eq!(merged["users"], json!(["Alice", "Bob"]));
    assert_eq!(merged["count"], json!(42));
    assert_eq!(merged["flash"]["success"], json!("Merged!"));
}

#[test]
fn test_user_data_serialization() {
    let user = UserData {
        id: "456".to_string(),
        email: "alice@example.com".to_string(),
        first_name: "Alice".to_string(),
        last_name: "Smith".to_string(),
        account: AccountData {
            id: 2,
            name: "Alice's Account".to_string(),
        },
        role: Some("user".to_string()),
        two_factor_enabled: true,
    };

    let json = serde_json::to_value(&user).unwrap();

    assert_eq!(json["id"], "456");
    assert_eq!(json["email"], "alice@example.com");
    assert_eq!(json["first_name"], "Alice");
    assert_eq!(json["last_name"], "Smith");
    assert_eq!(json["account"]["id"], 2);
    assert_eq!(json["account"]["name"], "Alice's Account");
    assert_eq!(json["role"], "user");
    assert_eq!(json["two_factor_enabled"], true);
}

#[test]
fn test_account_data_default() {
    let account = AccountData::default();

    assert_eq!(account.id, 0);
    assert_eq!(account.name, "");
}

#[test]
fn test_shared_props_with_impersonation_serialization() {
    let json_with_impersonation = json!({
        "auth": {
            "user": {
                "id": "123",
                "email": "test@example.com",
                "first_name": "Test",
                "last_name": "User",
                "account": {
                    "id": 1,
                    "name": "Test Account"
                },
                "role": "user",
                "two_factor_enabled": false
            },
            "impersonation": {
                "is_impersonating": true,
                "original_user": {
                    "id": "999",
                    "email": "admin@example.com",
                    "first_name": "Admin",
                    "last_name": "User"
                }
            }
        },
        "flash": {
            "success": null,
            "error": null
        },
        "errors": {}
    });

    let props: SharedProps = serde_json::from_value(json_with_impersonation).unwrap();

    assert!(props.auth.user.is_some());
    assert!(props.auth.impersonation.is_some());

    let imp = props.auth.impersonation.unwrap();
    assert!(imp.is_impersonating);
    assert_eq!(imp.original_user.id, "999");
    assert_eq!(imp.original_user.email, "admin@example.com");
}

#[test]
fn test_inertia_version_new() {
    let version = InertiaVersion::new("abc123");
    assert_eq!(version.as_str(), "abc123");
}

#[test]
fn test_inertia_version_default() {
    let version = InertiaVersion::default();
    assert_eq!(version.as_str(), "1");
}

#[test]
fn test_inertia_response_new() {
    let response = InertiaResponse::new("Users/Index");

    let response = response.with_props(json!({"test": true}));
    let _ = response;
}

#[test]
fn test_inertia_response_with_props() {
    let props = json!({
        "users": ["Alice", "Bob"],
        "count": 2
    });

    let response = InertiaResponse::new("Users/Index").with_props(props.clone());
    let _ = response;
}

#[test]
fn test_inertia_response_with_version() {
    let version = InertiaVersion::new("custom-version");
    let response = InertiaResponse::new("Users/Index").with_version(version);
    let _ = response;
}

#[test]
fn test_inertia_response_with_url() {
    let response = InertiaResponse::new("Users/Index").with_url("/custom/url".to_string());
    let _ = response;
}

#[test]
fn test_inertia_response_builder_chaining() {
    let response = InertiaResponse::new("Users/Index")
        .with_props(json!({"data": "test"}))
        .with_version(InertiaVersion::new("v2"))
        .with_url("/users".to_string());

    let _ = response;
}

#[test]
fn test_inertia_request_from_headers_basic() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Inertia", HeaderValue::from_static("true"));

    let req = InertiaRequest::from_headers(&headers);

    assert!(req.is_inertia());
    assert!(!req.is_partial());
}

#[test]
fn test_inertia_request_with_version() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Inertia", HeaderValue::from_static("true"));
    headers.insert("X-Inertia-Version", HeaderValue::from_static("abc123"));

    let req = InertiaRequest::from_headers(&headers);

    assert!(req.is_inertia());
    assert_eq!(req.version(), Some("abc123"));
}

#[test]
fn test_inertia_request_partial_data() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Inertia", HeaderValue::from_static("true"));
    headers.insert(
        "X-Inertia-Partial-Data",
        HeaderValue::from_static("users,posts,comments"),
    );

    let req = InertiaRequest::from_headers(&headers);

    assert!(req.is_partial());
    assert!(req.should_include_prop("users"));
    assert!(req.should_include_prop("posts"));
    assert!(req.should_include_prop("comments"));
    assert!(!req.should_include_prop("other"));
}

#[test]
fn test_inertia_request_partial_component() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Inertia", HeaderValue::from_static("true"));
    headers.insert(
        "X-Inertia-Partial-Component",
        HeaderValue::from_static("Users/Index"),
    );

    let req = InertiaRequest::from_headers(&headers);

    assert!(req.is_inertia());
    assert_eq!(req.partial_component, Some("Users/Index".to_string()));
}

#[test]
fn test_inertia_request_not_inertia() {
    let headers = HeaderMap::new();
    let req = InertiaRequest::from_headers(&headers);

    assert!(!req.is_inertia());
    assert!(!req.is_partial());
    assert_eq!(req.version(), None);
}

#[test]
fn test_inertia_request_default() {
    let req = InertiaRequest::default();

    assert!(!req.is_inertia());
    assert!(!req.is_partial());
    assert_eq!(req.version(), None);
}

#[test]
fn test_inertia_request_should_include_all_props_when_not_partial() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Inertia", HeaderValue::from_static("true"));

    let req = InertiaRequest::from_headers(&headers);

    assert!(req.should_include_prop("any_key"));
    assert!(req.should_include_prop("another_key"));
}

#[test]
fn test_request_type_inertia() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Inertia", HeaderValue::from_static("true"));

    let req_type = RequestType::detect(&headers, "/users");

    assert_eq!(req_type, RequestType::Inertia);
    assert!(req_type.is_inertia());
    assert!(!req_type.is_api());
    assert!(!req_type.is_web());
}

#[test]
fn test_request_type_json_api_by_path() {
    let headers = HeaderMap::new();

    let req_type = RequestType::detect(&headers, "/api/users");

    assert_eq!(req_type, RequestType::JsonApi);
    assert!(req_type.is_api());
    assert!(!req_type.is_inertia());
    assert!(!req_type.is_web());
}

#[test]
fn test_request_type_json_api_by_header() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Accept",
        HeaderValue::from_static("application/vnd.api+json"),
    );

    let req_type = RequestType::detect(&headers, "/users");

    assert_eq!(req_type, RequestType::JsonApi);
    assert!(req_type.is_api());
}

#[test]
fn test_request_type_web() {
    let headers = HeaderMap::new();

    let req_type = RequestType::detect(&headers, "/users");

    assert_eq!(req_type, RequestType::Web);
    assert!(req_type.is_web());
    assert!(!req_type.is_api());
    assert!(!req_type.is_inertia());
}

#[test]
fn test_request_type_inertia_takes_precedence() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Inertia", HeaderValue::from_static("true"));
    headers.insert(
        "Accept",
        HeaderValue::from_static("application/vnd.api+json"),
    );

    let req_type = RequestType::detect(&headers, "/api/users");

    assert_eq!(req_type, RequestType::Inertia);
}

#[test]
fn test_request_type_api_path_variants() {
    let headers = HeaderMap::new();

    assert!(RequestType::detect(&headers, "/api/v1/users").is_api());
    assert!(RequestType::detect(&headers, "/api/v2/posts").is_api());
    assert!(RequestType::detect(&headers, "/api/contacts").is_api());
}

#[test]
fn test_request_type_non_api_paths() {
    let headers = HeaderMap::new();

    assert!(RequestType::detect(&headers, "/users").is_web());
    assert!(RequestType::detect(&headers, "/dashboard").is_web());
    assert!(RequestType::detect(&headers, "/").is_web());
    assert!(RequestType::detect(&headers, "/about").is_web());
}

#[test]
fn test_full_inertia_flow_with_props() {
    let user = UserData {
        id: "100".to_string(),
        email: "integration@example.com".to_string(),
        first_name: "Integration".to_string(),
        last_name: "Test".to_string(),
        account: AccountData {
            id: 5,
            name: "Test Account".to_string(),
        },
        role: Some("admin".to_string()),
        two_factor_enabled: false,
    };

    let shared = SharedProps::new()
        .with_user(user)
        .with_success("Welcome!".to_string());

    let custom = json!({"items": ["a", "b", "c"]});

    let props = shared.merge_with(custom);

    let response = InertiaResponse::new("Dashboard")
        .with_props(props)
        .with_version(InertiaVersion::new("test-version"));

    let _ = response;
}

#[test]
fn test_partial_reload_filtering() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Inertia", HeaderValue::from_static("true"));
    headers.insert(
        "X-Inertia-Partial-Data",
        HeaderValue::from_static("users,count"),
    );

    let req = InertiaRequest::from_headers(&headers);

    assert!(req.should_include_prop("users"));
    assert!(req.should_include_prop("count"));

    assert!(!req.should_include_prop("posts"));
    assert!(!req.should_include_prop("settings"));
    assert!(!req.should_include_prop("flash"));
}

#[test]
fn test_flash_messages_with_unicode() {
    let props = SharedProps::new()
        .with_success("¡Éxito! 成功！ نجاح".to_string())
        .with_error("שְׁגִיאָה Σφάλμα 错误".to_string());

    assert!(props.flash.success.is_some());
    assert!(props.flash.error.is_some());
    assert!(props.flash.success.as_ref().unwrap().contains("成功"));
    assert!(props.flash.error.as_ref().unwrap().contains("错误"));
}

#[test]
fn test_validation_errors_with_nested_fields() {
    let mut errors = HashMap::new();
    errors.insert("user.email".to_string(), "Invalid".to_string());
    errors.insert("user.profile.age".to_string(), "Too young".to_string());
    errors.insert("address.0.street".to_string(), "Required".to_string());

    let props = SharedProps::new().with_errors(errors);

    assert_eq!(props.errors.len(), 3);
    assert!(props.errors.contains_key("user.email"));
    assert!(props.errors.contains_key("user.profile.age"));
    assert!(props.errors.contains_key("address.0.street"));
}

#[test]
fn test_props_serialization_roundtrip() {
    let user = UserData {
        id: "roundtrip-id".to_string(),
        email: "roundtrip@example.com".to_string(),
        first_name: "Round".to_string(),
        last_name: "Trip".to_string(),
        account: AccountData {
            id: 99,
            name: "Roundtrip Account".to_string(),
        },
        role: Some("tester".to_string()),
        two_factor_enabled: true,
    };

    let original = SharedProps::new()
        .with_user(user)
        .with_success("Test!".to_string());

    let json = serde_json::to_string(&original).unwrap();

    let deserialized: SharedProps = serde_json::from_str(&json).unwrap();

    assert!(deserialized.auth.user.is_some());
    assert_eq!(deserialized.flash.success, Some("Test!".to_string()));

    let user = deserialized.auth.user.unwrap();
    assert_eq!(user.id, "roundtrip-id");
    assert_eq!(user.email, "roundtrip@example.com");
    assert_eq!(user.account.id, 99);
}

#[test]
fn test_empty_props_serialization() {
    let props = SharedProps::new();
    let json = serde_json::to_value(&props).unwrap();

    assert!(json["auth"]["user"].is_null());
    assert!(json["auth"]["impersonation"].is_null());
    assert!(json["flash"]["success"].is_null());
    assert!(json["flash"]["error"].is_null());
    assert!(json["errors"].is_object());
    assert_eq!(json["errors"].as_object().unwrap().len(), 0);
}

#[test]
fn test_large_props_object() {
    let mut errors = HashMap::new();
    for i in 0..100 {
        errors.insert(format!("field_{}", i), format!("Error {}", i));
    }

    let props = SharedProps::new().with_errors(errors);

    assert_eq!(props.errors.len(), 100);

    let json = serde_json::to_value(&props).unwrap();
    assert_eq!(json["errors"].as_object().unwrap().len(), 100);
}

#[test]
fn test_user_without_role() {
    let user = UserData {
        id: "no-role".to_string(),
        email: "norole@example.com".to_string(),
        first_name: "No".to_string(),
        last_name: "Role".to_string(),
        account: AccountData::default(),
        role: None,
        two_factor_enabled: false,
    };

    let props = SharedProps::new().with_user(user);
    let json = serde_json::to_value(&props).unwrap();

    assert!(json["auth"]["user"]["role"].is_null());
}

#[test]
fn test_inertia_request_with_whitespace_in_partial_data() {
    let mut headers = HeaderMap::new();
    headers.insert("X-Inertia", HeaderValue::from_static("true"));
    headers.insert(
        "X-Inertia-Partial-Data",
        HeaderValue::from_static("  users  ,  posts  ,  comments  "),
    );

    let req = InertiaRequest::from_headers(&headers);

    assert!(req.should_include_prop("users"));
    assert!(req.should_include_prop("posts"));
    assert!(req.should_include_prop("comments"));
}

#[test]
fn test_complex_url_handling() {
    let response =
        InertiaResponse::new("Search").with_url("/search?q=test&page=2&sort=asc".to_string());

    let _ = response;
}
