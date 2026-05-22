use super::pattern::matches_pattern;
use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use axum::Router;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirewallRule {
    Public,

    Authenticated,

    ApiAuthenticated,

    AdminOnly,
}

impl FirewallRule {
    pub fn requires_auth(&self) -> bool {
        !matches!(self, FirewallRule::Public)
    }

    pub fn is_stateless(&self) -> bool {
        matches!(self, FirewallRule::ApiAuthenticated)
    }

    pub fn name(&self) -> &'static str {
        match self {
            FirewallRule::Public => "public",
            FirewallRule::Authenticated => "main",
            FirewallRule::ApiAuthenticated => "api",
            FirewallRule::AdminOnly => "admin",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FirewallConfig {
    pub pattern: String,

    pub rule: FirewallRule,
}

impl FirewallConfig {
    pub fn new(pattern: impl Into<String>, rule: FirewallRule) -> Self {
        Self {
            pattern: pattern.into(),
            rule,
        }
    }
}

pub struct FirewallService {
    configs: Vec<FirewallConfig>,

    inner: Router,
}

impl FirewallService {
    pub fn new(configs: Vec<FirewallConfig>, inner: Router) -> Self {
        Self { configs, inner }
    }

    pub fn with_default_rules(inner: Router) -> Self {
        let configs = vec![
            FirewallConfig::new("/login", FirewallRule::Public),
            FirewallConfig::new("/register", FirewallRule::Public),
            FirewallConfig::new("/health", FirewallRule::Public),
            FirewallConfig::new("/_health", FirewallRule::Public),
            FirewallConfig::new("/__clockwork", FirewallRule::Public),
            FirewallConfig::new("/assets", FirewallRule::Public),
            FirewallConfig::new("/images", FirewallRule::Public),
            FirewallConfig::new("/favicon.ico", FirewallRule::Public),
            FirewallConfig::new("/api/login", FirewallRule::Public),
            FirewallConfig::new("/api/register", FirewallRule::Public),
            FirewallConfig::new("/api", FirewallRule::ApiAuthenticated),
            FirewallConfig::new("/admin", FirewallRule::AdminOnly),
            FirewallConfig::new("/", FirewallRule::Authenticated),
        ];

        Self::new(configs, inner)
    }

    #[allow(dead_code)]
    fn find_rule(&self, path: &str) -> Option<FirewallRule> {
        self.configs
            .iter()
            .find(|config| matches_pattern(&config.pattern, path))
            .map(|config| config.rule.clone())
    }

    pub fn into_make_service(self) -> axum::Router {
        use axum::middleware;

        let configs = self.configs.clone();
        self.inner.layer(middleware::from_fn(
            move |req: Request<Body>, next: middleware::Next| {
                let path = req.uri().path().to_string();
                let configs = configs.clone();

                async move {
                    let rule = configs
                        .iter()
                        .find(|config| matches_pattern(&config.pattern, &path))
                        .map(|config| config.rule.clone());

                    match rule {
                        Some(rule) => {
                            tracing::debug!("Firewall: {:?} - {}", rule, path);

                            let mut req = req;
                            req.extensions_mut().insert(rule.clone());

                            next.run(req).await
                        }
                        None => {
                            tracing::warn!("Firewall: NO RULE - {}", path);
                            Response::builder()
                                .status(StatusCode::FORBIDDEN)
                                .body(Body::empty())
                                .unwrap()
                                .into()
                        }
                    }
                }
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_matching() {
        let configs = vec![
            FirewallConfig::new("/api", FirewallRule::ApiAuthenticated),
            FirewallConfig::new("/admin", FirewallRule::AdminOnly),
            FirewallConfig::new("/", FirewallRule::Authenticated),
        ];

        let firewall = FirewallService::new(configs, Router::new());

        assert_eq!(
            firewall.find_rule("/api/users").unwrap(),
            FirewallRule::ApiAuthenticated
        );

        assert_eq!(
            firewall.find_rule("/admin/dashboard").unwrap(),
            FirewallRule::AdminOnly
        );

        assert_eq!(
            firewall.find_rule("/some/path").unwrap(),
            FirewallRule::Authenticated
        );
    }

    #[test]
    fn test_public_routes() {
        let configs = vec![
            FirewallConfig::new("/login", FirewallRule::Public),
            FirewallConfig::new("/__clockwork", FirewallRule::Public),
            FirewallConfig::new("/", FirewallRule::Authenticated),
        ];

        let firewall = FirewallService::new(configs, Router::new());

        assert_eq!(firewall.find_rule("/login").unwrap(), FirewallRule::Public);

        assert_eq!(
            firewall.find_rule("/__clockwork/data").unwrap(),
            FirewallRule::Public
        );
    }

    #[test]
    fn test_first_match_wins() {
        let configs = vec![
            FirewallConfig::new("/api/public", FirewallRule::Public),
            FirewallConfig::new("/api", FirewallRule::ApiAuthenticated),
        ];

        let firewall = FirewallService::new(configs, Router::new());

        assert_eq!(
            firewall.find_rule("/api/public/data").unwrap(),
            FirewallRule::Public
        );

        assert_eq!(
            firewall.find_rule("/api/users").unwrap(),
            FirewallRule::ApiAuthenticated
        );
    }
}
