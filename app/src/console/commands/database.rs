use crate::console::output::ConsoleOutput;
use crate::database::{establish_connection_pool, run_migrations};
use crate::fixtures::{app_fixtures::AppFixtures, user_fixtures::UserFixtures, FixtureLoader};
use crate::seeder::EntityFactory;
use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct MigrateCommand {
    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub force: bool,
}

impl MigrateCommand {
    pub async fn execute(&self, database_url: &str) -> Result<()> {
        ConsoleOutput::section("Database Migrations");

        if self.dry_run {
            ConsoleOutput::info("Running in dry-run mode (no changes will be made)");
        }

        ConsoleOutput::task("Running migrations");

        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "prod".to_string());
        if env == "prod" && !self.force {
            ConsoleOutput::warning("Running migrations in production requires --force flag");
            return Ok(());
        }

        run_migrations(database_url, crate::MIGRATIONS)?;

        ConsoleOutput::done("Migrations completed");
        ConsoleOutput::success("All migrations have been applied successfully!");

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct SeedCommand {
    #[arg(long)]
    pub purge: bool,

    #[arg(long)]
    pub no_interaction: bool,
}

impl SeedCommand {
    pub async fn execute(&self, database_url: &str) -> Result<()> {
        ConsoleOutput::section("Database Seeding");

        ConsoleOutput::info(format!("Database: {}", database_url));
        ConsoleOutput::info(format!("Purge: {}", if self.purge { "Yes" } else { "No" }));

        if self.purge && !self.no_interaction {
            let env = std::env::var("APP_ENV").unwrap_or_else(|_| "prod".to_string());
            if env == "prod" {
                let confirmed = ConsoleOutput::confirm(
                    "You are about to purge the database in PRODUCTION. Continue?",
                    false,
                );
                if !confirmed {
                    ConsoleOutput::warning("Seeding cancelled");
                    return Ok(());
                }
            }
        }

        ConsoleOutput::task("Running migrations");
        run_migrations(database_url, crate::MIGRATIONS)?;
        ConsoleOutput::done("Migrations completed");

        ConsoleOutput::task("Connecting to database");
        let pool = establish_connection_pool(database_url).await?;
        ConsoleOutput::done("Connected");

        let factory = EntityFactory::new(pool);

        let mut loader = FixtureLoader::new();

        loader.add_fixture(Box::new(AppFixtures::new()));

        ConsoleOutput::task("Loading seed data");
        loader.execute(&factory, self.purge).await?;
        ConsoleOutput::done("Seed data loaded");

        ConsoleOutput::success("Database seeded successfully!");

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct FixturesCommand {
    #[arg(long)]
    pub purge: bool,

    #[arg(long, short = 'f')]
    pub fixture: Option<String>,

    #[arg(long)]
    pub no_interaction: bool,
}

impl FixturesCommand {
    pub async fn execute(&self, database_url: &str) -> Result<()> {
        ConsoleOutput::section("Load Fixtures");

        ConsoleOutput::info(format!("Database: {}", database_url));
        ConsoleOutput::info(format!("Purge: {}", if self.purge { "Yes" } else { "No" }));

        if self.purge && !self.no_interaction {
            let confirmed =
                ConsoleOutput::confirm("This will delete all existing data. Continue?", false);
            if !confirmed {
                ConsoleOutput::warning("Loading fixtures cancelled");
                return Ok(());
            }
        }

        ConsoleOutput::task("Running migrations");
        run_migrations(database_url, crate::MIGRATIONS)?;
        ConsoleOutput::done("Migrations completed");

        ConsoleOutput::task("Connecting to database");
        let pool = establish_connection_pool(database_url).await?;
        ConsoleOutput::done("Connected");

        let factory = EntityFactory::new(pool);

        let mut loader = FixtureLoader::new();

        match self.fixture.as_deref() {
            Some("users") => {
                ConsoleOutput::info("Loading user fixtures");
                loader.add_fixture(Box::new(UserFixtures::new()));
            }
            Some("app") => {
                ConsoleOutput::info("Loading app fixtures");
                loader.add_fixture(Box::new(AppFixtures::new()));
            }
            Some("all") | None => {
                ConsoleOutput::info("Loading all fixtures");
                loader.add_fixture(Box::new(UserFixtures::new()));
                loader.add_fixture(Box::new(AppFixtures::new()));
            }
            Some(name) => {
                ConsoleOutput::error(format!("Unknown fixture: {}", name));
                ConsoleOutput::info("Available fixtures: users, app, all");
                return Ok(());
            }
        }

        ConsoleOutput::task("Loading fixtures");
        loader.execute(&factory, self.purge).await?;
        ConsoleOutput::done("Fixtures loaded");

        ConsoleOutput::success("Fixtures loaded successfully!");

        Ok(())
    }
}
