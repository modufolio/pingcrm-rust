use super::error::{ErrorObject, ErrorSource};
use serde_json::json;
use std::collections::HashMap;
use validator::{ValidationError, ValidationErrors};

pub fn validation_errors_to_jsonapi(errors: &ValidationErrors) -> Vec<ErrorObject> {
    let mut error_objects = Vec::new();

    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            error_objects.push(validation_error_to_error_object(field, error));
        }
    }

    error_objects
}

fn validation_error_to_error_object(field: &str, error: &ValidationError) -> ErrorObject {
    let detail = error
        .message
        .as_ref()
        .map(|m| m.to_string())
        .unwrap_or_else(|| match error.code.as_ref() {
            "email" => "Invalid email format".to_string(),
            "length" => format_length_error(error),
            "range" => format_range_error(error),
            "url" => "Invalid URL format".to_string(),
            "required" => format!("Field '{}' is required", field),
            "custom" => "Validation failed".to_string(),
            _ => format!("Validation failed for '{}'", field),
        });

    ErrorObject {
        id: None,
        links: None,
        status: Some("422".to_string()),
        code: Some(error.code.to_string()),
        title: Some("Validation Error".to_string()),
        detail: Some(detail),
        source: Some(ErrorSource {
            pointer: Some(format!("/data/attributes/{}", field)),
            parameter: None,
            header: None,
        }),
        meta: if error.params.is_empty() {
            None
        } else {
            Some(
                error
                    .params
                    .iter()
                    .map(|(k, v)| (k.to_string(), json!(v)))
                    .collect(),
            )
        },
    }
}

fn format_length_error(error: &ValidationError) -> String {
    if let Some(min) = error.params.get("min") {
        if let Some(max) = error.params.get("max") {
            return format!("Length must be between {} and {} characters", min, max);
        }
        return format!("Length must be at least {} characters", min);
    }

    if let Some(max) = error.params.get("max") {
        return format!("Length must not exceed {} characters", max);
    }

    if let Some(equal) = error.params.get("equal") {
        return format!("Length must be exactly {} characters", equal);
    }

    "Invalid length".to_string()
}

fn format_range_error(error: &ValidationError) -> String {
    if let Some(min) = error.params.get("min") {
        if let Some(max) = error.params.get("max") {
            return format!("Value must be between {} and {}", min, max);
        }
        return format!("Value must be at least {}", min);
    }

    if let Some(max) = error.params.get("max") {
        return format!("Value must not exceed {}", max);
    }

    "Value out of range".to_string()
}

pub fn field_errors_to_jsonapi(errors: &HashMap<String, Vec<String>>) -> Vec<ErrorObject> {
    let mut error_objects = Vec::new();

    for (field, messages) in errors {
        for message in messages {
            error_objects.push(ErrorObject {
                id: None,
                links: None,
                status: Some("422".to_string()),
                code: Some("validation_error".to_string()),
                title: Some("Validation Error".to_string()),
                detail: Some(message.clone()),
                source: Some(ErrorSource {
                    pointer: Some(format!("/data/attributes/{}", field)),
                    parameter: None,
                    header: None,
                }),
                meta: None,
            });
        }
    }

    error_objects
}

pub trait ValidationErrorsExt {
    fn to_jsonapi_errors(&self) -> Vec<ErrorObject>;

    fn has_errors(&self) -> bool;
}

impl ValidationErrorsExt for ValidationErrors {
    fn to_jsonapi_errors(&self) -> Vec<ErrorObject> {
        validation_errors_to_jsonapi(self)
    }

