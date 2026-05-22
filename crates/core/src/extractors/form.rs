use super::validation_result::ValidationResult;
use validator::Validate;

pub trait Form: Validate + Sized {
    fn validate_self(&self) -> ValidationResult {
        ValidationResult::from_validatable(self)
    }

    fn validate_or_error(&self) -> Result<(), crate::error::AppError> {
        self.validate_self().throw_if_failed()
    }

    fn validate_and_return(self) -> Result<Self, crate::error::AppError> {
        self.validate_or_error()?;
        Ok(self)
    }
}

impl<T> Form for T where T: Validate {}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Debug, Validate)]
    struct TestForm {
        #[validate(email)]
        email: String,

        #[validate(length(min = 8))]
        password: String,
    }

    #[test]
    fn test_validate_self_success() {
        let form = TestForm {
            email: "test@example.com".to_string(),
            password: "validpassword".to_string(),
        };

        let result = form.validate_self();
        assert!(result.passed());
    }

    #[test]
    fn test_validate_self_failure() {
        let form = TestForm {
            email: "invalid".to_string(),
            password: "short".to_string(),
        };

        let result = form.validate_self();
        assert!(result.failed());
    }

    #[test]
    fn test_validate_or_error_success() {
        let form = TestForm {
            email: "test@example.com".to_string(),
            password: "validpassword".to_string(),
        };

        assert!(form.validate_or_error().is_ok());
    }

    #[test]
    fn test_validate_or_error_failure() {
        let form = TestForm {
            email: "invalid".to_string(),
            password: "short".to_string(),
        };

        assert!(form.validate_or_error().is_err());
    }

    #[test]
    fn test_validate_and_return_success() {
        let form = TestForm {
            email: "test@example.com".to_string(),
            password: "validpassword".to_string(),
        };

        let result = form.validate_and_return();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_and_return_failure() {
        let form = TestForm {
            email: "invalid".to_string(),
            password: "short".to_string(),
        };

        let result = form.validate_and_return();
        assert!(result.is_err());
    }
}
