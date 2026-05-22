use crate::app::App;

use crate::clockwork::controller;
use crate::router::loader::{RouteInfo, RouteLoader};
use axum::routing::get;
use axum::Router;

pub struct ClockworkRoutes;

impl ClockworkRoutes {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClockworkRoutes {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteLoader<App> for ClockworkRoutes {
    fn load(&self) -> Router<App> {
        Router::new()
            .route("/__clockwork", get(controller::list_requests))
            .route("/__clockwork/latest", get(controller::get_latest))
            .route("/__clockwork/{id}", get(controller::get_request))
            .route("/__clockwork/{id}/next/{limit}", get(controller::get_next))
            .route(
                "/__clockwork/{id}/previous/{limit}",
                get(controller::get_previous),
            )
    }

    fn get_routes(&self) -> Vec<RouteInfo> {
        vec![
            RouteInfo::new("clockwork.list", "/__clockwork", "GET"),
            RouteInfo::new("clockwork.latest", "/__clockwork/latest", "GET"),
            RouteInfo::new("clockwork.show", "/__clockwork/{id}", "GET"),
            RouteInfo::new("clockwork.next", "/__clockwork/{id}/next/{limit}", "GET"),
            RouteInfo::new(
                "clockwork.previous",
                "/__clockwork/{id}/previous/{limit}",
                "GET",
            ),
        ]
    }
}
