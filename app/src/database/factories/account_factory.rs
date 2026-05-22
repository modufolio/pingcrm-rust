use crate::database::models::{Account, NewAccount};
use chrono::Utc;
use fake::faker::company::en::CompanyName;
use fake::Fake;

use super::factory_trait::Factory;

#[derive(Clone, Default)]
pub struct AccountFactory {
    name: Option<String>,
}

impl AccountFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl Factory for AccountFactory {
    type Model = Account;
    type NewModel = NewAccount;

    fn build(&self) -> NewAccount {
        let now = Utc::now();

        let name = self.name.clone().unwrap_or_else(|| CompanyName().fake());

        NewAccount {
            name,
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
        }
    }

    fn table_name() -> &'static str {
        "accounts"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_factory_build() {
        let factory = AccountFactory::default();
        let account = factory.build();

        assert!(!account.name.is_empty());
    }

    #[test]
    fn test_account_factory_with_custom_values() {
        let factory = AccountFactory::default().with_name("Acme Corporation");

        let account = factory.build();

        assert_eq!(account.name, "Acme Corporation");
    }
}
