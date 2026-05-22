use crate::database::models::{NewUser, User};
use chrono::Utc;
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::{FirstName, LastName};
use fake::Fake;

use super::factory_trait::Factory;

#[derive(Clone, Default)]
pub struct UserFactory {
    email: Option<String>,
    password: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    roles: Option<Vec<String>>,
    account_status: Option<String>,
    account_id: Option<i32>,
    owner: Option<bool>,
    enabled: Option<bool>,
}

impl UserFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        let hash = appkit_core::security::hash_password(&password.into())
            .expect("Failed to hash password with argon2id");
        self.password = Some(hash);
        self
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
            self.last_name = Some(String::new());
        }
        self
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = Some(roles);
        self
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.roles = Some(vec![role]);
        self
    }

    pub fn with_two_factor(self, _secret: String) -> Self {
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.account_status = Some(status.into());
        self
    }

    pub fn with_account(mut self, account_id: impl Into<i32>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn with_owner(mut self, owner: bool) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }
}

impl Factory for UserFactory {
    type Model = User;
    type NewModel = NewUser;

    fn build(&self) -> NewUser {
        let now = Utc::now();

        let email = self.email.clone().unwrap_or_else(|| SafeEmail().fake());

        let password = self.password.clone().unwrap_or_else(|| {
            appkit_core::security::hash_password("password")
                .expect("Failed to hash default password with argon2id")
        });

        let first_name = self
            .first_name
            .clone()
            .unwrap_or_else(|| FirstName().fake());

        let last_name = self.last_name.clone().unwrap_or_else(|| LastName().fake());

        let roles = self
            .roles
            .clone()
            .unwrap_or_else(|| vec!["ROLE_USER".to_string()]);

        let account_status = self
            .account_status
            .clone()
            .unwrap_or_else(|| "active".to_string());

        NewUser {
            email,
            first_name,
            last_name,
            password,
            password_version: 1,
            owner: self.owner.unwrap_or(false),
            photo_filename: None,
            roles: Some(serde_json::to_string(&roles).unwrap_or_default()),
            created_at: now.naive_utc(),
            updated_at: now.naive_utc(),
            deleted_at: None,
            account_expires_at: None,
            credentials_expire_at: None,
            account_status,
            enabled: self.enabled.unwrap_or(true),
            locked: false,
            locked_at: None,
            locked_reason: None,
            account_id: self.account_id,
        }
    }

    fn table_name() -> &'static str {
        "users"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_factory_build() {
        let factory = UserFactory::default();
        let user = factory.build();

        assert!(!user.email.is_empty());
        assert!(!user.first_name.is_empty());
        assert!(!user.last_name.is_empty());
        assert_eq!(user.account_status, "active");
        assert!(user.enabled);
    }

    #[test]
    fn test_user_factory_with_custom_values() {
        let factory = UserFactory::default()
            .with_email("test@example.com")
            .with_first_name("Test")
            .with_last_name("User")
            .with_roles(vec!["ROLE_ADMIN".to_string()]);

        let user = factory.build();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.first_name, "Test");
        assert_eq!(user.last_name, "User");
    }
}
