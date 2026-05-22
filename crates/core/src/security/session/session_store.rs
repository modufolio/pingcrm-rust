use tower_sessions::{session_store::Error as SessionError, SessionStore};
use tower_sessions_memory_store::MemoryStore;
use tower_sessions_redis_store::{
    fred::{
        interfaces::ClientLike,
        prelude::{Config as RedisConfig, Pool as RedisPool},
    },
    RedisStore,
};

#[derive(Debug, Clone, Default)]
pub enum SessionStoreConfig {
    #[default]
    Memory,

    Redis {
        url: String,

        prefix: String,

        ttl_seconds: u64,
    },
}

impl SessionStoreConfig {
    pub fn memory() -> Self {
        Self::Memory
    }

    pub fn redis(url: impl Into<String>) -> Self {
        Self::Redis {
            url: url.into(),
            prefix: "session:".to_string(),
            ttl_seconds: 3600,
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        if let Self::Redis {
            prefix: ref mut p, ..
        } = self
        {
            *p = prefix.into();
        }
        self
    }

    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        if let Self::Redis {
            ttl_seconds: ref mut ttl,
            ..
        } = self
        {
            *ttl = ttl_seconds;
        }
        self
    }
}

#[derive(Clone)]
pub enum AppSessionStore {
    Memory(MemoryStore),

    Redis(RedisStore<RedisPool>),
}

impl AppSessionStore {
    pub async fn from_config(config: SessionStoreConfig) -> Result<Self, SessionError> {
        match config {
            SessionStoreConfig::Memory => {
                tracing::info!("Initializing memory session store");
                Ok(Self::Memory(MemoryStore::default()))
            }

            SessionStoreConfig::Redis {
                url,
                prefix: _,
                ttl_seconds: _,
            } => {
                tracing::info!("Initializing Redis session store at {}", url);

                let redis_config = RedisConfig::from_url(&url).map_err(|e| {
                    SessionError::Backend(format!("Failed to parse Redis URL: {}", e))
                })?;

                let pool = RedisPool::new(redis_config, None, None, None, 6).map_err(|e| {
                    SessionError::Backend(format!("Failed to create Redis pool: {}", e))
                })?;

                pool.connect();
                pool.wait_for_connect().await.map_err(|e| {
                    SessionError::Backend(format!("Failed to connect to Redis: {}", e))
                })?;

                tracing::info!("Redis session store connected successfully");

                let store = RedisStore::new(pool);

                Ok(Self::Redis(store))
            }
        }
    }

    pub fn memory() -> Self {
        Self::Memory(MemoryStore::default())
    }

    pub fn into_inner(self) -> Box<dyn SessionStore> {
        match self {
            Self::Memory(store) => Box::new(store),
            Self::Redis(store) => Box::new(store),
        }
    }
}

pub struct SessionStoreBuilder {
    config: SessionStoreConfig,
}

impl Default for SessionStoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStoreBuilder {
    pub fn new() -> Self {
        Self {
            config: SessionStoreConfig::Memory,
        }
    }

    pub fn redis(mut self, url: impl Into<String>) -> Self {
        self.config = SessionStoreConfig::redis(url);
        self
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config = self.config.with_prefix(prefix);
        self
    }

    pub fn ttl(mut self, ttl_seconds: u64) -> Self {
        self.config = self.config.with_ttl(ttl_seconds);
        self
    }

    pub async fn build(self) -> Result<AppSessionStore, SessionError> {
        AppSessionStore::from_config(self.config).await
    }
}

pub struct SessionManager;

impl SessionManager {
    pub fn memory() -> AppSessionStore {
        AppSessionStore::memory()
    }

    pub async fn redis(url: impl Into<String>) -> Result<AppSessionStore, SessionError> {
        AppSessionStore::from_config(SessionStoreConfig::redis(url)).await
    }

    pub async fn from_env() -> Result<AppSessionStore, SessionError> {
        if let Ok(redis_url) =
            std::env::var("REDIS_URL").or_else(|_| std::env::var("SESSION_REDIS_URL"))
        {
            tracing::info!("Using Redis session store from environment");
            Self::redis(redis_url).await
        } else {
            tracing::warn!("No Redis URL found in environment, using memory session store");
            Ok(Self::memory())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_default() {
        let config = SessionStoreConfig::default();
        matches!(config, SessionStoreConfig::Memory);
    }

    #[test]
    fn test_session_config_redis() {
        let config = SessionStoreConfig::redis("redis://localhost:6379")
            .with_prefix("test:")
            .with_ttl(7200);

        match config {
            SessionStoreConfig::Redis {
                url,
                prefix,
                ttl_seconds,
            } => {
                assert_eq!(url, "redis://localhost:6379");
                assert_eq!(prefix, "test:");
                assert_eq!(ttl_seconds, 7200);
            }
            _ => panic!("Expected Redis config"),
        }
    }

    #[tokio::test]
    async fn test_memory_store_creation() {
        let store = SessionManager::memory();
        matches!(store, AppSessionStore::Memory(_));
    }

    #[test]
    fn test_builder_pattern() {
        let builder = SessionStoreBuilder::new()
            .redis("redis://localhost:6379")
            .prefix("app:")
            .ttl(3600);

        match builder.config {
            SessionStoreConfig::Redis {
                url,
                prefix,
                ttl_seconds,
            } => {
                assert_eq!(url, "redis://localhost:6379");
                assert_eq!(prefix, "app:");
                assert_eq!(ttl_seconds, 3600);
            }
            _ => panic!("Expected Redis config"),
        }
    }
}
