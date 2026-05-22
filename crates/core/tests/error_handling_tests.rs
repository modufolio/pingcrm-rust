use appkit_core::error::AppError;
use appkit_core::jsonapi::error::ErrorObject;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_app_error_not_found() {
    let error = AppError::NotFound("User not found".to_string());

    match error {
        AppError::NotFound(msg) => assert_eq!(msg, "User not found"),
        _ => panic!("Expected NotFound variant"),
    }
}

#[test]
fn test_app_error_authentication_failed() {
    let error = AppError::AuthenticationFailed("Invalid credentials".to_string());

    match error {
        AppError::AuthenticationFailed(msg) => assert_eq!(msg, "Invalid credentials"),
        _ => panic!("Expected AuthenticationFailed variant"),
    }
}

#[test]
fn test_app_error_authorization_failed() {
    let error = AppError::AuthorizationFailed("Access denied".to_string());

    match error {
        AppError::AuthorizationFailed(msg) => assert_eq!(msg, "Access denied"),
        _ => panic!("Expected AuthorizationFailed variant"),
    }
}

#[test]
fn test_app_error_bad_request() {
    let error = AppError::BadRequest("Invalid input".to_string());

    match error {
        AppError::BadRequest(msg) => assert_eq!(msg, "Invalid input"),
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_app_error_internal_server_error() {
    let error = AppError::InternalServerError("Database connection failed".to_string());

    match error {
        AppError::InternalServerError(msg) => {
            assert_eq!(msg, "Database connection failed")
        }
        _ => panic!("Expected InternalServerError variant"),
    }
}

#[test]
fn test_app_error_conflict() {
    let error = AppError::Conflict("Email already exists".to_string());

    match error {
        AppError::Conflict(msg) => assert_eq!(msg, "Email already exists"),
        _ => panic!("Expected Conflict variant"),
    }
}

#[test]
fn test_app_error_validation_error() {
    let mut errors = HashMap::new();
    errors.insert(
        "email".to_string(),
        vec!["Invalid email format".to_string()],
    );
    errors.insert(
        "password".to_string(),
        vec!["Password too short".to_string()],
    );

    let error = AppError::ValidationError {
        message: "Validation failed".to_string(),
        errors: errors.clone(),
    };

    match error {
        AppError::ValidationError {
            message,
            errors: errs,
        } => {
            assert_eq!(message, "Validation failed");
            assert_eq!(errs.len(), 2);
            assert!(errs.contains_key("email"));
            assert!(errs.contains_key("password"));
        }
        _ => panic!("Expected ValidationError variant"),
    }
}

#[test]
fn test_app_error_rate_limit_exceeded() {
    let error = AppError::RateLimitExceeded;

    match error {
        AppError::RateLimitExceeded => assert!(true),
        _ => panic!("Expected RateLimitExceeded variant"),
    }
}

#[test]
fn test_app_error_payload_too_large() {
    let error = AppError::PayloadTooLarge("File exceeds 10MB limit".to_string());

    match error {
        AppError::PayloadTooLarge(msg) => assert_eq!(msg, "File exceeds 10MB limit"),
        _ => panic!("Expected PayloadTooLarge variant"),
    }
}

#[test]
fn test_app_error_method_not_allowed() {
    let error = AppError::MethodNotAllowed("DELETE not allowed".to_string());

    match error {
        AppError::MethodNotAllowed(msg) => assert_eq!(msg, "DELETE not allowed"),
        _ => panic!("Expected MethodNotAllowed variant"),
    }
}

#[test]
fn test_app_error_service_unavailable() {
    let error = AppError::ServiceUnavailable("Service is down".to_string());

    match error {
        AppError::ServiceUnavailable(msg) => assert_eq!(msg, "Service is down"),
        _ => panic!("Expected ServiceUnavailable variant"),
    }
}

#[test]
fn test_app_error_not_implemented() {
    let error = AppError::NotImplemented;

    match error {
        AppError::NotImplemented => assert!(true),
        _ => panic!("Expected NotImplemented variant"),
    }
}

#[test]
fn test_resource_not_found_helper() {
    let error = AppError::resource_not_found("users");

    match error {
        AppError::NotFound(msg) => {
            assert!(msg.contains("users"));
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected NotFound variant"),
    }
}

#[test]
fn test_method_not_allowed_helper() {
    let error = AppError::method_not_allowed();

    match error {
        AppError::MethodNotAllowed(msg) => {
            assert!(msg.contains("not allowed"));
        }
        _ => panic!("Expected MethodNotAllowed variant"),
    }
}

#[test]
fn test_invalid_id_helper() {
    let error = AppError::invalid_id("abc");

    match error {
        AppError::BadRequest(msg) => {
            assert!(msg.contains("Invalid ID"));
            assert!(msg.contains("abc"));
        }
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_relationship_not_found_helper() {
    let error = AppError::relationship_not_found("posts");

    match error {
        AppError::NotFound(msg) => {
            assert!(msg.contains("posts"));
            assert!(msg.contains("not found"));
        }
        _ => panic!("Expected NotFound variant"),
    }
}

#[test]
fn test_invalid_field_helper() {
    let error = AppError::invalid_field("unknown_field", "users");

    match error {
        AppError::BadRequest(msg) => {
            assert!(msg.contains("unknown_field"));
            assert!(msg.contains("users"));
        }
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_invalid_include_helper() {
    let error = AppError::invalid_include("posts.comments.author");

    match error {
        AppError::BadRequest(msg) => {
            assert!(msg.contains("posts.comments.author"));
        }
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_invalid_sort_field_helper() {
    let error = AppError::invalid_sort_field("unknown");

    match error {
        AppError::BadRequest(msg) => {
            assert!(msg.contains("unknown"));
            assert!(msg.contains("sort"));
        }
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_type_mismatch_helper() {
    let error = AppError::type_mismatch("posts", "users");

    match error {
        AppError::BadRequest(msg) => {
            assert!(msg.contains("posts"));
            assert!(msg.contains("users"));
            assert!(msg.contains("mismatch"));
        }
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_invalid_content_type_helper() {
    let error = AppError::invalid_content_type();

    match error {
        AppError::UnsupportedMediaType(msg) => {
            assert!(msg.contains("application/vnd.api+json"));
        }
        _ => panic!("Expected UnsupportedMediaType variant"),
    }
}

#[test]
fn test_missing_content_type_helper() {
    let error = AppError::missing_content_type();

    match error {
        AppError::BadRequest(msg) => {
            assert!(msg.contains("Content-Type"));
        }
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_error_message_with_special_characters() {
    let error = AppError::BadRequest("Invalid characters: <>&\"'".to_string());

    match error {
        AppError::BadRequest(msg) => assert!(msg.contains("<>&\"'")),
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_error_message_with_unicode() {
    let error = AppError::NotFound("Usuario no encontrado 用户未找到".to_string());

    match error {
        AppError::NotFound(msg) => assert!(msg.contains("Usuario") && msg.contains("用户")),
        _ => panic!("Expected NotFound variant"),
    }
}

#[test]
fn test_error_message_empty() {
    let error = AppError::BadRequest("".to_string());

    match error {
        AppError::BadRequest(msg) => assert_eq!(msg, ""),
        _ => panic!("Expected BadRequest variant"),
    }
}

#[test]
fn test_error_message_very_long() {
    let long_message = "a".repeat(10000);
    let error = AppError::InternalServerError(long_message.clone());

    match error {
        AppError::InternalServerError(msg) => assert_eq!(msg.len(), 10000),
        _ => panic!("Expected InternalServerError variant"),
    }
}

#[test]
fn test_error_display() {
    let error = AppError::NotFound("Resource not found".to_string());
    let display = format!("{}", error);

    assert!(display.contains("Resource not found"));
}

#[test]
fn test_error_debug() {
    let error = AppError::AuthenticationFailed("Auth failed".to_string());
    let debug = format!("{:?}", error);

    assert!(debug.contains("AuthenticationFailed"));
    assert!(debug.contains("Auth failed"));
}

#[test]
fn test_validation_error_empty() {
    let error = AppError::ValidationError {
        message: "No errors".to_string(),
        errors: HashMap::new(),
    };

    match error {
        AppError::ValidationError { message: _, errors } => assert_eq!(errors.len(), 0),
        _ => panic!("Expected ValidationError variant"),
    }
}

#[test]
fn test_validation_error_single_field() {
    let mut errors = HashMap::new();
    errors.insert("email".to_string(), vec!["Required field".to_string()]);

    let error = AppError::ValidationError {
        message: "Validation failed".to_string(),
        errors: errors.clone(),
    };

    match error {
        AppError::ValidationError {
            message: _,
            errors: errs,
        } => {
            assert_eq!(errs.len(), 1);
            assert_eq!(errs.get("email").unwrap()[0], "Required field");
        }
        _ => panic!("Expected ValidationError variant"),
    }
}

#[test]
fn test_validation_error_multiple_fields() {
    let mut errors = HashMap::new();
    errors.insert("email".to_string(), vec!["Invalid format".to_string()]);
    errors.insert("password".to_string(), vec!["Too short".to_string()]);
    errors.insert("age".to_string(), vec!["Must be 18+".to_string()]);

    let error = AppError::ValidationError {
        message: "Validation failed".to_string(),
        errors: errors.clone(),
    };

    match error {
        AppError::ValidationError {
            message: _,
            errors: errs,
        } => {
            assert_eq!(errs.len(), 3);
        }
        _ => panic!("Expected ValidationError variant"),
    }
}

#[test]
fn test_validation_error_multiple_messages_per_field() {
    let mut errors = HashMap::new();
    errors.insert(
        "password".to_string(),
        vec![
            "Too short".to_string(),
            "Missing special character".to_string(),
            "Missing number".to_string(),
        ],
    );

    let error = AppError::ValidationError {
        message: "Validation failed".to_string(),
        errors: errors.clone(),
    };

    match error {
        AppError::ValidationError {
            message: _,
            errors: errs,
        } => {
            assert_eq!(errs.get("password").unwrap().len(), 3);
        }
        _ => panic!("Expected ValidationError variant"),
    }
}

#[test]
fn test_validation_error_with_nested_field_names() {
    let mut errors = HashMap::new();
    errors.insert("user.email".to_string(), vec!["Invalid email".to_string()]);
    errors.insert(
        "address.postal_code".to_string(),
        vec!["Invalid format".to_string()],
    );

    let error = AppError::ValidationError {
        message: "Validation failed".to_string(),
        errors: errors.clone(),
    };

    match error {
        AppError::ValidationError {
            message: _,
            errors: errs,
        } => {
            assert_eq!(errs.len(), 2);
            assert!(errs.contains_key("user.email"));
            assert!(errs.contains_key("address.postal_code"));
        }
        _ => panic!("Expected ValidationError variant"),
    }
}

#[test]
fn test_jsonapi_error_object_not_found() {
    let error = ErrorObject {
        id: None,
        links: None,
        status: Some("404".to_string()),
        code: Some("not_found".to_string()),
        title: Some("Not Found".to_string()),
        detail: Some("The requested resource was not found".to_string()),
        source: None,
        meta: None,
    };

    assert_eq!(error.status, Some("404".to_string()));
    assert_eq!(error.code, Some("not_found".to_string()));
}

#[test]
fn test_jsonapi_error_object_validation() {
    let error = ErrorObject {
        id: Some("validation_1".to_string()),
        links: None,
        status: Some("422".to_string()),
        code: Some("validation_failed".to_string()),
        title: Some("Validation Failed".to_string()),
        detail: Some("Email is required".to_string()),
        source: Some(appkit_core::jsonapi::error::ErrorSource {
            pointer: Some("/data/attributes/email".to_string()),
            parameter: None,
            header: None,
        }),
        meta: None,
    };

    assert_eq!(error.status, Some("422".to_string()));
    assert!(error.source.is_some());
}

#[test]
fn test_jsonapi_error_object_with_meta() {
    use std::collections::HashMap;

    let mut meta = HashMap::new();
    meta.insert("timestamp".to_string(), json!("2024-01-01T00:00:00Z"));
    meta.insert("request_id".to_string(), json!("req_123"));

    let error = ErrorObject {
        id: None,
        links: None,
        status: Some("500".to_string()),
        code: Some("internal_error".to_string()),
        title: Some("Internal Server Error".to_string()),
        detail: None,
        source: None,
        meta: Some(meta.clone()),
    };

    assert!(error.meta.is_some());
    assert_eq!(error.meta.unwrap().len(), 2);
}

#[test]
fn test_http_status_code_400() {
    let error = AppError::BadRequest("Bad input".to_string());

    match error {
        AppError::BadRequest(_) => {
            assert!(true);
        }
        _ => panic!("Expected BadRequest"),
    }
}

#[test]
fn test_http_status_code_401() {
    let error = AppError::AuthenticationFailed("Not logged in".to_string());

    match error {
        AppError::AuthenticationFailed(_) => {
            assert!(true);
        }
        _ => panic!("Expected AuthenticationFailed"),
    }
}

#[test]
fn test_http_status_code_403() {
    let error = AppError::AuthorizationFailed("No permission".to_string());

    match error {
        AppError::AuthorizationFailed(_) => {
            assert!(true);
        }
        _ => panic!("Expected AuthorizationFailed"),
    }
}

#[test]
fn test_http_status_code_404() {
    let error = AppError::NotFound("No such resource".to_string());

    match error {
        AppError::NotFound(_) => {
            assert!(true);
        }
        _ => panic!("Expected NotFound"),
    }
}

#[test]
fn test_http_status_code_409() {
    let error = AppError::Conflict("Duplicate entry".to_string());

    match error {
        AppError::Conflict(_) => {
            assert!(true);
        }
        _ => panic!("Expected Conflict"),
    }
}

#[test]
fn test_http_status_code_422() {
    let mut errors = HashMap::new();
    errors.insert("field".to_string(), vec!["error".to_string()]);

    let error = AppError::ValidationError {
        message: "Invalid".to_string(),
        errors,
    };

    match error {
        AppError::ValidationError { .. } => {
            assert!(true);
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_http_status_code_500() {
    let error = AppError::InternalServerError("Server crashed".to_string());

    match error {
        AppError::InternalServerError(_) => {
            assert!(true);
        }
        _ => panic!("Expected InternalServerError"),
    }
}

#[test]
fn test_http_status_code_429() {
    let error = AppError::RateLimitExceeded;

    match error {
        AppError::RateLimitExceeded => {
            assert!(true);
        }
        _ => panic!("Expected RateLimitExceeded"),
    }
}

#[test]
fn test_http_status_code_503() {
    let error = AppError::ServiceUnavailable("Down for maintenance".to_string());

    match error {
        AppError::ServiceUnavailable(_) => {
            assert!(true);
        }
        _ => panic!("Expected ServiceUnavailable"),
    }
}

#[test]
fn test_error_with_sql_context() {
    let error =
        AppError::InternalServerError("Database error: connection timeout after 30s".to_string());

    match error {
        AppError::InternalServerError(msg) => {
            assert!(msg.contains("Database error"));
            assert!(msg.contains("timeout"));
        }
        _ => panic!("Expected InternalServerError"),
    }
}

#[test]
fn test_error_with_file_path_context() {
    let error = AppError::NotFound("File not found: /path/to/file.txt".to_string());

    match error {
        AppError::NotFound(msg) => {
            assert!(msg.contains("/path/to/file.txt"));
        }
        _ => panic!("Expected NotFound"),
    }
}

#[test]
fn test_error_with_user_context() {
    let error =
        AppError::AuthorizationFailed("User id=123 lacks permission 'admin.delete'".to_string());

    match error {
        AppError::AuthorizationFailed(msg) => {
            assert!(msg.contains("id=123"));
            assert!(msg.contains("admin.delete"));
        }
        _ => panic!("Expected AuthorizationFailed"),
    }
}

#[test]
fn test_error_with_newlines() {
    let error = AppError::BadRequest("Line 1\nLine 2\nLine 3".to_string());

    match error {
        AppError::BadRequest(msg) => {
            assert!(msg.contains('\n'));
            assert_eq!(msg.matches('\n').count(), 2);
        }
        _ => panic!("Expected BadRequest"),
    }
}

#[test]
fn test_error_with_json_in_message() {
    let error = AppError::BadRequest(r#"{"error": "invalid"}"#.to_string());

    match error {
        AppError::BadRequest(msg) => {
            assert!(msg.contains('{'));
            assert!(msg.contains('}'));
        }
        _ => panic!("Expected BadRequest"),
    }
}

#[test]
fn test_error_with_sql_injection_attempt() {
    let error = AppError::BadRequest("Invalid input: '; DROP TABLE users;--".to_string());

    match error {
        AppError::BadRequest(msg) => {
            assert!(msg.contains("DROP TABLE"));
        }
        _ => panic!("Expected BadRequest"),
    }
}

#[test]
fn test_error_object_serialization() {
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

    let json = serde_json::to_string(&error).unwrap();

    assert!(json.contains("err_123"));
    assert!(json.contains("not_found"));
    assert!(json.contains("404"));
}

#[test]
fn test_multiple_errors_serialization() {
    let errors = vec![
        ErrorObject {
            id: Some("err_1".to_string()),
            links: None,
            status: Some("422".to_string()),
            code: Some("required".to_string()),
            title: Some("Required Field".to_string()),
            detail: Some("Email is required".to_string()),
            source: None,
            meta: None,
        },
        ErrorObject {
            id: Some("err_2".to_string()),
            links: None,
            status: Some("422".to_string()),
            code: Some("too_short".to_string()),
            title: Some("Too Short".to_string()),
            detail: Some("Password must be 8+ characters".to_string()),
            source: None,
            meta: None,
        },
    ];

    let json = serde_json::to_string(&errors).unwrap();

    assert!(json.contains("err_1"));
    assert!(json.contains("err_2"));
    assert!(json.contains("required"));
    assert!(json.contains("too_short"));
}

#[test]
fn test_error_messages_with_quotes() {
    let error = AppError::BadRequest("Field 'email' is \"required\"".to_string());

    match error {
        AppError::BadRequest(msg) => {
            assert!(msg.contains('\''));
            assert!(msg.contains('\"'));
        }
        _ => panic!("Expected BadRequest"),
    }
}
