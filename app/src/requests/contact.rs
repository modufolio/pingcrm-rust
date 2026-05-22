use appkit_core::error::AppResult;
use appkit_core::jsonapi::{DeserializeJsonApi, NormalizedPayload};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateContactRequest {
    #[validate(length(min = 1, max = 50))]
    pub first_name: String,

    #[validate(length(min = 1, max = 50))]
    pub last_name: String,

    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(max = 20))]
    pub phone: Option<String>,

    #[validate(length(max = 100))]
    pub address: Option<String>,

    #[validate(length(max = 50))]
    pub city: Option<String>,

    #[validate(length(max = 50))]
    pub region: Option<String>,

    #[validate(length(max = 2))]
    pub country: Option<String>,

    #[validate(length(max = 10))]
    pub postal_code: Option<String>,

    #[serde(skip)]
    pub organization_id: Option<i32>,

    #[serde(skip)]
    pub account_id: Option<i32>,
}

impl DeserializeJsonApi for CreateContactRequest {
    fn from_normalized(payload: &NormalizedPayload) -> AppResult<Self> {
        let mut request: Self =
            serde_json::from_value(payload.attributes.clone()).map_err(|e| {
                appkit_core::error::AppError::BadRequest(format!(
                    "Failed to deserialize contact attributes: {}",
                    e
                ))
            })?;

        if let Some(org_rel) = payload.relationship("organization") {
            request.organization_id = org_rel.as_one();
        }

        if let Some(account_rel) = payload.relationship("account") {
            request.account_id = account_rel.as_one();
        }

        Ok(request)
    }
}

impl crate::database::ToNewModel<crate::database::models::NewContact> for CreateContactRequest {
    fn to_new_model(&self) -> crate::database::models::NewContact {
        use crate::database::models::NewContact;

        let mut new_contact = NewContact::new(self.first_name.clone(), self.last_name.clone());

        new_contact.email = self.email.clone();
        new_contact.phone = self.phone.clone();
        new_contact.address = self.address.clone();
        new_contact.city = self.city.clone();
        new_contact.region = self.region.clone();
        new_contact.country = self.country.clone();
        new_contact.postal_code = self.postal_code.clone();
        new_contact.organization_id = self.organization_id;
        new_contact.account_id = self.account_id.or(Some(1));

        new_contact
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateContactRequest {
    #[validate(length(min = 1, max = 50))]
    pub first_name: Option<String>,

    #[validate(length(min = 1, max = 50))]
    pub last_name: Option<String>,

    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(max = 20))]
    pub phone: Option<String>,

    #[validate(length(max = 100))]
    pub address: Option<String>,

    #[validate(length(max = 50))]
    pub city: Option<String>,

    #[validate(length(max = 50))]
    pub region: Option<String>,

    #[validate(length(max = 2))]
    pub country: Option<String>,

    #[validate(length(max = 10))]
    pub postal_code: Option<String>,

    #[serde(skip)]
    pub organization_id: Option<Option<i32>>,

    #[serde(skip)]
    pub account_id: Option<Option<i32>>,
}

impl DeserializeJsonApi for UpdateContactRequest {
    fn from_normalized(payload: &NormalizedPayload) -> AppResult<Self> {
        let mut request: Self =
            serde_json::from_value(payload.attributes.clone()).map_err(|e| {
                appkit_core::error::AppError::BadRequest(format!(
                    "Failed to deserialize contact attributes: {}",
                    e
                ))
            })?;

        if let Some(org_rel) = payload.relationship("organization") {
            if org_rel.is_null() {
                request.organization_id = Some(None);
            } else {
                request.organization_id = Some(org_rel.as_one());
            }
        }

        if let Some(account_rel) = payload.relationship("account") {
            if account_rel.is_null() {
                request.account_id = Some(None);
            } else {
                request.account_id = Some(account_rel.as_one());
            }
        }

        Ok(request)
    }
}

