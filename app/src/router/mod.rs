pub mod app_router_service;
pub mod controllers;
pub mod decorators;
pub mod kernel;
pub mod loader;
pub mod loaders;
pub mod router_interface;

pub use app_router_service::AppRouterService;
pub use router_interface::RouterInterface;

use crate::app::App;
use axum::Router;

pub fn build_router(app: &App) -> Router {
    let router_service = app.router();

    (*router_service.router()).clone()
}
