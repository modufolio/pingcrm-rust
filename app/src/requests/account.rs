use appkit_core::jsonapi::{DeserializeJsonApi, NormalizedPayload};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAccountRequest {
    #[validate(length(min = 2, max = 100))]
    pub name: String,
}

impl DeserializeJsonApi for CreateAccountRequest {
    fn from_normalized(payload: &NormalizedPayload) -> Result<Self, appkit_core::error::AppError> {
        let request: Self = serde_json::from_value(payload.attributes.clone()).map_err(|e| {
            appkit_core::error::AppError::BadRequest(format!("Invalid attributes: {}", e))
        })?;

        Ok(request)
    }
}

impl crate::database::ToNewModel<crate::database::models::NewAccount> for CreateAccountRequest {
    fn to_new_model(&self) -> crate::database::models::NewAccount {
        use crate::database::models::NewAccount;

        NewAccount::new(self.name.clone())
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAccountRequest {
    #[validate(length(min = 2, max = 100))]
    pub name: Option<String>,
}

impl DeserializeJsonApi for UpdateAccountRequest {
    fn from_normalized(payload: &NormalizedPayload) -> Result<Self, appkit_core::error::AppError> {
        let request: Self = serde_json::from_value(payload.attributes.clone()).map_err(|e| {
            appkit_core::error::AppError::BadRequest(format!("Invalid attributes: {}", e))
        })?;

        Ok(request)
    }
}

impl crate::database::ToUpdateModel<crate::database::models::AccountUpdate>
    for UpdateAccountRequest
{
    fn to_update_model(&self) -> crate::database::models::AccountUpdate {
        use crate::database::models::AccountUpdate;
        use chrono::Utc;

        let mut update = AccountUpdate::new();
        update.updated_at = Utc::now().naive_utc();

        if let Some(name) = &self.name {
            update.name = Some(name.clone());
        }

        update
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_account_validation() {
        let request = CreateAccountRequest {
            name: "A".to_string(),
        };

        let result = request.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_create_account_valid() {
        let request = CreateAccountRequest {
            name: "Acme Corporation".to_string(),
        };

        let result = request.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_from_jsonapi() {
        let payload = json!({
            "data": {
                "type": "accounts",
                "attributes": {
                    "name": "Acme Corporation"
                }
            }
        });

        let result = CreateAccountRequest::deserialize_and_validate(
            &payload,
            "application/vnd.api+json",
            "accounts",
        );

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.name, "Acme Corporation");
    }
}
