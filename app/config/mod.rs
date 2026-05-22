pub mod routes;

pub use routes::{
    configure_content_routes, configure_v1_resources, ContentRoutesConfig, V1JsonApiConfig,
};

use crate::app::App;
use crate::registry::V1Registry;
use crate::router::kernel::HttpKernel;
use crate::router::loader::{AppRouter, RouteLoader};
use crate::router::loaders::{
    AdminRoutes, ApiRoutes, AppRoutes, JsonApiRouteLoader, StaticFileRoutes, WebRoutes,
};
use axum::Router;

pub fn configure_routes(app: &App) -> Router {
    let auth_chain = app.authenticator_chain();
    let access_control_config = app.access_control_config();
    let v1_config = configure_v1_resources(app.db_pool.clone());
    let v1_registry = V1Registry::from_config(v1_config.handlers, v1_config.configurator.clone());
    let app_state = app.clone().with_v1_registry(v1_registry);

    let session_layer = app.create_session_layer();

    let router = AppRouter::new()
        .load(WebRoutes::new())
        .load(StaticFileRoutes::new())
        .load(ApiRoutes::new())
        .load(AdminRoutes::new())
        .load(AppRoutes::new())
        .nest(
            "/api/v1",
            JsonApiRouteLoader::new(v1_config.configurator).load(),
        )
        .fallback(crate::router::controllers::health::not_found)
        .build();

    HttpKernel::new()
        .with_debug(app_state.config.debug)
        .with_auth_chain(auth_chain)
        .with_access_control(access_control_config)
        .apply(router, app_state, session_layer)
}
