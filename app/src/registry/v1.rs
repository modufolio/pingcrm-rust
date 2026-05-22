use crate::config::configure_v1_resources;
use crate::database::pool::DbPool;
use appkit_core::error::AppError;
use appkit_core::jsonapi::{EntityConfig, JsonApiConfigurator, RelationshipMeta, ResourceHandler};
use std::collections::HashMap;
use std::sync::Arc;

pub struct V1Registry {
    handlers: HashMap<String, Arc<dyn ResourceHandler + Send + Sync>>,
    configurator: JsonApiConfigurator,
}

impl V1Registry {
    pub fn new(db_pool: DbPool) -> Self {
        let config = configure_v1_resources(db_pool);

        Self {
            handlers: config.handlers,
            configurator: config.configurator,
        }
    }

    pub(crate) fn from_config(
        handlers: HashMap<String, Arc<dyn ResourceHandler + Send + Sync>>,
        configurator: JsonApiConfigurator,
    ) -> Self {
        Self {
            handlers,
            configurator,
        }
    }

    pub fn get(&self, resource: &str) -> Result<&Arc<dyn ResourceHandler + Send + Sync>, AppError> {
        self.handlers
            .get(resource)
            .ok_or_else(|| AppError::NotFound(format!("Resource '{}' not found", resource)))
    }

    pub fn has(&self, resource: &str) -> bool {
        self.handlers.contains_key(resource)
    }

    pub fn resources(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }

    /// Fetch the registered `EntityConfig` for a resource. Used by the
    /// JSON:API controller to validate `?filter[…]` and `?sort=…` against
    /// the per-entity allowlists.
    pub fn entity_config(&self, resource: &str) -> Option<&EntityConfig> {
        self.configurator.entities().get(resource)
    }

    pub fn get_relationship(
        &self,
        resource: &str,
        relationship_name: &str,
    ) -> Result<&RelationshipMeta, AppError> {
        let entity_config = self
            .configurator
            .entities()
            .get(resource)
            .ok_or_else(|| AppError::NotFound(format!("Resource '{}' not found", resource)))?;

        entity_config
            .relationships
            .iter()
            .find(|r| r.name == relationship_name)
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Relationship '{}' does not exist on resource '{}'",
                    relationship_name, resource
                ))
            })
    }
}
