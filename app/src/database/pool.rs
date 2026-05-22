use anyhow::Result;
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, RunQueryDsl};
use diesel_async::{
    pooled_connection::deadpool::Pool, pooled_connection::AsyncDieselConnectionManager,
    sync_connection_wrapper::SyncConnectionWrapper,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

pub type DbConnection = SyncConnectionWrapper<SqliteConnection>;

pub type DbPool = Pool<DbConnection>;

pub fn run_migrations(database_url: &str, migrations: EmbeddedMigrations) -> Result<()> {
    tracing::info!("Running database migrations...");

    let mut connection = SqliteConnection::establish(database_url)?;

    diesel::sql_query("PRAGMA journal_mode = WAL;").execute(&mut connection)?;

    diesel::sql_query("PRAGMA busy_timeout = 30000;").execute(&mut connection)?;

    diesel::sql_query("PRAGMA synchronous = NORMAL;").execute(&mut connection)?;

    connection
        .run_pending_migrations(migrations)
        .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

    drop(connection);

    tracing::info!("✓ Database migrations completed");
    Ok(())
}

pub async fn establish_connection_pool(
    database_url: &str,
) -> Result<DbPool, diesel_async::pooled_connection::deadpool::BuildError> {
    let config =
        AsyncDieselConnectionManager::<SyncConnectionWrapper<SqliteConnection>>::new(database_url);
    let pool = Pool::builder(config).build()?;

    tracing::info!("Database connection pool established");

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_connection_pool() {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = establish_connection_pool(&database_url).await;
        assert!(pool.is_ok());
    }
}
