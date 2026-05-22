use crate::clockwork::DebugStack;
use crate::config::TusConfig;
use crate::database::repositories::audit_log_repository::AuditLogRepository;
use crate::database::DbPool as DieselPool;
use crate::listeners::*;
use crate::registry::V1Registry;
use crate::router::controllers::flat_file::MicrocontrollerRegistry;
use anyhow::Result;
use appkit_core::event::EventDispatcher;
use appkit_core::security::authenticator::{
    AuthenticatorChain, JwtAuthenticator, SessionAuthenticator,
};
use appkit_core::security::{AccessControlConfig, RateLimiter};
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use tokio::signal;
use tower_sessions::{MemoryStore, SessionManagerLayer, SessionStore};
use tower_sessions_redis_store::{
    fred::{
        interfaces::ClientLike,
        prelude::{Config as RedisConfig, Pool as RedisPool},
    },
    RedisStore,
};

pub use crate::database::DbPool;

#[derive(Clone)]
pub enum SessionStoreType {
    Memory(MemoryStore),
    Redis(RedisStore<RedisPool>),
}

pub enum SessionLayerType {
    Memory(SessionManagerLayer<MemoryStore>),
    Redis(SessionManagerLayer<RedisStore<RedisPool>>),
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

#[derive(Clone)]
pub struct App {
    pub config: Arc<AppConfig>,

    pub db_pool: DieselPool,

    session_store: Arc<dyn SessionStore>,

    session_store_type: SessionStoreType,

    secure_cookies: bool,

    pub event_dispatcher: Arc<EventDispatcher>,

    authenticator_chain:
        Arc<OnceLock<Arc<AuthenticatorChain<crate::database::DieselUserRepository>>>>,

    access_control_config: Arc<OnceLock<Arc<AccessControlConfig>>>,

    rate_limiter: Arc<OnceLock<Arc<RateLimiter>>>,

    router: Arc<OnceLock<Arc<dyn crate::router::RouterInterface>>>,

    pub debug_stack: DebugStack,

    pub microcontroller_registry: Option<Arc<MicrocontrollerRegistry>>,

    pub v1_registry: Option<Arc<V1Registry>>,
}

impl App {
    pub async fn new(diesel_pool: DieselPool, config: AppConfig) -> anyhow::Result<Self> {
        let (session_store_trait, session_store_type) = Self::create_session_store(&config).await?;

        let event_dispatcher = Arc::new(EventDispatcher::new());
        Self::register_event_listeners(&event_dispatcher, diesel_pool.clone(), &config).await;

        let secure_cookies = !config.debug;

        Ok(Self {
            config: Arc::new(config),

            db_pool: diesel_pool,
            session_store: session_store_trait,
            session_store_type,
            secure_cookies,
            event_dispatcher,

            authenticator_chain: Arc::new(OnceLock::new()),
            access_control_config: Arc::new(OnceLock::new()),
            rate_limiter: Arc::new(OnceLock::new()),
            router: Arc::new(OnceLock::new()),

            debug_stack: DebugStack::new(),
            microcontroller_registry: None,
            v1_registry: None,
        })
    }

    async fn create_session_store(
        config: &AppConfig,
    ) -> anyhow::Result<(Arc<dyn SessionStore>, SessionStoreType)> {
        if let Some(redis_url) = &config.redis_url {
            tracing::info!("Using Redis session store: {}", redis_url);

            let redis_config = RedisConfig::from_url(redis_url)?;
            let redis_pool = RedisPool::new(redis_config, None, None, None, 6)?;

            redis_pool.connect();
            redis_pool.wait_for_connect().await?;

            tracing::info!("✓ Connected to Redis for sessions");

            let store = RedisStore::new(redis_pool);
            let trait_obj: Arc<dyn SessionStore> = Arc::new(store.clone());
            let concrete = SessionStoreType::Redis(store);

            Ok((trait_obj, concrete))
        } else {
            if !config.debug {
                tracing::warn!("Using in-memory session store in production mode!");
                tracing::warn!("Sessions will be lost on server restart.");
                tracing::warn!("Set REDIS_URL environment variable for persistent sessions.");
            } else {
                tracing::info!("Using in-memory session store (development mode)");
            }

            let store = MemoryStore::default();
            let trait_obj: Arc<dyn SessionStore> = Arc::new(store.clone());
            let concrete = SessionStoreType::Memory(store);

            Ok((trait_obj, concrete))
        }
    }

