use crate::console::output::ConsoleOutput;
use anyhow::Result;
use clap::Args;
use std::path::Path;

#[derive(Debug, Args)]
pub struct SchemaCreateCommand {
    #[arg(long)]
    pub database_url: Option<String>,

    #[arg(long)]
    pub force: bool,
}

impl SchemaCreateCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Create Database Schema");

        let database_url = self.database_url.clone().unwrap_or_else(|| {
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "appkit.db".to_string())
        });

        ConsoleOutput::info(format!("Database: {}", database_url));

        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "prod".to_string());
        if env == "prod" && !self.force {
            ConsoleOutput::warning("Creating schema in production requires --force flag");
            return Ok(());
        }

        ConsoleOutput::task("Creating database schema");
        crate::database::run_migrations(&database_url, crate::MIGRATIONS)?;
        ConsoleOutput::done("Schema created");

        ConsoleOutput::success("Database schema has been created successfully!");

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct SchemaDropCommand {
    #[arg(long)]
    pub database_url: Option<String>,

    #[arg(long)]
    pub force: bool,

    #[arg(long)]
    pub no_interaction: bool,
}

impl SchemaDropCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Drop Database Schema");

        let database_url = self.database_url.clone().unwrap_or_else(|| {
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "appkit.db".to_string())
        });

        ConsoleOutput::warning(format!("This will delete ALL data in: {}", database_url));

        if !self.force && !self.no_interaction {
            let confirmed = ConsoleOutput::confirm(
                "This operation is DESTRUCTIVE and will delete all data. Continue?",
                false,
            );
            if !confirmed {
                ConsoleOutput::info("Schema drop cancelled");
                return Ok(());
            }
        }

        ConsoleOutput::task("Dropping database schema");

        if database_url.ends_with(".db") || database_url == ":memory:" {
            if database_url != ":memory:" && Path::new(&database_url).exists() {
                std::fs::remove_file(&database_url)?;
                ConsoleOutput::done("Database file deleted");
            } else {
                ConsoleOutput::warning("Database file does not exist");
            }
        } else {
            ConsoleOutput::warning("Full schema drop for PostgreSQL/MySQL not yet implemented");
            ConsoleOutput::info("Please use your database client to drop the schema");
        }

        ConsoleOutput::success("Database schema has been dropped!");

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct SchemaValidateCommand {
    #[arg(long)]
    pub database_url: Option<String>,
}

impl SchemaValidateCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Validate Database Schema");

        let database_url = self.database_url.clone().unwrap_or_else(|| {
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "appkit.db".to_string())
        });

        ConsoleOutput::info(format!("Database: {}", database_url));
        ConsoleOutput::task("Validating schema");

        if database_url.ends_with(".db") && database_url != ":memory:" {
            if !Path::new(&database_url).exists() {
                ConsoleOutput::error("Database file does not exist");
                ConsoleOutput::info("Run 'migrate' to create the schema");
                return Ok(());
            }
        }

        let pool = crate::database::establish_connection_pool(&database_url).await?;
        let mut conn = pool.get().await?;

        ConsoleOutput::done("Database connection successful");

        use diesel_async::RunQueryDsl;

        let result = diesel::sql_query("SELECT name FROM sqlite_master WHERE type='table' AND name='__diesel_schema_migrations'")
            .execute(&mut conn)
            .await;

        match result {
            Ok(count) if count > 0 => {
                ConsoleOutput::done("Migrations table exists");
                ConsoleOutput::success("Schema validation passed!");
            }
            _ => {
                ConsoleOutput::warning("Migrations table not found");
                ConsoleOutput::info("Run 'migrate' to initialize the database");
            }
        }

        Ok(())
    }
}

#[derive(Debug, Args)]
pub struct DatabaseInfoCommand {
    #[arg(long)]
    pub database_url: Option<String>,
}

impl DatabaseInfoCommand {
    pub async fn execute(&self) -> Result<()> {
        ConsoleOutput::section("Database Information");

        let database_url = self.database_url.clone().unwrap_or_else(|| {
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "appkit.db".to_string())
        });

        ConsoleOutput::info(format!("Database URL: {}", database_url));

        let db_type = if database_url.starts_with("postgres://") {
            "PostgreSQL"
        } else if database_url.starts_with("mysql://") {
            "MySQL"
        } else {
            "SQLite"
        };

        ConsoleOutput::list_item(format!("Type: {}", db_type));

        if database_url.ends_with(".db") && database_url != ":memory:" {
            if let Ok(metadata) = std::fs::metadata(&database_url) {
                ConsoleOutput::list_item(format!("Size: {} bytes", metadata.len()));
                ConsoleOutput::list_item(format!("Modified: {:?}", metadata.modified()?));
            }
        }

        ConsoleOutput::task("Connecting to database");
        let pool = crate::database::establish_connection_pool(&database_url).await?;
        let _conn = pool.get().await?;
        ConsoleOutput::done("Connected");

        ConsoleOutput::list_item("Tables: Use 'query:tables' to see all tables");
        ConsoleOutput::newline();

        Ok(())
    }
}
