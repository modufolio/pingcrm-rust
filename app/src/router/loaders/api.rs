use crate::app::App;

use crate::router::controllers::{auth, users};
use crate::router::loader::{RouteInfo, RouteLoader};
use axum::routing::{get, post};
use axum::Router;

pub struct ApiRoutes;

impl ApiRoutes {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ApiRoutes {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteLoader<App> for ApiRoutes {
    fn load(&self) -> Router<App> {
        Router::new()
            .route("/api/login", post(auth::login))
            .route("/api/profile", get(users::get_profile))
            .route("/api/user", get(users::list_users))
            .route("/api/user/{id}", get(users::get_user_by_id))
    }

    fn get_routes(&self) -> Vec<RouteInfo> {
        vec![RouteInfo::new("api.profile", "/api/profile", "GET")]
    }
}
