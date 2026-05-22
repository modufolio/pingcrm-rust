use super::validation_result::ValidationResult;
use crate::error::AppError;
use axum::{
    extract::{FromRequest, Query, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(format!("Invalid JSON payload: {}", e)))?;

        let result = ValidationResult::from_validatable(&value);

        if result.failed() {
            let error_messages = result.messages().join(", ");
            return Err(AppError::ValidationError {
                message: error_messages,
                errors: result.errors(),
            });
        }

        Ok(ValidatedJson(value))
    }
}

pub struct ValidatedForm<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Form(value) = axum::Form::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(format!("Invalid form data: {}", e)))?;

        let result = ValidationResult::from_validatable(&value);

        if result.failed() {
            let error_messages = result.messages().join(", ");
            return Err(AppError::ValidationError {
                message: error_messages,
                errors: result.errors(),
            });
        }

        Ok(ValidatedForm(value))
    }
}

pub struct ValidatedQuery<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(format!("Invalid query parameters: {}", e)))?;

        let result = ValidationResult::from_validatable(&value);

        if result.failed() {
            let error_messages = result.messages().join(", ");
            return Err(AppError::ValidationError {
                message: error_messages,
                errors: result.errors(),
            });
        }

        Ok(ValidatedQuery(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, extract::FromRequest, http::Request as HttpRequest};
    use serde::Deserialize;
    use validator::Validate;

    #[derive(Debug, Deserialize, Validate)]
    struct TestPayload {
        #[validate(email)]
        email: String,

        #[validate(length(min = 8))]
        password: String,
    }

    #[tokio::test]
    async fn test_validated_json_success() {
        let json = r#"{"email": "test@example.com", "password": "validpassword"}"#;
        let req = HttpRequest::builder()
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap();

        let result = ValidatedJson::<TestPayload>::from_request(req, &()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validated_json_invalid_email() {
        let json = r#"{"email": "invalid", "password": "validpassword"}"#;
        let req = HttpRequest::builder()
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap();

        let result = ValidatedJson::<TestPayload>::from_request(req, &()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validated_json_short_password() {
        let json = r#"{"email": "test@example.com", "password": "short"}"#;
        let req = HttpRequest::builder()
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap();

        let result = ValidatedJson::<TestPayload>::from_request(req, &()).await;
        assert!(result.is_err());
    }
}