impl crate::database::ToUpdateModel<crate::database::models::ContactUpdate>
    for UpdateContactRequest
{
    fn to_update_model(&self) -> crate::database::models::ContactUpdate {
        use crate::database::models::ContactUpdate;
        use chrono::Utc;

        let mut update = ContactUpdate::new();
        update.updated_at = Utc::now().naive_utc();

        update.first_name = self.first_name.clone();
        update.last_name = self.last_name.clone();
        update.email = self.email.clone();
        update.phone = self.phone.clone();
        update.address = self.address.clone();
        update.city = self.city.clone();
        update.region = self.region.clone();
        update.country = self.country.clone();
        update.postal_code = self.postal_code.clone();

        if let Some(org_id) = self.organization_id {
            update.organization_id = org_id;
        }

        update
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_contact_valid_jsonapi() {
        let payload = json!({
            "data": {
                "type": "contacts",
                "attributes": {
                    "first_name": "John",
                    "last_name": "Doe",
                    "email": "john@example.com",
                    "phone": "+1-555-0100",
                    "city": "New York"
                },
                "relationships": {
                    "organization": {
                        "data": {"type": "organizations", "id": "5"}
                    }
                }
            }
        });

        let result = CreateContactRequest::deserialize_and_validate(
            &payload,
            "application/vnd.api+json",
            "contacts",
        );

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.first_name, "John");
        assert_eq!(request.last_name, "Doe");
        assert_eq!(request.email, Some("john@example.com".to_string()));
        assert_eq!(request.organization_id, Some(5));
    }

    #[test]
    fn test_create_contact_plain_json() {
        let payload = json!({
            "first_name": "Jane",
            "last_name": "Smith",
            "email": "jane@example.com",
            "organization_id": 10
        });

        let result =
            CreateContactRequest::deserialize_and_validate(&payload, "application/json", "");

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.first_name, "Jane");
        assert_eq!(request.last_name, "Smith");
        assert_eq!(request.email, Some("jane@example.com".to_string()));
        assert_eq!(request.organization_id, Some(10));
    }

    #[test]
    fn test_create_contact_validation_errors() {
        let payload = json!({
            "data": {
                "type": "contacts",
                "attributes": {
                    "first_name": "",
                    "last_name": "D",
                    "email": "not-an-email",
                    "country": "USA"
                }
            }
        });

        let result = CreateContactRequest::deserialize_and_validate(
            &payload,
            "application/vnd.api+json",
            "contacts",
        );

        assert!(result.is_err());
        let errors = result.unwrap_err();

        assert!(errors.len() >= 3);

        for error in &errors {
            assert_eq!(error.status, Some("422".to_string()));
        }
    }

    #[test]
    fn test_update_contact_partial_update() {
        let payload = json!({
            "data": {
                "type": "contacts",
                "id": "123",
                "attributes": {
                    "email": "newemail@example.com",
                    "city": "San Francisco"
                }
            }
        });

        let result = UpdateContactRequest::deserialize_and_validate(
            &payload,
            "application/vnd.api+json",
            "contacts",
        );

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.first_name, None);
        assert_eq!(request.last_name, None);
        assert_eq!(request.email, Some("newemail@example.com".to_string()));
        assert_eq!(request.city, Some("San Francisco".to_string()));
    }

    #[test]
    fn test_update_contact_clear_relationship() {
        let payload = json!({
            "data": {
                "type": "contacts",
                "id": "123",
                "attributes": {
                    "first_name": "John"
                },
                "relationships": {
                    "organization": {
                        "data": null
                    }
                }
            }
        });

        let result = UpdateContactRequest::deserialize_and_validate(
            &payload,
            "application/vnd.api+json",
            "contacts",
        );

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.organization_id, Some(None));
    }

    #[test]
    fn test_update_contact_validation_error() {
        let payload = json!({
            "data": {
                "type": "contacts",
                "id": "123",
                "attributes": {
                    "email": "invalid-email"
                }
            }
        });

        let result = UpdateContactRequest::deserialize_and_validate(
            &payload,
            "application/vnd.api+json",
            "contacts",
        );

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 1);

        let email_error = errors.iter().find(|e| {
            e.source
                .as_ref()
                .and_then(|s| s.pointer.as_ref())
                .map(|p| p.contains("email"))
                .unwrap_or(false)
        });
        assert!(email_error.is_some());
    }
}