    async fn register_event_listeners(
        dispatcher: &Arc<EventDispatcher>,
        diesel_pool: DieselPool,
        config: &AppConfig,
    ) {
        let audit_log_repo = Arc::new(AuditLogRepository::new(diesel_pool.clone()));

        let environment = if config.debug {
            "development".to_string()
        } else {
            "production".to_string()
        };

        tracing::info!("Registering user event listeners...");

        dispatcher.register(SendWelcomeEmailListener::new());
        dispatcher.register(CreateUserAuditLogListener::new(audit_log_repo.clone()));
        dispatcher.register(ClearUserCacheListener::new());
        dispatcher.register(NotifyAdminUserCreatedListener::new(environment));

        dispatcher.register(UpdateUserAuditLogListener::new(audit_log_repo.clone()));
        dispatcher.register(ClearUserCacheOnUpdateListener::new());

        dispatcher.register(DeleteUserAuditLogListener::new(audit_log_repo.clone()));
        dispatcher.register(ClearUserCacheOnDeleteListener::new());

        tracing::info!("Registering security event listeners...");

        dispatcher.register(TrackFailedLoginListener::new(audit_log_repo.clone()));

        dispatcher.register(NotifyPasswordChangeListener::new(audit_log_repo.clone()));

        dispatcher.register(Detect2FABypassListener::new(audit_log_repo.clone()));
        dispatcher.register(LogSuccessfulLoginListener::new(audit_log_repo.clone()));

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        tracing::info!("✓ Event listeners registered successfully");
    }

    pub fn authenticator_chain(
        &self,
    ) -> Arc<AuthenticatorChain<crate::database::DieselUserRepository>> {
        self.authenticator_chain
            .get_or_init(|| {
                let user_repo = crate::database::DieselUserRepository::new(self.db_pool.clone());

                Arc::new(
                    AuthenticatorChain::new()
                        .add_jwt(JwtAuthenticator::new(
                            self.config.jwt_secret.clone(),
                            user_repo.clone(),
                        ))
                        .add_session(SessionAuthenticator::new(user_repo)),
                )
            })
            .clone()
    }

    pub fn access_control_config(&self) -> Arc<AccessControlConfig> {
        self.access_control_config
            .get_or_init(|| Arc::new(AccessControlConfig::default_rules()))
            .clone()
    }

    pub fn rate_limiter(&self) -> Arc<RateLimiter> {
        self.rate_limiter
            .get_or_init(|| Arc::new(RateLimiter::new()))
            .clone()
    }

    pub fn router(&self) -> Arc<dyn crate::router::RouterInterface> {
        self.router
            .get_or_init(|| {
                let axum_router = crate::config::configure_routes(self);
                let router_service = crate::router::AppRouterService::new(axum_router);
                Arc::new(router_service) as Arc<dyn crate::router::RouterInterface>
            })
            .clone()
    }

    pub fn session_store(&self) -> Arc<dyn SessionStore> {
        self.session_store.clone()
    }

    pub fn event_dispatcher(&self) -> Arc<EventDispatcher> {
        self.event_dispatcher.clone()
    }

    pub fn db_pool(&self) -> &DieselPool {
        &self.db_pool
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

    pub fn create_session_layer(&self) -> SessionLayerType {
        match &self.session_store_type {
            SessionStoreType::Memory(store) => SessionLayerType::Memory(
                SessionManagerLayer::new(store.clone()).with_secure(self.secure_cookies),
            ),
            SessionStoreType::Redis(store) => SessionLayerType::Redis(
                SessionManagerLayer::new(store.clone()).with_secure(self.secure_cookies),
            ),
        }
    }

    pub async fn run(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let socket_addr: SocketAddr = addr.parse()?;

        tracing::info!("Starting server on {}", socket_addr);
        tracing::info!("Debug mode: {}", self.config.debug);
        tracing::info!("Maintenance mode: {}", self.config.maintenance_mode);

        let router_service = self.router();
        let router = (*router_service.router()).clone();

        let listener = tokio::net::TcpListener::bind(socket_addr).await?;

        tracing::info!("Server listening on {}", socket_addr);

        axum::serve(listener, router.into_make_service())
            .with_graceful_shutdown(Self::shutdown_signal())
            .await?;

        tracing::info!("Server shutdown complete");

        Ok(())
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!("Received Ctrl+C signal, initiating graceful shutdown...");
            },
            _ = terminate => {
                tracing::info!("Received SIGTERM signal, initiating graceful shutdown...");
            },
        }
    }
}

pub struct AppFactory;

impl AppFactory {
    pub async fn create() -> Result<App> {
        tracing::info!("Creating application...");

        let config = AppConfig::from_env()?;

        tracing::info!("Initializing Inertia templates...");
        appkit_core::inertia::init_templates()?;
        tracing::info!("Inertia templates initialized successfully");

        if config.database_url.contains(":memory:")
            || config.database_url.starts_with("sqlite://")
            || config.debug
        {
            tracing::info!("Running migrations for SQLite database...");
            crate::database::run_migrations(&config.database_url, crate::MIGRATIONS)?;
        }

        tracing::info!("Connecting to database...");
        let diesel_pool = crate::database::establish_connection_pool(&config.database_url).await?;
        tracing::info!("Database connected successfully");

        let app = App::new(diesel_pool, config).await?;

        tracing::info!("Application created successfully");

        Ok(app)
    }

    #[cfg(test)]
    pub async fn create_test() -> Result<App> {
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        std::env::set_var("DEBUG", "true");

        Self::create().await
    }
}
