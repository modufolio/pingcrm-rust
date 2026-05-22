pub mod account_fixtures;
pub mod app_fixtures;
pub mod audit_log_fixtures;
pub mod entity_fixtures;

pub mod user_fixtures;

use crate::database::schema;
use crate::seeder::EntityFactory;
use anyhow::{Context, Result};
use async_trait::async_trait;
use diesel_async::RunQueryDsl;

#[async_trait]
pub trait Fixture: Send + Sync {
    async fn load(&self, factory: &EntityFactory) -> Result<()>;

    fn name(&self) -> &str;

    fn description(&self) -> &str {
        ""
    }

    fn order(&self) -> i32 {
        0
    }

    fn enabled(&self) -> bool {
        true
    }
}

pub struct FixtureLoader {
    fixtures: Vec<Box<dyn Fixture>>,
}

impl FixtureLoader {
    pub fn new() -> Self {
        Self {
            fixtures: Vec::new(),
        }
    }

    pub fn add_fixture(&mut self, fixture: Box<dyn Fixture>) -> &mut Self {
        self.fixtures.push(fixture);
        self
    }

    pub fn add_fixtures(&mut self, fixtures: Vec<Box<dyn Fixture>>) -> &mut Self {
        for fixture in fixtures {
            self.fixtures.push(fixture);
        }
        self
    }

    pub async fn execute(&mut self, factory: &EntityFactory, purge: bool) -> Result<()> {
        if purge {
            println!("\nPurging database...\n");
            self.purge_database(factory).await?;
            println!("  ✓ Database purged successfully\n");
        }

        let mut enabled_fixtures: Vec<&Box<dyn Fixture>> =
            self.fixtures.iter().filter(|f| f.enabled()).collect();

        enabled_fixtures.sort_by_key(|f| f.order());

        println!("\nLoading {} fixture(s)...\n", enabled_fixtures.len());

        for fixture in enabled_fixtures {
            let name = fixture.name();
            let description = fixture.description();

            if !description.is_empty() {
                println!("Loading: {} - {}", name, description);
            } else {
                println!("Loading: {}", name);
            }

            Fixture::load(fixture.as_ref(), factory).await?;
            println!("  ✓ Loaded: {}\n", name);
        }

        Ok(())
    }

    pub fn count(&self) -> usize {
        self.fixtures.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.fixtures.iter().filter(|f| f.enabled()).count()
    }

    async fn purge_database(&self, factory: &EntityFactory) -> Result<()> {
        let mut conn = factory
            .pool()
            .get()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get database connection: {}", e))?;

        println!("Deleting order_items...");
        diesel::delete(schema::order_items::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete order_items")?;

        println!("Deleting orders...");
        diesel::delete(schema::orders::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete orders")?;

        println!("Deleting addresses...");
        diesel::delete(schema::addresses::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete addresses")?;

        println!("Deleting products...");
        diesel::delete(schema::products::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete products")?;

        println!("Deleting contacts...");
        diesel::delete(schema::contacts::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete contacts")?;

        println!("Deleting audit_logs...");
        diesel::delete(schema::audit_logs::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete audit_logs")?;

        println!("Deleting customers...");
        diesel::delete(schema::customers::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete customers")?;

        println!("Deleting brands...");
        diesel::delete(schema::brands::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete brands")?;

        println!("Deleting categories...");
        diesel::delete(schema::categories::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete categories")?;

        println!("Deleting organizations...");
        diesel::delete(schema::organizations::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete organizations")?;

        println!("Deleting users...");
        diesel::delete(schema::users::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete users")?;

        println!("Deleting accounts...");
        diesel::delete(schema::accounts::table)
            .execute(&mut conn)
            .await
            .context("Failed to delete accounts")?;

        Ok(())
    }
}

impl Default for FixtureLoader {
    fn default() -> Self {
        Self::new()
    }
}
