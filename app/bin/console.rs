use anyhow::Result;

use app::console::ConsoleRunner;

#[tokio::main]
async fn main() -> Result<()> {
    ConsoleRunner::run().await
}
