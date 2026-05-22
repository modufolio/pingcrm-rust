use crate::app::App;

use crate::handlers::V1Controller;
use crate::router::loader::{RouteInfo, RouteLoader};
use appkit_core::jsonapi::OpenApiGenerator;
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use std::sync::Arc;

pub struct JsonApiRouteLoader {
    configurator: appkit_core::jsonapi::JsonApiConfigurator,
}

impl JsonApiRouteLoader {
    pub fn new(configurator: appkit_core::jsonapi::JsonApiConfigurator) -> Self {
        Self { configurator }
    }
}

impl RouteLoader<App> for JsonApiRouteLoader {
    fn load(&self) -> Router<App> {
        let mut router = Router::new();

        for (_, entity_config) in self.configurator.entities() {
            let resource_key: Arc<str> = Arc::from(entity_config.resource_key.as_str());
            let collection_path = format!("/{}", resource_key);
            let item_path = format!("/{}/{{id}}", resource_key);

            let ops = &entity_config.operations;

            let mut collection_router = None;

            if ops.index && ops.create {
                let res_index = Arc::clone(&resource_key);
                let res_create = Arc::clone(&resource_key);
                collection_router = Some(
                    get(move |state, query| {
                        V1Controller::index_for_resource(state, res_index.to_string(), query)
                    })
                    .post(move |state, payload| {
                        V1Controller::create_for_resource(state, res_create.to_string(), payload)
                    }),
                );
            } else if ops.index {
                let res = Arc::clone(&resource_key);
                collection_router = Some(get(move |state, query| {
                    V1Controller::index_for_resource(state, res.to_string(), query)
                }));
            } else if ops.create {
                let res = Arc::clone(&resource_key);
                collection_router = Some(post(move |state, payload| {
                    V1Controller::create_for_resource(state, res.to_string(), payload)
                }));
            }

            if let Some(method_router) = collection_router {
                router = router.route(&collection_path, method_router);
            }

            let mut item_router = None;

            if ops.show {
                let res = Arc::clone(&resource_key);
                item_router = Some(get(move |state, id, query| {
                    V1Controller::show_for_resource(state, res.to_string(), id, query)
                }));
            }

            if ops.update {
                let res = Arc::clone(&resource_key);
                item_router = Some(match item_router {
                    Some(r) => r.patch(move |state, id, query, payload| {
                        V1Controller::update_for_resource(
                            state,
                            res.to_string(),
                            id,
                            query,
                            payload,
                        )
                    }),
                    None => patch(move |state, id, query, payload| {
                        V1Controller::update_for_resource(
                            state,
                            res.to_string(),
                            id,
                            query,
                            payload,
                        )
                    }),
                });
            }

            if ops.delete {
                let res = Arc::clone(&resource_key);
                item_router = Some(match item_router {
                    Some(r) => r.delete(move |state, id| {
                        V1Controller::delete_for_resource(state, res.to_string(), id)
                    }),
                    None => delete(move |state, id| {
                        V1Controller::delete_for_resource(state, res.to_string(), id)
                    }),
                });
            }

            if let Some(method_router) = item_router {
                router = router.route(&item_path, method_router);
            }

            for relationship in &entity_config.relationships {
                let relationship_path = format!("/{}/{{id}}/{}", resource_key, relationship.name);
                let res = Arc::clone(&resource_key);
                let rel_name: Arc<str> = Arc::from(relationship.name.as_str());

                router = router.route(
                    &relationship_path,
                    get(move |state, id, query| {
                        V1Controller::related_for_resource(
                            state,
                            res.to_string(),
                            id,
                            rel_name.to_string(),
                            query,
                        )
                    }),
                );
            }
        }

        let configurator_clone = self.configurator.clone();
        router = router.route(
            "/openapi.json",
            get(move || async move {
                let generator = OpenApiGenerator::new(configurator_clone.clone())
                    .title("PingCRM Rust API")
                    .version("1.0.0")
                    .description("JSON:API v1.0 compliant REST API for PingCRM");
                Json(generator.generate())
            }),
        );

        router
    }

    fn get_routes(&self) -> Vec<RouteInfo> {
        let mut routes = Vec::new();

        for (_, entity_config) in self.configurator.entities() {
            let resource_key = &entity_config.resource_key;

            let base_path = format!("/{}", resource_key);
            let item_path = format!("/{}/:id", resource_key);

            let ops = &entity_config.operations;

            if ops.index {
                routes.push(RouteInfo::new(
                    format!("{}.index", resource_key),
                    &base_path,
                    "GET",
                ));
            }

            if ops.create {
                routes.push(RouteInfo::new(
                    format!("{}.create", resource_key),
                    &base_path,
                    "POST",
                ));
            }

            if ops.show {
                routes.push(RouteInfo::new(
                    format!("{}.show", resource_key),
                    &item_path,
                    "GET",
                ));
            }

            if ops.update {
                routes.push(RouteInfo::new(
                    format!("{}.update", resource_key),
                    &item_path,
                    "PATCH",
                ));
            }

            if ops.delete {
                routes.push(RouteInfo::new(
                    format!("{}.delete", resource_key),
                    &item_path,
                    "DELETE",
                ));
            }

            for relationship in &entity_config.relationships {
                let relationship_path = format!("/{}/:id/{}", resource_key, relationship.name);
                routes.push(RouteInfo::new(
                    format!("{}.related.{}", resource_key, relationship.name),
                    &relationship_path,
                    "GET",
                ));
            }
        }

        routes.push(RouteInfo::new(
            "openapi".to_string(),
            "/openapi.json",
            "GET",
        ));

        routes
    }
}
