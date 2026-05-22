use crate::console::commands::*;
use crate::console::environment::Environment;
use crate::console::output::ConsoleOutput;
use crate::database::establish_connection_pool;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "console",
    about = "AppKit Console - Comprehensive command-line interface",
    version,
    author,
    long_about = "A powerful command-line interface for managing your AppKit application.\n\
                  Inspired by Symfony Console and Laravel Artisan.\n\n\
                  Use 'console list' to see all available commands grouped by category.\n\n\
                  Available command groups:\n\
                  • General: list\n\
                  • User management: user:add, user:list\n\
                  • Database: migrate, db:seed, fixtures:load\n\
                  • Schema (Doctrine-style): schema:create, schema:drop, schema:validate, db:info\n\
                  • Migrations: migration:generate, migration:list, migration:status, migration:diff\n\
                  • Queries: query:sql, query:tables\n\
                  • Server: serve"
)]
pub struct ConsoleRunner {
    #[arg(long, short = 'e', global = true)]
    pub env: Option<String>,

    #[arg(long, global = true)]
    pub test: bool,

    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "list")]
    List(ListCommand),

    #[command(name = "user:add", alias = "add-user")]
    UserAdd(UserAddCommand),

    #[command(name = "user:list", alias = "list-users")]
    UserList(UserListCommand),

    #[command(name = "migrate", alias = "m")]
    Migrate(MigrateCommand),

    #[command(name = "db:seed", alias = "seed")]
    DbSeed(SeedCommand),

    #[command(name = "fixtures:load", alias = "fixtures")]
    FixturesLoad(FixturesCommand),

    #[command(name = "schema:create")]
    SchemaCreate(SchemaCreateCommand),

    #[command(name = "schema:drop")]
    SchemaDrop(SchemaDropCommand),

    #[command(name = "schema:validate")]
    SchemaValidate(SchemaValidateCommand),

    #[command(name = "database:info", alias = "db:info")]
    DatabaseInfo(DatabaseInfoCommand),

    #[command(name = "migration:generate", alias = "make:migration")]
    MigrationGenerate(MigrationGenerateCommand),

    #[command(name = "migration:list", alias = "migrations")]
    MigrationList(MigrationListCommand),

    #[command(name = "migration:status")]
    MigrationStatus(MigrationStatusCommand),

    #[command(name = "migration:diff")]
    MigrationDiff(MigrationDiffCommand),

    #[command(name = "query:sql", alias = "sql")]
    QuerySql(QuerySqlCommand),

    #[command(name = "query:tables", alias = "tables")]
    QueryTables(QueryTablesCommand),

    #[command(name = "serve", alias = "server")]
    Serve(ServeCommand),

    #[command(name = "router:debug", alias = "routes")]
    RouterDebug(RouterDebugCommand),
}

impl ConsoleRunner {
    pub async fn run() -> Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "console=info".into()),
            )
            .init();

        dotenv::dotenv().ok();

        let cli = Self::parse();

        let env = if cli.test {
            Environment::Test
        } else if let Some(env_str) = &cli.env {
            Environment::from_str(env_str).unwrap_or_else(|| {
                ConsoleOutput::warning(format!("Invalid environment '{}', using default", env_str));
                Environment::from_env()
            })
        } else {
            Environment::from_env()
        };

        env.set_current();

        if cli.verbose {
            ConsoleOutput::info(format!("Environment: {}", env));
            ConsoleOutput::newline();
        }

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            if env.is_test() {
                ":memory:".to_string()
            } else {
                "appkit.db".to_string()
            }
        });

        let result: Result<()> = match cli.command {
            Commands::List(cmd) => cmd.execute().await,

            Commands::UserAdd(cmd) => {
                let pool = establish_connection_pool(&database_url).await?;
                cmd.execute(pool).await
            }
            Commands::UserList(cmd) => {
                let pool = establish_connection_pool(&database_url).await?;
                cmd.execute(pool).await
            }

            Commands::Migrate(cmd) => cmd.execute(&database_url).await,
            Commands::DbSeed(cmd) => cmd.execute(&database_url).await,
            Commands::FixturesLoad(cmd) => cmd.execute(&database_url).await,

            Commands::SchemaCreate(cmd) => cmd.execute().await,
            Commands::SchemaDrop(cmd) => cmd.execute().await,
            Commands::SchemaValidate(cmd) => cmd.execute().await,
            Commands::DatabaseInfo(cmd) => cmd.execute().await,

            Commands::MigrationGenerate(cmd) => cmd.execute().await,
            Commands::MigrationList(cmd) => cmd.execute().await,
            Commands::MigrationStatus(cmd) => cmd.execute().await,
            Commands::MigrationDiff(cmd) => cmd.execute().await,

            Commands::QuerySql(cmd) => cmd.execute().await,
            Commands::QueryTables(cmd) => cmd.execute().await,

            Commands::Serve(cmd) => cmd.execute().await,

            Commands::RouterDebug(cmd) => cmd.execute().await,
        };

        if let Err(e) = result {
            ConsoleOutput::error(format!("{}", e));
            if cli.verbose {
                ConsoleOutput::error(format!("Stack trace: {:?}", e));
            }
            std::process::exit(1);
        }

        Ok(())
    }
}
