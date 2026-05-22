use anyhow::Result;
use app::database::{establish_connection_pool, run_migrations};
use app::fixtures::{
    account_fixtures::AccountFixtures, entity_fixtures::EntityFixtures,
    user_fixtures::UserFixtures, FixtureLoader,
};
use app::seeder::EntityFactory;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let purge = std::env::args().any(|arg| arg == "--purge");

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "appkit.db".to_string());

    println!("\nDatabase Seeding Tool\n");
    println!("Database: {}", database_url);
    println!("Purge: {}\n", if purge { "Yes" } else { "No" });

    println!("Running migrations...\n");
    run_migrations(&database_url, app::MIGRATIONS)?;
    println!("Migrations completed\n");

    let pool = establish_connection_pool(&database_url).await?;

    let factory = EntityFactory::new(pool);

    let mut loader = FixtureLoader::new();

    loader.add_fixture(Box::new(AccountFixtures::new()));
    loader.add_fixture(Box::new(UserFixtures::new()));
    loader.add_fixture(Box::new(EntityFixtures::new()));

    loader.execute(&factory, purge).await?;

    println!("\nDatabase seeded successfully!\n");

    Ok(())
}
