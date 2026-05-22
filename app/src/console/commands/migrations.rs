use crate::console::output::ConsoleOutput;
use anyhow::Result;
use chrono::Utc;
use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Args)]
pub struct MigrationGenerateCommand {
    pub name: String,

    #[arg(long, default_value = "migrations")]
    pub dir: String,
}

impl MigrationGenerateCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Generate Migration");

        let migrations_dir = Path::new(&self.dir);

        if !migrations_dir.exists() {
            fs::create_dir_all(migrations_dir)?;
            ConsoleOutput::info(format!("Created migrations directory: {}", self.dir));
        }

        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let migration_name = format!("{}_{}", timestamp, self.name);
        let migration_dir = migrations_dir.join(&migration_name);

        ConsoleOutput::task(format!("Creating migration: {}", migration_name));

        fs::create_dir(&migration_dir)?;

        let up_sql = migration_dir.join("up.sql");
        fs::write(&up_sql, "-- Your SQL goes here\n\n")?;

        let down_sql = migration_dir.join("down.sql");
        fs::write(
            &down_sql,
            "-- This file should undo anything in `up.sql`\n\n",
        )?;

        ConsoleOutput::done("Migration files created");
        ConsoleOutput::newline();

        ConsoleOutput::success("Migration generated successfully!");
        ConsoleOutput::list_item(format!("Directory: {}", migration_dir.display()));
        ConsoleOutput::list_item(format!("Up: {}", up_sql.display()));
        ConsoleOutput::list_item(format!("Down: {}", down_sql.display()));
        ConsoleOutput::newline();

        ConsoleOutput::info("Next steps:");
        ConsoleOutput::list_item(format!(
            "1. Edit {} to add your schema changes",
            up_sql.display()
        ));
        ConsoleOutput::list_item(format!(
            "2. Edit {} to add rollback logic",
            down_sql.display()
        ));
        ConsoleOutput::list_item("3. Run 'migrate' to apply the migration");
        ConsoleOutput::newline();

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct MigrationListCommand {
    #[arg(long, default_value = "migrations")]
    pub dir: String,

    #[arg(long, short = 'v')]
    pub verbose: bool,
}

impl MigrationListCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Available Migrations");

        let migrations_dir = Path::new(&self.dir);

        if !migrations_dir.exists() {
            ConsoleOutput::warning(format!("Migrations directory does not exist: {}", self.dir));
            return Ok(());
        }

        ConsoleOutput::task("Scanning migrations directory");

        let mut migrations: Vec<PathBuf> = fs::read_dir(migrations_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect();

        migrations.sort();

        ConsoleOutput::done(format!("Found {} migrations", migrations.len()));
        ConsoleOutput::newline();

        if migrations.is_empty() {
            ConsoleOutput::warning("No migrations found");
            ConsoleOutput::info("Run 'migration:generate <name>' to create one");
            return Ok(());
        }

        if self.verbose {
            for migration in migrations {
                let name = migration.file_name().unwrap().to_string_lossy();
                ConsoleOutput::info(format!("Migration: {}", name));
                ConsoleOutput::list_item(format!("Path: {}", migration.display()));

                let up_exists = migration.join("up.sql").exists();
                let down_exists = migration.join("down.sql").exists();

                ConsoleOutput::list_item(format!("up.sql: {}", if up_exists { "✓" } else { "✗" }));
                ConsoleOutput::list_item(format!(
                    "down.sql: {}",
                    if down_exists { "✓" } else { "✗" }
                ));
                ConsoleOutput::newline();
            }
        } else {
            let mut table = crate::console::Table::new(vec!["Migration", "Status"]);
            for migration in migrations {
                let name = migration.file_name().unwrap().to_string_lossy();
                let up_exists = migration.join("up.sql").exists();
                let down_exists = migration.join("down.sql").exists();

                let status = if up_exists && down_exists {
                    "Ready"
                } else if up_exists {
                    "No rollback"
                } else {
                    "Incomplete"
                };

                table.add_row(vec![name.to_string(), status.to_string()]);
            }
            table.render();
        }

        ConsoleOutput::newline();
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct MigrationStatusCommand {
    #[arg(long)]
    pub database_url: Option<String>,

    #[arg(long, default_value = "migrations")]
    pub dir: String,
}

impl MigrationStatusCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Migration Status");

        let database_url = self.database_url.clone().unwrap_or_else(|| {
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "appkit.db".to_string())
        });

        ConsoleOutput::info(format!("Database: {}", database_url));

        let migrations_dir = Path::new(&self.dir);

        if !migrations_dir.exists() {
            ConsoleOutput::warning(format!("Migrations directory does not exist: {}", self.dir));
            return Ok(());
        }

        let available_migrations: Vec<PathBuf> = fs::read_dir(migrations_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect();

        ConsoleOutput::list_item(format!(
            "Available migrations: {}",
            available_migrations.len()
        ));

        ConsoleOutput::task("Checking database");

        match crate::database::establish_connection_pool(&database_url).await {
            Ok(pool) => {
                let mut conn = pool.get().await?;
                ConsoleOutput::done("Database connection successful");

                use diesel_async::RunQueryDsl;

                let result =
                    diesel::sql_query("SELECT COUNT(*) as count FROM __diesel_schema_migrations")
                        .execute(&mut conn)
                        .await;

                match result {
                    Ok(_) => {
                        ConsoleOutput::list_item("Migrations table: ✓ exists");
                        ConsoleOutput::success("Database is ready for migrations");
                    }
                    Err(_) => {
                        ConsoleOutput::list_item("Migrations table: ✗ not found");
                        ConsoleOutput::warning("Run 'migrate' to initialize the database");
                    }
                }
            }
            Err(e) => {
                ConsoleOutput::error(format!("Cannot connect to database: {}", e));
                ConsoleOutput::info("Check your DATABASE_URL configuration");
            }
        }

        ConsoleOutput::newline();
        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct MigrationDiffCommand {
    pub name: String,

    #[arg(long)]
    pub database_url: Option<String>,

    #[arg(long, default_value = "migrations")]
    pub dir: String,
}

impl MigrationDiffCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Generate Migration Diff");

        ConsoleOutput::warning("Automatic schema diff generation is not yet implemented");
        ConsoleOutput::info("This would compare your Diesel schema with the current database");
        ConsoleOutput::newline();

        ConsoleOutput::info("For now, use 'migration:generate' to create a blank migration:");
        ConsoleOutput::list_item(format!(
            "cargo run --bin console -- migration:generate {}",
            self.name
        ));
        ConsoleOutput::newline();

        ConsoleOutput::info("Future implementation will:");
        ConsoleOutput::list_item("1. Read your diesel schema (schema.rs)");
        ConsoleOutput::list_item("2. Connect to the database and read current schema");
        ConsoleOutput::list_item("3. Generate SQL to transform database to match schema");
        ConsoleOutput::list_item("4. Create up.sql and down.sql automatically");
        ConsoleOutput::newline();

        Ok(())
    }
}
