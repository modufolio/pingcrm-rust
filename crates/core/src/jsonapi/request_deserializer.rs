use super::error::ErrorObject;
use super::input_normalizer::{InputNormalizer, NormalizedPayload};
use super::validation::{validation_errors_to_jsonapi, ValidationErrorsExt};
use crate::error::{AppError, AppResult};
use serde_json::Value;
use validator::Validate;

pub trait DeserializeJsonApi: Sized + Validate {
    fn from_normalized(payload: &NormalizedPayload) -> AppResult<Self>;

    fn deserialize_and_validate(
        payload: &Value,
        content_type: &str,
        expected_type: &str,
    ) -> Result<Self, Vec<ErrorObject>> {
        let normalizer = InputNormalizer::new();
        let normalized = normalizer
            .normalize(payload, content_type, expected_type)
            .map_err(|e| {
                vec![ErrorObject::from_status(
                    400,
                    "Request Normalization Error",
                    e.to_string(),
                )]
            })?;

        let instance = Self::from_normalized(&normalized).map_err(|e| {
            vec![ErrorObject::from_status(
                400,
                "Deserialization Error",
                e.to_string(),
            )]
        })?;

        match instance.validate() {
            Ok(_) => Ok(instance),
            Err(errors) => Err(errors.to_jsonapi_errors()),
        }
    }

    fn deserialize(payload: &Value, content_type: &str, expected_type: &str) -> AppResult<Self> {
        let normalizer = InputNormalizer::new();
        let normalized = normalizer.normalize(payload, content_type, expected_type)?;
        Self::from_normalized(&normalized)
    }
}

pub fn deserialize_simple<T>(payload: &Value, content_type: &str) -> AppResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let normalizer = InputNormalizer::new();

    let normalized = normalizer.normalize(payload, content_type, "")?;

    serde_json::from_value(normalized.attributes)
        .map_err(|e| AppError::BadRequest(format!("Failed to deserialize: {}", e)))
}

pub fn deserialize_and_validate_simple<T>(
    payload: &Value,
    content_type: &str,
) -> Result<T, Vec<ErrorObject>>
where
    T: serde::de::DeserializeOwned + Validate,
{
    let normalizer = InputNormalizer::new();

    let normalized = normalizer
        .normalize(payload, content_type, "")
        .map_err(|e| {
            vec![ErrorObject::from_status(
                400,
                "Request Normalization Error",
                e.to_string(),
            )]
        })?;

    let instance: T = serde_json::from_value(normalized.attributes).map_err(|e| {
        vec![ErrorObject::from_status(
            400,
            "Deserialization Error",
            e.to_string(),
        )]
    })?;

    match instance.validate() {
        Ok(_) => Ok(instance),
        Err(errors) => Err(validation_errors_to_jsonapi(&errors)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::RelationshipValue;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize, Validate)]
    struct TestRequest {
        #[validate(length(min = 2, max = 50))]
        name: String,

        #[validate(email)]
        email: String,

        #[serde(skip)]
        organization_id: Option<i32>,
    }

    impl DeserializeJsonApi for TestRequest {
        fn from_normalized(payload: &NormalizedPayload) -> AppResult<Self> {
            let mut request: Self = serde_json::from_value(payload.attributes.clone())
                .map_err(|e| AppError::BadRequest(format!("Deserialization failed: {}", e)))?;

            if let Some(org_rel) = payload.relationship("organization") {
                request.organization_id = org_rel.as_one();
            }

            Ok(request)
        }
    }

    #[test]
    fn test_deserialize_jsonapi_with_validation_success() {
        let payload = json!({
            "data": {
                "type": "contacts",
                "attributes": {
                    "name": "John Doe",
                    "email": "john@example.com"
                },
                "relationships": {
                    "organization": {
                        "data": {"type": "organizations", "id": "5"}
                    }
                }
            }
        });

        let result =
            TestRequest::deserialize_and_validate(&payload, "application/vnd.api+json", "contacts");

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.name, "John Doe");
        assert_eq!(request.email, "john@example.com");
        assert_eq!(request.organization_id, Some(5));
    }

    #[test]
    fn test_deserialize_jsonapi_validation_failure() {
        let payload = json!({
            "data": {
                "type": "contacts",
                "attributes": {
                    "name": "J",
                    "email": "invalid-email"
                }
            }
        });

        let result =
            TestRequest::deserialize_and_validate(&payload, "application/vnd.api+json", "contacts");

        assert!(result.is_err());
        let errors = result.unwrap_err();

        assert_eq!(errors.len(), 2);

        for error in &errors {
            assert_eq!(error.status, Some("422".to_string()));
        }
    }

    #[test]
    fn test_deserialize_plain_json_with_validation() {
        let payload = json!({
            "name": "Jane Smith",
            "email": "jane@example.com",
            "organization_id": 10
        });

        let result = TestRequest::deserialize_and_validate(&payload, "application/json", "");

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.name, "Jane Smith");
        assert_eq!(request.email, "jane@example.com");
        assert_eq!(request.organization_id, Some(10));
    }

    #[test]
    fn test_relationship_helpers() {
        let one = RelationshipValue::One(5);
        assert_eq!(one.as_one(), Some(5));
        assert_eq!(one.as_many(), None);
        assert!(!one.is_null());
        assert!(one.is_one());

        let many = RelationshipValue::Many(vec![1, 2, 3]);
        assert_eq!(many.as_one(), None);
        assert_eq!(many.as_many(), Some(&[1, 2, 3][..]));
        assert!(!many.is_null());
        assert!(many.is_many());

        let null = RelationshipValue::Null;
        assert_eq!(null.as_one(), None);
        assert_eq!(null.as_many(), None);
        assert!(null.is_null());
    }

    #[test]
    fn test_deserialize_simple() {
        #[derive(Debug, Deserialize)]
        struct Simple {
            name: String,
            count: i32,
        }

        let payload = json!({
            "name": "Test",
            "count": 42
        });

        let result: AppResult<Simple> = deserialize_simple(&payload, "application/json");
        assert!(result.is_ok());

        let simple = result.unwrap();
        assert_eq!(simple.name, "Test");
        assert_eq!(simple.count, 42);
    }
}
