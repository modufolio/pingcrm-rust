use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::{Validate, ValidationErrors};

pub type FieldErrors = HashMap<String, Vec<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    errors: Option<ValidationErrors>,
}

impl ValidationResult {
    pub fn new(errors: Option<ValidationErrors>) -> Self {
        Self { errors }
    }

    pub fn from_validatable<T: Validate>(value: &T) -> Self {
        match value.validate() {
            Ok(_) => Self::new(None),
            Err(errors) => Self::new(Some(errors)),
        }
    }

    pub fn success() -> Self {
        Self::new(None)
    }

    pub fn single_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        let field_str = field.into();
        let message_str = message.into();
        let mut errors = ValidationErrors::new();

        let field_static: &'static str = Box::leak(field_str.into_boxed_str());
        let code: &'static str = Box::leak(message_str.clone().into_boxed_str());

        let mut error = validator::ValidationError::new(code);
        error.message = Some(std::borrow::Cow::Owned(message_str));

        errors.add(field_static, error);
        Self::new(Some(errors))
    }

    pub fn passed(&self) -> bool {
        self.errors.is_none()
    }

    pub fn failed(&self) -> bool {
        self.errors.is_some()
    }

    pub fn errors(&self) -> FieldErrors {
        let Some(ref errors) = self.errors else {
            return HashMap::new();
        };

        let mut field_errors: FieldErrors = HashMap::new();

        for (field, errors_for_field) in errors.field_errors() {
            let messages: Vec<String> = errors_for_field
                .iter()
                .map(|e| {
                    e.message
                        .as_ref()
                        .map(|cow| cow.to_string())
                        .unwrap_or_else(|| format!("Validation failed for field: {}", field))
                })
                .collect();

            field_errors.insert(field.to_string(), messages);
        }

        field_errors
    }

    pub fn first(&self, field: &str) -> Option<String> {
        self.errors()
            .get(field)
            .and_then(|messages| messages.first().cloned())
    }

    pub fn messages(&self) -> Vec<String> {
        let Some(ref errors) = self.errors else {
            return Vec::new();
        };

        errors
            .field_errors()
            .iter()
            .flat_map(|(field, field_errors)| {
                field_errors.iter().map(move |e| {
                    e.message
                        .as_ref()
                        .map(|cow| cow.to_string())
                        .unwrap_or_else(|| format!("Validation failed for field: {}", field))
                })
            })
            .collect()
    }

    pub fn validation_errors(&self) -> Vec<ValidationError> {
        let Some(ref errors) = self.errors else {
            return Vec::new();
        };

        errors
            .field_errors()
            .iter()
            .flat_map(|(field, field_errors)| {
                field_errors.iter().map(move |e| ValidationError {
                    field: field.to_string(),
                    message: e
                        .message
                        .as_ref()
                        .map(|cow| cow.to_string())
                        .unwrap_or_else(|| format!("Validation failed for field: {}", field)),
                    code: Some(e.code.to_string()),
                })
            })
            .collect()
    }

    pub fn raw_errors(&self) -> Option<&ValidationErrors> {
        self.errors.as_ref()
    }

    pub fn into_result(self) -> Result<(), crate::error::AppError> {
        if self.passed() {
            Ok(())
        } else {
            Err(crate::error::AppError::ValidationFailed(
                self.messages().join(", "),
            ))
        }
    }

    pub fn throw_if_failed(self) -> Result<(), crate::error::AppError> {
        self.into_result()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::success()
    }
}

impl From<ValidationErrors> for ValidationResult {
    fn from(errors: ValidationErrors) -> Self {
        Self::new(Some(errors))
    }
}

impl From<Result<(), ValidationErrors>> for ValidationResult {
    fn from(result: Result<(), ValidationErrors>) -> Self {
        match result {
            Ok(_) => Self::success(),
            Err(errors) => Self::new(Some(errors)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Validate)]
    struct TestForm {
        #[validate(email)]
        email: String,

        #[validate(length(min = 8))]
        password: String,
    }

    #[test]
    fn test_validation_success() {
        let form = TestForm {
            email: "test@example.com".to_string(),
            password: "longenoughpassword".to_string(),
        };

        let result = ValidationResult::from_validatable(&form);
        assert!(result.passed());
        assert!(!result.failed());
        assert!(result.errors().is_empty());
    }

    #[test]
    fn test_validation_failure() {
        let form = TestForm {
            email: "invalid-email".to_string(),
            password: "short".to_string(),
        };

        let result = ValidationResult::from_validatable(&form);
        assert!(!result.passed());
        assert!(result.failed());
        assert!(!result.errors().is_empty());
    }

    #[test]
    fn test_first_error() {
        let form = TestForm {
            email: "invalid".to_string(),
            password: "short".to_string(),
        };

        let result = ValidationResult::from_validatable(&form);
        assert!(result.first("email").is_some());
        assert!(result.first("nonexistent").is_none());
    }

    #[test]
    fn test_messages() {
        let form = TestForm {
            email: "invalid".to_string(),
            password: "short".to_string(),
        };

        let result = ValidationResult::from_validatable(&form);
        let messages = result.messages();
        assert!(!messages.is_empty());
    }
}
