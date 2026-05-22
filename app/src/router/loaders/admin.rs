use crate::app::App;

use crate::router::controllers::users;
use crate::router::loader::{RouteInfo, RouteLoader};
use axum::routing::get;
use axum::Router;

pub struct AdminRoutes;

impl AdminRoutes {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AdminRoutes {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteLoader<App> for AdminRoutes {
    fn load(&self) -> Router<App> {
        Router::new().route("/admin/users", get(users::admin_list_users))
    }

    fn get_routes(&self) -> Vec<RouteInfo> {
        vec![RouteInfo::new("admin.users.list", "/admin/users", "GET")]
    }
}
