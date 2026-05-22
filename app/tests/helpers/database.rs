use app::AppFactory;
use axum_test::TestServer;
use std::sync::atomic::{AtomicU64, Ordering};

static DB_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct DatabaseTest {
    pub server: TestServer,
    db_id: u64,
}

impl DatabaseTest {
    pub async fn new() -> Self {
        let db_id = DB_COUNTER.fetch_add(1, Ordering::SeqCst);

        let db_path = format!("/tmp/test_db_{}.sqlite", db_id);

        if !std::path::Path::new(&db_path).exists() {
            std::fs::File::create(&db_path)
                .unwrap_or_else(|e| panic!("Failed to create database file {}: {}", db_path, e));
        }

        let db_url = format!("sqlite://{}", db_path);

        std::env::set_var("DATABASE_URL", &db_url);
        std::env::set_var("JWT_SECRET", "test-secret-key-for-testing-min-32-chars");
        std::env::set_var("DEBUG", "true");
        std::env::set_var("RUST_LOG", "debug");

        let app = AppFactory::create()
            .await
            .unwrap_or_else(|e| panic!("Failed to create test app: {}", e));

        let router_service = app.router();
        let router = (*router_service.router()).clone();

        let server = TestServer::new(router)
            .unwrap_or_else(|e| panic!("Failed to create test server: {}", e));

        Self { server, db_id }
    }

    pub fn db_path(&self) -> String {
        format!("/tmp/test_db_{}.sqlite", self.db_id)
    }

    pub fn cleanup(&self) {
        let path = self.db_path();
        if std::path::Path::new(&path).exists() {
            let _ = std::fs::remove_file(&path);
        }

        let _ = std::fs::remove_file(format!("{}-wal", path));
        let _ = std::fs::remove_file(format!("{}-shm", path));
    }

    #[allow(dead_code)]
    pub async fn truncate_all(&self) {}

    #[allow(dead_code)]
    pub async fn execute_sql(&self, _sql: &str) {}
}

impl Drop for DatabaseTest {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_isolation() {
        let db1 = DatabaseTest::new().await;
        let db2 = DatabaseTest::new().await;

        assert_ne!(db1.db_id, db2.db_id);

        assert_ne!(db1.db_path(), db2.db_path());
    }
}
