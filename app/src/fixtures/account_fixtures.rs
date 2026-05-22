use crate::database::pool::DbPool;
use crate::database::schema::accounts;

use crate::database::{Account, NewAccount};
use crate::fixtures::Fixture;
use crate::seeder::EntityFactory;
use anyhow::Result;
use async_trait::async_trait;
use diesel_async::RunQueryDsl;

pub struct AccountFixtures {
    pub create_demo_accounts: bool,
    pub demo_account_count: usize,
}

impl AccountFixtures {
    pub fn new() -> Self {
        Self {
            create_demo_accounts: true,
            demo_account_count: 5,
        }
    }

    pub fn with_demo_accounts(mut self, count: usize) -> Self {
        self.create_demo_accounts = true;
        self.demo_account_count = count;
        self
    }

    pub fn without_demo_accounts(mut self) -> Self {
        self.create_demo_accounts = false;
        self
    }

    async fn create_account(&self, pool: &DbPool, new_account: NewAccount) -> Result<Account> {
        let mut conn = pool.get().await?;

        let account = diesel::insert_into(accounts::table)
            .values(&new_account)
            .get_result::<Account>(&mut conn)
            .await?;

        Ok(account)
    }
}

impl Default for AccountFixtures {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fixture for AccountFixtures {
    async fn load(&self, factory: &EntityFactory) -> Result<()> {
        let pool = factory.pool();

        let acme = self
            .create_account(pool, NewAccount::new("Acme Corporation".to_string()))
            .await?;
        println!("    ✓ Account: {} ({})", acme.name, acme.id);

        let global = self
            .create_account(pool, NewAccount::new("Global Industries Inc.".to_string()))
            .await?;
        println!("    ✓ Account: {} ({})", global.name, global.id);

        if self.create_demo_accounts {
            println!("    Creating {} demo accounts...", self.demo_account_count);

            let demo_names = [
                "TechStart Solutions",
                "Creative Digital Agency",
                "Enterprise Systems Ltd",
                "Innovative Products Co",
                "Strategic Consulting Group",
                "Advanced Manufacturing",
                "Modern Services Inc",
                "Dynamic Solutions LLC",
            ];

            for (i, name) in demo_names.iter().take(self.demo_account_count).enumerate() {
                let account = self
                    .create_account(pool, NewAccount::new(name.to_string()))
                    .await?;

                if i < 3 {
                    println!("    ✓ Demo Account: {}", account.name);
                }
            }

            println!("    ✓ Created {} demo accounts", self.demo_account_count);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "AccountFixtures"
    }

    fn description(&self) -> &str {
        "Create test accounts for multi-tenant architecture"
    }

    fn order(&self) -> i32 {
        5
    }
}