    fn has_errors(&self) -> bool {
        !self.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Debug, Validate)]
    struct TestRequest {
        #[validate(email)]
        email: String,

        #[validate(length(min = 8, max = 50))]
        password: String,

        #[validate(range(min = 18, max = 120))]
        age: i32,
    }

    #[test]
    fn test_email_validation_error() {
        let request = TestRequest {
            email: "invalid-email".to_string(),
            password: "validpassword123".to_string(),
            age: 25,
        };

        if let Err(errors) = request.validate() {
            let error_objects = validation_errors_to_jsonapi(&errors);

            assert!(!error_objects.is_empty());
            assert_eq!(error_objects[0].status, Some("422".to_string()));
            assert_eq!(error_objects[0].code, Some("email".to_string()));
            assert!(error_objects[0].source.is_some());
            assert_eq!(
                error_objects[0].source.as_ref().unwrap().pointer,
                Some("/data/attributes/email".to_string())
            );
        } else {
            panic!("Expected validation errors");
        }
    }

    #[test]
    fn test_length_validation_error() {
        let request = TestRequest {
            email: "valid@example.com".to_string(),
            password: "short".to_string(),
            age: 25,
        };

        if let Err(errors) = request.validate() {
            let error_objects = validation_errors_to_jsonapi(&errors);

            assert!(!error_objects.is_empty());
            let password_error = error_objects
                .iter()
                .find(|e| {
                    e.source
                        .as_ref()
                        .and_then(|s| s.pointer.as_ref())
                        .map(|p| p == "/data/attributes/password")
                        .unwrap_or(false)
                })
                .expect("Password error should exist");

            assert_eq!(password_error.status, Some("422".to_string()));
            assert_eq!(password_error.code, Some("length".to_string()));
            assert!(password_error
                .detail
                .as_ref()
                .unwrap()
                .contains("between 8 and 50 characters"));
        } else {
            panic!("Expected validation errors");
        }
    }

    #[test]
    fn test_range_validation_error() {
        let request = TestRequest {
            email: "valid@example.com".to_string(),
            password: "validpassword".to_string(),
            age: 15,
        };

        if let Err(errors) = request.validate() {
            let error_objects = validation_errors_to_jsonapi(&errors);

            assert!(!error_objects.is_empty());
            let age_error = error_objects
                .iter()
                .find(|e| {
                    e.source
                        .as_ref()
                        .and_then(|s| s.pointer.as_ref())
                        .map(|p| p == "/data/attributes/age")
                        .unwrap_or(false)
                })
                .expect("Age error should exist");

            assert_eq!(age_error.status, Some("422".to_string()));
            assert_eq!(age_error.code, Some("range".to_string()));
            assert!(age_error
                .detail
                .as_ref()
                .unwrap()
                .contains("between 18 and 120"));
        } else {
            panic!("Expected validation errors");
        }
    }

    #[test]
    fn test_multiple_validation_errors() {
        let request = TestRequest {
            email: "invalid".to_string(),
            password: "bad".to_string(),
            age: 200,
        };

        if let Err(errors) = request.validate() {
            let error_objects = validation_errors_to_jsonapi(&errors);

            assert_eq!(error_objects.len(), 3);

            for error in &error_objects {
                assert_eq!(error.status, Some("422".to_string()));
                assert_eq!(error.title, Some("Validation Error".to_string()));
            }
        } else {
            panic!("Expected validation errors");
        }
    }

    #[test]
    fn test_field_errors_to_jsonapi() {
        let mut errors = HashMap::new();
        errors.insert("email".to_string(), vec!["Invalid format".to_string()]);
        errors.insert(
            "password".to_string(),
            vec!["Too short".to_string(), "No special characters".to_string()],
        );

        let error_objects = field_errors_to_jsonapi(&errors);

        assert_eq!(error_objects.len(), 3);

        for error in &error_objects {
            assert_eq!(error.status, Some("422".to_string()));
            assert!(error.source.is_some());
        }
    }

    #[test]
    fn test_validation_errors_ext_trait() {
        let request = TestRequest {
            email: "invalid".to_string(),
            password: "validpassword".to_string(),
            age: 25,
        };

        if let Err(errors) = request.validate() {
            assert!(errors.has_errors());
            let error_objects = errors.to_jsonapi_errors();
            assert!(!error_objects.is_empty());
        } else {
            panic!("Expected validation errors");
        }
    }
}
