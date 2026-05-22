use crate::app::{App, SessionLayerType};
use crate::middleware::{auth_middleware, clockwork_middleware, maintenance_middleware};
use appkit_core::middleware::{
    enhanced_security_headers_middleware, prepare_response_middleware, SecurityHeadersConfig,
};
use appkit_core::security::authenticator::AuthenticatorChain;
use appkit_core::security::rate_limit_middleware;
use appkit_core::security::{access_control_middleware, AccessControlConfig};
use axum::{middleware, Router};
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

pub struct HttpKernel {
    timeout: Duration,
    security_config: SecurityHeadersConfig,
    default_body_limit: usize,
    debug: bool,
    auth_chain: Option<Arc<AuthenticatorChain<crate::database::DieselUserRepository>>>,
    access_control_config: Option<Arc<AccessControlConfig>>,
}

impl HttpKernel {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            security_config: SecurityHeadersConfig::default(),
            default_body_limit: 10 * 1024 * 1024,
            debug: false,
            auth_chain: None,
            access_control_config: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_security_config(mut self, config: SecurityHeadersConfig) -> Self {
        self.security_config = config;
        self
    }

    pub fn with_body_limit(mut self, limit: usize) -> Self {
        self.default_body_limit = limit;
        self
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_auth_chain(
        mut self,
        auth_chain: Arc<AuthenticatorChain<crate::database::DieselUserRepository>>,
    ) -> Self {
        self.auth_chain = Some(auth_chain);
        self
    }

    pub fn with_access_control(mut self, config: Arc<AccessControlConfig>) -> Self {
        self.access_control_config = Some(config);
        self
    }

    pub fn apply(self, router: Router<App>, state: App, session_layer: SessionLayerType) -> Router {
        let router = self.apply_application_middleware(router, &state);

        let router = self.apply_session_layer(router, session_layer);

        let router = self.apply_infrastructure_layers(router, &state);

        router.with_state(state)
    }

    fn apply_application_middleware(&self, router: Router<App>, state: &App) -> Router<App> {
        let security_config = self.security_config.clone();
        let rate_limiter = state.rate_limiter();
        let state_clone = state.clone();

        let mut router = router
            .layer(RequestBodyLimitLayer::new(self.default_body_limit))
            .layer(middleware::from_fn(move |request, next| {
                let config = security_config.clone();
                async move { enhanced_security_headers_middleware(config, request, next).await }
            }))
            .layer(middleware::from_fn(move |request, next| {
                let limiter = rate_limiter.clone();
                async move { rate_limit_middleware(limiter, request, next).await }
            }))
            .layer(middleware::from_fn_with_state(
                state_clone,
                maintenance_middleware,
            ));

        if let Some(access_control_config) = &self.access_control_config {
            let config = access_control_config.clone();
            router = router.layer(middleware::from_fn(move |request, next| {
                let cfg = config.clone();
                async move { access_control_middleware(cfg, request, next).await }
            }));
        }

        if let Some(auth_chain) = &self.auth_chain {
            let auth_chain = auth_chain.clone();
            router = router.layer(middleware::from_fn_with_state(
                state.clone(),
                move |state, request, next| {
                    let chain = auth_chain.clone();
                    async move { auth_middleware(state, chain, request, next).await }
                },
            ));
        }

        router
    }

    fn apply_session_layer(
        &self,
        router: Router<App>,
        session_layer: SessionLayerType,
    ) -> Router<App> {
        match session_layer {
            SessionLayerType::Memory(layer) => router.layer(layer),
            SessionLayerType::Redis(layer) => router.layer(layer),
        }
    }

    fn apply_infrastructure_layers(&self, mut router: Router<App>, state: &App) -> Router<App> {
        use appkit_core::security::firewall::pattern::matches_pattern;
        use appkit_core::security::firewall::{FirewallConfig, FirewallRule};

        let firewall_configs = vec![
            FirewallConfig::new("/login", FirewallRule::Public),
            FirewallConfig::new("/register", FirewallRule::Public),
            FirewallConfig::new("/health", FirewallRule::Public),
            FirewallConfig::new("/_health", FirewallRule::Public),
            FirewallConfig::new("/__clockwork", FirewallRule::Public),
            FirewallConfig::new("/assets", FirewallRule::Public),
            FirewallConfig::new("/build", FirewallRule::Public),
            FirewallConfig::new("/images", FirewallRule::Public),
            FirewallConfig::new("/favicon.ico", FirewallRule::Public),
            FirewallConfig::new("/api/login", FirewallRule::Public),
            FirewallConfig::new("/api/register", FirewallRule::Public),
            FirewallConfig::new("/api", FirewallRule::ApiAuthenticated),
            FirewallConfig::new("/admin", FirewallRule::AdminOnly),
            FirewallConfig::new("/dashboard", FirewallRule::Authenticated),
            FirewallConfig::new("/users", FirewallRule::Authenticated),
            FirewallConfig::new("/contacts", FirewallRule::Authenticated),
            FirewallConfig::new("/organizations", FirewallRule::Authenticated),
            FirewallConfig::new("/products", FirewallRule::Authenticated),
            FirewallConfig::new("/customers", FirewallRule::Authenticated),
            FirewallConfig::new("/categories", FirewallRule::Authenticated),
            FirewallConfig::new("/brands", FirewallRule::Authenticated),
            FirewallConfig::new("/reports", FirewallRule::Authenticated),
            FirewallConfig::new("/settings", FirewallRule::Authenticated),
            FirewallConfig::new("/profile", FirewallRule::Authenticated),
            FirewallConfig::new("/upload", FirewallRule::Authenticated),
            FirewallConfig::new("/files", FirewallRule::Authenticated),
            FirewallConfig::new("/", FirewallRule::Authenticated),
        ];

        router = router
            .layer(CorsLayer::permissive())
            .layer(TimeoutLayer::new(self.timeout))
            .layer(TraceLayer::new_for_http())
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(CatchPanicLayer::new())
            .layer(middleware::from_fn(prepare_response_middleware))
            .layer(middleware::from_fn(
                move |mut req: axum::extract::Request, next: middleware::Next| {
                    let configs = firewall_configs.clone();
                    async move {
                        let path = req.uri().path().to_string();

                        let rule = configs
                            .iter()
                            .find(|config| matches_pattern(&config.pattern, &path))
                            .map(|config| config.rule.clone());

                        match rule {
                            Some(rule) => {
                                tracing::debug!("Firewall: {:?} - {}", rule, path);

                                req.extensions_mut().insert(rule.clone());
                                next.run(req).await
                            }
                            None => {
                                tracing::debug!(
                                    "Firewall: No rule for path, allowing through: {}",
                                    path
                                );
                                req.extensions_mut().insert(FirewallRule::Public);
                                next.run(req).await
                            }
                        }
                    }
                },
            ));

        if self.debug {
            router = router.layer(middleware::from_fn_with_state(
                state.clone(),
                clockwork_middleware,
            ));
        }

        router
    }

    #[allow(dead_code)]
    fn observability_layers(&self) -> impl tower::Layer<Router> + Clone {
        tower::ServiceBuilder::new()
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(PropagateRequestIdLayer::x_request_id())
            .layer(TraceLayer::new_for_http())
    }

    #[allow(dead_code)]
    fn reliability_layers(&self) -> impl tower::Layer<Router> + Clone {
        tower::ServiceBuilder::new()
            .layer(TimeoutLayer::new(self.timeout))
            .layer(CatchPanicLayer::new())
    }
}

impl Default for HttpKernel {
    fn default() -> Self {
        Self::new()
    }
}
