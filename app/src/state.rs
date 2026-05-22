use crate::clockwork::DebugStack;
use crate::config::TusConfig;
use crate::database::DbPool as DieselPool;
use crate::registry::V1Registry;
use crate::router::controllers::flat_file::MicrocontrollerRegistry;
use appkit_core::event::EventDispatcher;
use appkit_core::security::RateLimiter;
use std::sync::Arc;
use tower_sessions::SessionStore;

pub use crate::database::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DieselPool,

    pub session_store: Arc<dyn SessionStore>,

    pub config: Arc<AppConfig>,

    pub debug_stack: DebugStack,

    pub event_dispatcher: Arc<EventDispatcher>,

    pub rate_limiter: Arc<RateLimiter>,

    pub microcontroller_registry: Option<Arc<MicrocontrollerRegistry>>,

    pub v1_registry: Option<Arc<V1Registry>>,
}

impl AppState {
    pub fn new(
        db_pool: DieselPool,
        session_store: Arc<dyn SessionStore>,
        config: AppConfig,
        event_dispatcher: Arc<EventDispatcher>,
        rate_limiter: Arc<RateLimiter>,
    ) -> Self {
        Self {
            db_pool,
            session_store,
            config: Arc::new(config),
            debug_stack: DebugStack::new(),
            event_dispatcher,
            rate_limiter,
            microcontroller_registry: None,
            v1_registry: None,
        }
    }

    pub fn with_debug_stack(mut self, debug_stack: DebugStack) -> Self {
        self.debug_stack = debug_stack;
        self
    }

    pub fn with_microcontroller_registry(mut self, registry: MicrocontrollerRegistry) -> Self {
        self.microcontroller_registry = Some(Arc::new(registry));
        self
    }

    pub fn with_v1_registry(mut self, registry: V1Registry) -> Self {
        self.v1_registry = Some(Arc::new(registry));
        self
    }
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub host: String,

    pub port: u16,

    pub database_url: String,

    pub jwt_secret: String,

    pub maintenance_mode: bool,

    pub maintenance_bypass_token: Option<String>,

    pub debug: bool,

    pub session_lifetime: i64,

    pub cors_origins: Vec<String>,

    pub redis_url: Option<String>,

    pub tus: TusConfig,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let tus = TusConfig::load()?;

        let debug = std::env::var("DEBUG")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| {
            anyhow::anyhow!(
                "JWT_SECRET environment variable is required.\n\
                    For development, set: export JWT_SECRET=\"dev-secret-key\"\n\
                    For production, use a strong random secret: openssl rand -base64 32"
            )
        })?;

        if !debug && jwt_secret.len() < 32 {
            anyhow::bail!(
                "JWT_SECRET must be at least 32 characters in production mode.\n\
                Current length: {}. Generate a secure secret: openssl rand -base64 32",
                jwt_secret.len()
            );
        }

        let cors_origins_str = std::env::var("CORS_ORIGINS").unwrap_or_else(|_| {
            if debug {
                "http://localhost:3000,http://localhost:5173".to_string()
            } else {
                String::new()
            }
        });

        let cors_origins: Vec<String> = if cors_origins_str.is_empty() {
            Vec::new()
        } else {
            cors_origins_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        };

        if !debug && cors_origins.iter().any(|origin| origin == "*") {
            anyhow::bail!(
                "CORS_ORIGINS=\"*\" is forbidden in production when using sessions.\n\
                The W3C CORS specification does not allow Access-Control-Allow-Origin: * with credentials.\n\
                Set explicit origins: export CORS_ORIGINS=\"https://example.com,https://app.example.com\""
            );
        }

        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/appkit".to_string()),
            jwt_secret,
            maintenance_mode: std::env::var("MAINTENANCE_MODE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            maintenance_bypass_token: std::env::var("MAINTENANCE_BYPASS_TOKEN").ok(),
            debug,
            session_lifetime: std::env::var("SESSION_LIFETIME")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
            cors_origins,
            redis_url: std::env::var("REDIS_URL").ok(),
            tus,
        })
    }
}
