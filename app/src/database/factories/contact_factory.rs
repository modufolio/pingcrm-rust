use crate::database::models::{Contact, NewContact};
use chrono::Utc;
use fake::faker::address::en::{
    BuildingNumber, CityName, CountryName, PostCode, StateName, StreetName,
};
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::{FirstName, LastName};
use fake::faker::phone_number::en::PhoneNumber;
use fake::Fake;

use super::factory_trait::Factory;

#[derive(Clone, Default)]
pub struct ContactFactory {
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<Option<String>>,
    phone: Option<Option<String>>,
    address: Option<Option<String>>,
    city: Option<Option<String>>,
    region: Option<Option<String>>,
    country: Option<Option<String>>,
    postal_code: Option<Option<String>>,
    account_id: Option<i32>,
    organization_id: Option<i32>,
}

impl ContactFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_first_name(mut self, first_name: impl Into<String>) -> Self {
        self.first_name = Some(first_name.into());
        self
    }

    pub fn with_last_name(mut self, last_name: impl Into<String>) -> Self {
        self.last_name = Some(last_name.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        let name_str = name.into();
        let parts: Vec<&str> = name_str.split_whitespace().collect();
        if parts.len() >= 2 {
            self.first_name = Some(parts[0].to_string());
            self.last_name = Some(parts[1..].join(" "));
        } else if !parts.is_empty() {
            self.first_name = Some(parts[0].to_string());
        }
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

    pub fn with_organization(mut self, organization_id: impl Into<i32>) -> Self {
        self.organization_id = Some(organization_id.into());
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

    pub fn without_address(mut self) -> Self {
        self.address = Some(None);
        self
    }
}

impl Factory for ContactFactory {
    type Model = Contact;
    type NewModel = NewContact;

    fn build(&self) -> NewContact {
        let now = Utc::now();

        let first_name = self
            .first_name
            .clone()
            .unwrap_or_else(|| FirstName().fake());

        let last_name = self.last_name.clone().unwrap_or_else(|| LastName().fake());

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

        NewContact {
            first_name,
            last_name,
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
            organization_id: self.organization_id,
        }
    }

    fn table_name() -> &'static str {
        "contacts"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_factory_build() {
        let factory = ContactFactory::default();
        let contact = factory.build();

        assert!(!contact.first_name.is_empty());
        assert!(!contact.last_name.is_empty());
        assert!(contact.email.is_some());
        assert!(contact.phone.is_some());
    }

    #[test]
    fn test_contact_factory_with_custom_values() {
        let factory = ContactFactory::default()
            .with_first_name("John")
            .with_last_name("Doe")
            .with_email("john.doe@example.com")
            .with_account(1)
            .with_organization(5);

        let contact = factory.build();

        assert_eq!(contact.first_name, "John");
        assert_eq!(contact.last_name, "Doe");
        assert_eq!(contact.email, Some("john.doe@example.com".to_string()));
        assert_eq!(contact.account_id, Some(1));
        assert_eq!(contact.organization_id, Some(5));
    }

    #[test]
    fn test_contact_factory_with_name() {
        let factory = ContactFactory::default().with_name("Jane Smith");

        let contact = factory.build();

        assert_eq!(contact.first_name, "Jane");
        assert_eq!(contact.last_name, "Smith");
    }

    #[test]
    fn test_contact_factory_without_fields() {
        let factory = ContactFactory::default()
            .with_name("Test User")
            .without_email()
            .without_phone()
            .without_address();

        let contact = factory.build();

        assert_eq!(contact.first_name, "Test");
        assert_eq!(contact.last_name, "User");
        assert_eq!(contact.email, None);
        assert_eq!(contact.phone, None);
        assert_eq!(contact.address, None);
    }
}
