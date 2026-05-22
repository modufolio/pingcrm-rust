use appkit_core::jsonapi::{DeserializeJsonApi, NormalizedPayload};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrganizationRequest {
    #[validate(length(min = 2, max = 100))]
    pub name: String,

    #[validate(email)]
    pub email: Option<String>,

    pub phone: Option<String>,

    pub address: Option<String>,

    pub city: Option<String>,

    pub region: Option<String>,

    pub country: Option<String>,

    pub postal_code: Option<String>,

    pub account_id: Option<i32>,
}

impl DeserializeJsonApi for CreateOrganizationRequest {
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

impl crate::database::ToNewModel<crate::database::models::NewOrganization>
    for CreateOrganizationRequest
{
    fn to_new_model(&self) -> crate::database::models::NewOrganization {
        use crate::database::models::NewOrganization;

        let mut new_org = NewOrganization::new(self.name.clone());

        new_org.email = self.email.clone();
        new_org.phone = self.phone.clone();
        new_org.address = self.address.clone();
        new_org.city = self.city.clone();
        new_org.region = self.region.clone();
        new_org.country = self.country.clone();
        new_org.postal_code = self.postal_code.clone();
        new_org.account_id = self.account_id.or(Some(1));

        new_org
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateOrganizationRequest {
    #[validate(length(min = 2, max = 100))]
    pub name: Option<String>,

    #[validate(email)]
    pub email: Option<Option<String>>,

    pub phone: Option<Option<String>>,

    pub address: Option<Option<String>>,

    pub city: Option<Option<String>>,

    pub region: Option<Option<String>>,

    pub country: Option<Option<String>>,

    pub postal_code: Option<Option<String>>,

    pub account_id: Option<Option<i32>>,
}

impl DeserializeJsonApi for UpdateOrganizationRequest {
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

impl crate::database::ToUpdateModel<crate::database::models::OrganizationUpdate>
    for UpdateOrganizationRequest
{
    fn to_update_model(&self) -> crate::database::models::OrganizationUpdate {
        use crate::database::models::OrganizationUpdate;
        use chrono::Utc;

        let mut update = OrganizationUpdate::new();
        update.updated_at = Utc::now().naive_utc();

        update.name = self.name.clone();
        update.email = self.email.clone().flatten();
        update.phone = self.phone.clone().flatten();
        update.address = self.address.clone().flatten();
        update.city = self.city.clone().flatten();
        update.region = self.region.clone().flatten();
        update.country = self.country.clone().flatten();
        update.postal_code = self.postal_code.clone().flatten();

        update
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_organization_validation() {
        let request = CreateOrganizationRequest {
            name: "A".to_string(),
            email: Some("invalid-email".to_string()),
            phone: None,
            address: None,
            city: None,
            region: None,
            country: None,
            postal_code: None,
            account_id: None,
        };

        let result = request.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_create_organization_valid() {
        let request = CreateOrganizationRequest {
            name: "Acme Corp".to_string(),
            email: Some("contact@acme.com".to_string()),
            phone: None,
            address: None,
            city: None,
            region: None,
            country: None,
            postal_code: None,
            account_id: None,
        };

        let result = request.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_deserialize_from_jsonapi() {
        let payload = json!({
            "data": {
                "type": "organizations",
                "attributes": {
                    "name": "Acme Corp",
                    "email": "contact@acme.com"
                }
            }
        });

        let result = CreateOrganizationRequest::deserialize_and_validate(
            &payload,
            "application/vnd.api+json",
            "organizations",
        );

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.name, "Acme Corp");
        assert_eq!(request.email, Some("contact@acme.com".to_string()));
    }
}
