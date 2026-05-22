use appkit_core::jsonapi::{DeserializeJsonApi, NormalizedPayload};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 2, max = 50))]
    pub first_name: String,

    #[validate(length(min = 2, max = 50))]
    pub last_name: String,

    #[validate(length(min = 8, max = 255))]
    pub password: String,

    pub owner: Option<bool>,

    pub photo_filename: Option<String>,

    pub account_id: Option<i32>,
}

impl DeserializeJsonApi for CreateUserRequest {
    fn from_normalized(payload: &NormalizedPayload) -> Result<Self, appkit_core::error::AppError> {
        let mut request: Self =
            serde_json::from_value(payload.attributes.clone()).map_err(|e| {
                appkit_core::error::AppError::BadRequest(format!("Invalid attributes: {}", e))
            })?;

        if let Some(account_rel) = payload.relationship("account") {
            if let Some(id) = account_rel.as_one() {
                request.account_id = Some(id);
            }
        }

        Ok(request)
    }
}

impl crate::database::ToNewModel<crate::database::models::NewUser> for CreateUserRequest {
    fn to_new_model(&self) -> crate::database::models::NewUser {
        use crate::database::models::NewUser;

        let mut new_user = NewUser::new(
            self.email.clone(),
            self.password.clone(),
            self.first_name.clone(),
            self.last_name.clone(),
        );

        new_user.owner = self.owner.unwrap_or(false);
        new_user.photo_filename = self.photo_filename.clone();
        new_user.account_id = self.account_id.or(Some(1));

        new_user
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(min = 2, max = 50))]
    pub first_name: Option<String>,

    #[validate(length(min = 2, max = 50))]
    pub last_name: Option<String>,

    pub owner: Option<bool>,

    pub photo_filename: Option<Option<String>>,

    pub account_id: Option<Option<i32>>,
}

impl DeserializeJsonApi for UpdateUserRequest {
    fn from_normalized(payload: &NormalizedPayload) -> Result<Self, appkit_core::error::AppError> {
        let mut request: Self =
            serde_json::from_value(payload.attributes.clone()).map_err(|e| {
                appkit_core::error::AppError::BadRequest(format!("Invalid attributes: {}", e))
            })?;

        if let Some(account_rel) = payload.relationship("account") {
            if account_rel.is_null() {
                request.account_id = Some(None);
            } else if let Some(id) = account_rel.as_one() {
                request.account_id = Some(Some(id));
            }
        }

        Ok(request)
    }
}

impl crate::database::ToUpdateModel<crate::database::models::UserUpdate> for UpdateUserRequest {
    fn to_update_model(&self) -> crate::database::models::UserUpdate {
        use crate::database::models::UserUpdate;
        use chrono::Utc;

        let mut update = UserUpdate::new();
        update.updated_at = Utc::now().naive_utc();

        update.email = self.email.clone();
        update.first_name = self.first_name.clone();
        update.last_name = self.last_name.clone();
        update.photo_filename = self.photo_filename.clone().flatten();

        update
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_user_validation_fails() {
        let request = CreateUserRequest {
            email: "invalid-email".to_string(),
            first_name: "J".to_string(),
            last_name: "D".to_string(),
            password: "123".to_string(),
            owner: None,
            photo_filename: None,
            account_id: None,
        };

        let result = request.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_create_user_valid() {
        let request = CreateUserRequest {
            email: "john@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            password: "SecurePassword123!".to_string(),
            owner: Some(false),
            photo_filename: None,
            account_id: Some(1),
        };

        let result = request.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_from_jsonapi() {
        let payload = json!({
            "data": {
                "type": "users",
                "attributes": {
                    "email": "john@example.com",
                    "first_name": "John",
                    "last_name": "Doe",
                    "password": "SecurePass123!"
                },
                "relationships": {
                    "account": {
                        "data": {"type": "accounts", "id": "1"}
                    }
                }
            }
        });

        let result = CreateUserRequest::deserialize_and_validate(
            &payload,
            "application/vnd.api+json",
            "users",
        );

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.email, "john@example.com");
        assert_eq!(request.account_id, Some(1));
    }
}
