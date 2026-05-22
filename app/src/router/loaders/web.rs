use crate::app::App;

use crate::router::controllers::{auth, dashboard, health, security};
use crate::router::loader::{RouteInfo, RouteLoader};
use axum::routing::get;
use axum::Router;

pub struct WebRoutes;

impl WebRoutes {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WebRoutes {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteLoader<App> for WebRoutes {
    fn load(&self) -> Router<App> {
        Router::new()
            .route("/health", get(health::health_check))
            .route("/login", get(security::login).post(security::form_login))
            .route("/logout", get(security::logout))
            .route("/register", get(security::register).post(auth::register))
            .route("/", get(dashboard::index))
    }

    fn get_routes(&self) -> Vec<RouteInfo> {
        vec![
            RouteInfo::new("health", "/health", "GET"),
            RouteInfo::new("auth.login", "/login", "GET,POST"),
            RouteInfo::new("auth.logout", "/logout", "GET"),
            RouteInfo::new("auth.register", "/register", "GET"),
            RouteInfo::new("home", "/", "GET"),
        ]
    }
}
