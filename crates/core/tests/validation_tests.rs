use appkit_core::validation::result::{ValidationError, ValidationResult};
use validator::Validate;

#[derive(Debug, Validate)]
struct SimpleForm {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8))]
    password: String,
}

#[test]
fn test_validation_result_success() {
    let form = SimpleForm {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = ValidationResult::from_validatable(&form);

    assert!(!result.failed());
    assert!(result.passed());
}

#[test]
fn test_validation_result_with_errors() {
    let form = SimpleForm {
        email: "invalid-email".to_string(),
        password: "short".to_string(),
    };

    let result = ValidationResult::from_validatable(&form);

    assert!(result.failed());
    assert!(!result.passed());
}

#[test]
fn test_validation_result_error_messages() {
    let form = SimpleForm {
        email: "invalid-email".to_string(),
        password: "short".to_string(),
    };

    let result = ValidationResult::from_validatable(&form);

    let messages = result.messages();
    assert!(messages.len() > 0);
}

#[test]
fn test_validation_result_first_error() {
    let form = SimpleForm {
        email: "invalid-email".to_string(),
        password: "validpassword".to_string(),
    };

    let result = ValidationResult::from_validatable(&form);

    let first_error = result.first("email");
    assert!(first_error.is_some());

    let password_error = result.first("password");
    assert!(password_error.is_none());
}

#[test]
fn test_validation_result_all_errors() {
    let form = SimpleForm {
        email: "invalid".to_string(),
        password: "bad".to_string(),
    };

    let result = ValidationResult::from_validatable(&form);

    let errors = result.errors();

    assert!(errors.contains_key("email"));
    assert!(errors.contains_key("password"));
}

#[test]
fn test_validation_error_creation() {
    let error = ValidationError {
        field: "email".to_string(),
        message: "Invalid email format".to_string(),
        code: Some("invalid_email".to_string()),
    };

    assert_eq!(error.field, "email");
    assert_eq!(error.message, "Invalid email format");
    assert_eq!(error.code, Some("invalid_email".to_string()));
}

#[test]
fn test_validation_error_without_code() {
    let error = ValidationError {
        field: "password".to_string(),
        message: "Password too short".to_string(),
        code: None,
    };

    assert_eq!(error.field, "password");
    assert_eq!(error.message, "Password too short");
    assert!(error.code.is_none());
}

#[derive(Debug, Validate)]
struct ComplexForm {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8, max = 100))]
    password: String,

    #[validate(range(min = 18, max = 120))]
    age: i32,
}

#[test]
fn test_validation_boundary_values() {
    let form1 = ComplexForm {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        age: 18,
    };
    assert!(ValidationResult::from_validatable(&form1).passed());

    let form2 = ComplexForm {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        age: 120,
    };
    assert!(ValidationResult::from_validatable(&form2).passed());

    let form3 = ComplexForm {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        age: 17,
    };
    assert!(ValidationResult::from_validatable(&form3).failed());
}

#[test]
fn test_validation_email_with_unicode() {
    let form = SimpleForm {
        email: "user@рф.example".to_string(),
        password: "password123".to_string(),
    };

    let result = ValidationResult::from_validatable(&form);

    let _ = result.passed();
}

#[test]
fn test_validation_password_with_special_chars() {
    let form = SimpleForm {
        email: "test@example.com".to_string(),
        password: "P@ssw0rd!#$%".to_string(),
    };

    assert!(ValidationResult::from_validatable(&form).passed());
}

#[test]
fn test_validation_empty_values() {
    let form = SimpleForm {
        email: "".to_string(),
        password: "".to_string(),
    };

    let result = ValidationResult::from_validatable(&form);

    assert!(result.failed());
    assert!(result.errors().len() >= 2);
}

#[test]
fn test_validation_result_success_factory() {
    let result = ValidationResult::success();

    assert!(result.passed());
    assert!(!result.failed());
    assert_eq!(result.messages().len(), 0);
}
