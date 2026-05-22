use crate::database::models::{NewOrganization, Organization};
use chrono::Utc;
use fake::faker::address::en::{
    BuildingNumber, CityName, CountryName, PostCode, StateName, StreetName,
};
use fake::faker::company::en::CompanyName;
use fake::faker::internet::en::SafeEmail;
use fake::faker::phone_number::en::PhoneNumber;
use fake::Fake;

use super::factory_trait::Factory;

#[derive(Clone, Default)]
pub struct OrganizationFactory {
    name: Option<String>,
    email: Option<Option<String>>,
    phone: Option<Option<String>>,
    address: Option<Option<String>>,
    city: Option<Option<String>>,
    region: Option<Option<String>>,
    country: Option<Option<String>>,
    postal_code: Option<Option<String>>,
    account_id: Option<i32>,
}

impl OrganizationFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(Some(email.into()));
        self
    }

    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(Some(phone.into()));
        self
    }

    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(Some(address.into()));
        self
    }

    pub fn with_city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(Some(city.into()));
        self
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(Some(region.into()));
        self
    }

    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(Some(country.into()));
        self
    }

    pub fn with_postal_code(mut self, postal_code: impl Into<String>) -> Self {
        self.postal_code = Some(Some(postal_code.into()));
        self
    }

    pub fn with_account(mut self, account_id: impl Into<i32>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn without_email(mut self) -> Self {
        self.email = Some(None);
        self
    }

    pub fn without_phone(mut self) -> Self {
        self.phone = Some(None);
        self
    }
}

impl Factory for OrganizationFactory {
    type Model = Organization;
    type NewModel = NewOrganization;

    fn build(&self) -> NewOrganization {
        let now = Utc::now();

        let name = self.name.clone().unwrap_or_else(|| CompanyName().fake());

        let email = self
            .email
            .clone()
            .unwrap_or_else(|| Some(SafeEmail().fake()));

        let phone = self
            .phone
            .clone()
            .unwrap_or_else(|| Some(PhoneNumber().fake()));

        let address = self.address.clone().unwrap_or_else(|| {
            let number: String = BuildingNumber().fake();
            let street: String = StreetName().fake();
            Some(format!("{} {}", number, street))
        });

        let city = self.city.clone().unwrap_or_else(|| Some(CityName().fake()));

        let region = self
            .region
            .clone()
            .unwrap_or_else(|| Some(StateName().fake()));

        let country = self
            .country
            .clone()
            .unwrap_or_else(|| Some(CountryName().fake()));

        let postal_code = self
            .postal_code
            .clone()
            .unwrap_or_else(|| Some(PostCode().fake()));

        NewOrganization {
            name,
            email,
            phone,
            address,
            city,
            region,
            country,
            postal_code,
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
            deleted_at: None,
            account_id: self.account_id,
        }
    }

    fn table_name() -> &'static str {
        "organizations"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_organization_factory_build() {
        let factory = OrganizationFactory::default();
        let org = factory.build();

        assert!(!org.name.is_empty());
        assert!(org.email.is_some());
        assert!(org.phone.is_some());
    }

    #[test]
    fn test_organization_factory_with_custom_values() {
        let factory = OrganizationFactory::default()
            .with_name("Acme Corp")
            .with_email("contact@acme.com")
            .with_account(1);

        let org = factory.build();

        assert_eq!(org.name, "Acme Corp");
        assert_eq!(org.email, Some("contact@acme.com".to_string()));
        assert_eq!(org.account_id, Some(1));
    }

    #[test]
    fn test_organization_factory_without_fields() {
        let factory = OrganizationFactory::default()
            .with_name("Test Corp")
            .without_email()
            .without_phone();

        let org = factory.build();

        assert_eq!(org.name, "Test Corp");
        assert_eq!(org.email, None);
        assert_eq!(org.phone, None);
    }
}
