use crate::app::App;

use crate::router::loader::{RouteInfo, RouteLoader};
use axum::routing::get_service;
use axum::Router;
use tower_http::services::ServeDir;

pub struct StaticFileRoutes {
    assets_dir: String,
    uploads_dir: String,
    build_dir: String,
}

impl StaticFileRoutes {
    pub fn new() -> Self {
        Self {
            assets_dir: "public/assets".to_string(),
            uploads_dir: "public/uploads".to_string(),
            build_dir: "public/build".to_string(),
        }
    }

    pub fn with_assets_dir(mut self, dir: impl Into<String>) -> Self {
        self.assets_dir = dir.into();
        self
    }

    pub fn with_uploads_dir(mut self, dir: impl Into<String>) -> Self {
        self.uploads_dir = dir.into();
        self
    }
}

impl Default for StaticFileRoutes {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteLoader<App> for StaticFileRoutes {
    fn load(&self) -> Router<App> {
        Router::new()
            .nest_service(
                "/assets",
                get_service(ServeDir::new(&self.assets_dir)).handle_error(|e| async move {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Static file error: {}", e),
                    )
                }),
            )
            .nest_service(
                "/uploads",
                get_service(ServeDir::new(&self.uploads_dir)).handle_error(|e| async move {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Static file error: {}", e),
                    )
                }),
            )
            .nest_service(
                "/build",
                get_service(ServeDir::new(&self.build_dir)).handle_error(|e| async move {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Static file error: {}", e),
                    )
                }),
            )
    }

    fn get_routes(&self) -> Vec<RouteInfo> {
        vec![
            RouteInfo::new("assets", "/assets/*", "GET"),
            RouteInfo::new("uploads", "/uploads/*", "GET"),
            RouteInfo::new("build", "/build/*", "GET"),
        ]
    }
}
