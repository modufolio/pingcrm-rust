use crate::database::models::Account;
use crate::database::schema;
use crate::fixtures::Fixture;
use crate::seeder::{EntityFactory, UserFactory};
use anyhow::Result;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

pub struct UserFixtures {
    pub create_random_users: bool,
    pub random_user_count: usize,
}

impl UserFixtures {
    pub fn new() -> Self {
        Self {
            create_random_users: true,
            random_user_count: 20,
        }
    }

    pub fn with_random_users(mut self, count: usize) -> Self {
        self.create_random_users = true;
        self.random_user_count = count;
        self
    }

    pub fn without_random_users(mut self) -> Self {
        self.create_random_users = false;
        self
    }
}

impl Default for UserFixtures {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fixture for UserFixtures {
    async fn load(&self, factory: &EntityFactory) -> Result<()> {
        let mut conn = factory.pool().get().await?;
        let account: Account = schema::accounts::table
            .order_by(schema::accounts::created_at.asc())
            .select(Account::as_select())
            .first(&mut conn)
            .await?;
        let account_id = account.id;
        drop(conn);

        let super_admin = factory
            .create_user(
                UserFactory::new()
                    .with_account(account_id)
                    .with_email("admin@example.com")
                    .with_name("Super Admin")
                    .with_password("admin123")
                    .with_role("ROLE_SUPER_ADMIN".to_string())
                    .with_status("active".to_string()),
            )
            .await?;
        println!(
            "    ✓ Super Admin: {} ({})",
            super_admin.full_name(),
            super_admin.email
        );

        let admin1 = factory
            .create_user(
                UserFactory::new()
                    .with_account(account_id)
                    .with_email("manager@example.com")
                    .with_name("Sarah Manager")
                    .with_password("password123")
                    .with_role("ROLE_ADMIN".to_string()),
            )
            .await?;
        println!("    ✓ Admin: {} ({})", admin1.full_name(), admin1.email);

        let admin2 = factory
            .create_user(
                UserFactory::new()
                    .with_account(account_id)
                    .with_email("admin.john@example.com")
                    .with_name("John Administrator")
                    .with_password("password123")
                    .with_role("ROLE_ADMIN".to_string()),
            )
            .await?;
        println!("    ✓ Admin: {} ({})", admin2.full_name(), admin2.email);

        let demo_user = factory
            .create_user(
                UserFactory::new()
                    .with_account(account_id)
                    .with_email("johndoe@example.com")
                    .with_name("John Doe")
                    .with_password("secret")
                    .with_role("ROLE_USER".to_string()),
            )
            .await?;
        println!(
            "    ✓ Demo: {} ({})",
            demo_user.full_name(),
            demo_user.email
        );

        let named_users = vec![
            ("jane.smith@example.com", "Jane Smith"),
            ("bob.wilson@example.com", "Bob Wilson"),
            ("alice.johnson@example.com", "Alice Johnson"),
            ("charlie.brown@example.com", "Charlie Brown"),
        ];

        for (email, name) in named_users {
            let user = factory
                .create_user(
                    UserFactory::new()
                        .with_account(account_id)
                        .with_email(email)
                        .with_name(name)
                        .with_password("password123")
                        .with_role("ROLE_USER".to_string()),
                )
                .await?;
            println!("    ✓ User: {} ({})", user.full_name(), user.email);
        }

        if self.create_random_users {
            println!("Creating {} random users...", self.random_user_count);
            for i in 0..self.random_user_count {
                let user_factory = if i % 5 == 0 {
                    UserFactory::new()
                        .with_account(account_id)
                        .with_role("ROLE_ADMIN".to_string())
                } else {
                    UserFactory::new().with_account(account_id)
                };
                let _user = factory.create_user(user_factory).await?;
            }
            println!("    ✓ Created {} random users", self.random_user_count);
        }

        let _disabled_user = factory
            .create_user(
                UserFactory::new()
                    .with_account(account_id)
                    .with_email("disabled@example.com")
                    .with_name("Disabled User")
                    .with_password("password123")
                    .with_status("disabled".to_string()),
            )
            .await?;
        println!("    ✓ Disabled User: disabled@example.com");

        let _locked_user = factory
            .create_user(
                UserFactory::new()
                    .with_account(account_id)
                    .with_email("locked@example.com")
                    .with_name("Locked User")
                    .with_password("password123")
                    .with_status("locked".to_string()),
            )
            .await?;
        println!("    ✓ Locked User: locked@example.com");

        let _twofa_user = factory
            .create_user(
                UserFactory::new()
                    .with_account(account_id)
                    .with_email("secure@example.com")
                    .with_name("Secure User")
                    .with_password("password123")
                    .with_two_factor("JBSWY3DPEHPK3PXP".to_string()),
            )
            .await?;
        println!("    ✓ 2FA User: secure@example.com");

        Ok(())
    }

    fn name(&self) -> &str {
        "UserFixtures"
    }

    fn description(&self) -> &str {
        "Create test users (admins, regular users, special users)"
    }

    fn order(&self) -> i32 {
        10
    }
}
