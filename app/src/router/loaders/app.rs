use crate::app::App;

use crate::router::controllers::{
    contacts_pages, organizations_pages, placeholder_pages, security, users_pages,
};
use crate::router::loader::{RouteInfo, RouteLoader};
use axum::routing::{get, post, put};
use axum::Router;

pub struct AppRoutes;

impl AppRoutes {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AppRoutes {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteLoader<App> for AppRoutes {
    fn load(&self) -> Router<App> {
        Router::new()
            .route("/logout", post(security::logout))
            .route("/users", get(users_pages::index).post(users_pages::store))
            .route("/users/create", get(users_pages::create))
            .route("/users/{id}/edit", get(users_pages::edit))
            .route(
                "/users/{id}",
                post(users_pages::update).delete(users_pages::destroy),
            )
            .route("/users/{id}/restore", put(users_pages::restore))
            .route(
                "/contacts",
                get(contacts_pages::index).post(contacts_pages::store),
            )
            .route("/contacts/create", get(contacts_pages::create))
            .route("/contacts/{id}/edit", get(contacts_pages::edit))
            .route(
                "/contacts/{id}",
                put(contacts_pages::update).delete(contacts_pages::destroy),
            )
            .route("/contacts/{id}/restore", put(contacts_pages::restore))
            .route(
                "/organizations",
                get(organizations_pages::index).post(organizations_pages::store),
            )
            .route("/organizations/create", get(organizations_pages::create))
            .route("/organizations/{id}/edit", get(organizations_pages::edit))
            .route(
                "/organizations/{id}",
                put(organizations_pages::update).delete(organizations_pages::destroy),
            )
            .route(
                "/organizations/{id}/restore",
                put(organizations_pages::restore),
            )
            .route("/orders", get(placeholder_pages::orders_index))
            .route("/reports", get(placeholder_pages::reports_index))
            .route("/upload", get(placeholder_pages::upload_index))
    }

    fn get_routes(&self) -> Vec<RouteInfo> {
        vec![
            RouteInfo::new("logout", "/logout", "POST"),
            RouteInfo::new("users", "/users", "GET"),
            RouteInfo::new("users_create", "/users/create", "GET"),
            RouteInfo::new("users_store", "/users", "POST"),
            RouteInfo::new("users_edit", "/users/{id}/edit", "GET"),
            RouteInfo::new("users_update", "/users/{id}", "POST"),
            RouteInfo::new("users_destroy", "/users/{id}", "DELETE"),
            RouteInfo::new("users_restore", "/users/{id}/restore", "PUT"),
            RouteInfo::new("contacts", "/contacts", "GET"),
            RouteInfo::new("contacts_create", "/contacts/create", "GET"),
            RouteInfo::new("contacts_store", "/contacts", "POST"),
            RouteInfo::new("contacts_edit", "/contacts/{id}/edit", "GET"),
            RouteInfo::new("contacts_update", "/contacts/{id}", "PUT"),
            RouteInfo::new("contacts_destroy", "/contacts/{id}", "DELETE"),
            RouteInfo::new("contacts_restore", "/contacts/{id}/restore", "PUT"),
            RouteInfo::new("organizations", "/organizations", "GET"),
            RouteInfo::new("organizations_create", "/organizations/create", "GET"),
            RouteInfo::new("organizations_store", "/organizations", "POST"),
            RouteInfo::new("organizations_edit", "/organizations/{id}/edit", "GET"),
            RouteInfo::new("organizations_update", "/organizations/{id}", "PUT"),
            RouteInfo::new("organizations_destroy", "/organizations/{id}", "DELETE"),
            RouteInfo::new(
                "organizations_restore",
                "/organizations/{id}/restore",
                "PUT",
            ),
            RouteInfo::new("orders", "/orders", "GET"),
            RouteInfo::new("reports", "/reports", "GET"),
            RouteInfo::new("upload", "/upload", "GET"),
        ]
    }
}
