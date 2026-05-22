use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operations {
    pub index: bool,

    pub show: bool,

    pub create: bool,

    pub update: bool,

    pub delete: bool,
}

impl Operations {
    pub fn all() -> Self {
        Self {
            index: true,
            show: true,
            create: true,
            update: true,
            delete: true,
        }
    }

    pub fn read_only() -> Self {
        Self {
            index: true,
            show: true,
            create: false,
            update: false,
            delete: false,
        }
    }

    pub fn custom() -> OperationsBuilder {
        OperationsBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct OperationsBuilder {
    index: bool,
    show: bool,
    create: bool,
    update: bool,
    delete: bool,
}

impl OperationsBuilder {
    pub fn index(mut self) -> Self {
        self.index = true;
        self
    }

    pub fn show(mut self) -> Self {
        self.show = true;
        self
    }

    pub fn create(mut self) -> Self {
        self.create = true;
        self
    }

    pub fn update(mut self) -> Self {
        self.update = true;
        self
    }

    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }

    pub fn build(self) -> Operations {
        Operations {
            index: self.index,
            show: self.show,
            create: self.create,
            update: self.update,
            delete: self.delete,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntityConfig {
    pub resource_key: String,

    pub operations: Operations,

    pub relationships: Vec<RelationshipMeta>,

    pub read_only: bool,

    pub filterable_fields: Vec<String>,

    pub sortable_fields: Vec<String>,

    pub search_strategies: std::collections::HashMap<String, super::query::SearchStrategy>,
}

#[derive(Debug, Clone)]
pub struct RelationshipMeta {
    pub name: String,
    pub target_type: String,
    pub local_field: Option<String>,
    pub remote_field: String,
    pub cardinality: RelationshipCardinality,
    pub owning_side: bool,
    pub fetch_strategy: FetchStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipCardinality {
    ToOne,
    ToMany,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchStrategy {
    Lazy,
    Eager,
}

impl RelationshipMeta {
    pub fn belongs_to(
        name: impl Into<String>,
        target_type: impl Into<String>,
        local_field: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            target_type: target_type.into(),
            local_field: Some(local_field.into()),
            remote_field: "id".to_string(),
            cardinality: RelationshipCardinality::ToOne,
            owning_side: true,
            fetch_strategy: FetchStrategy::Lazy,
        }
    }

    pub fn has_one(
        name: impl Into<String>,
        target_type: impl Into<String>,
        remote_field: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            target_type: target_type.into(),
            local_field: None,
            remote_field: remote_field.into(),
            cardinality: RelationshipCardinality::ToOne,
            owning_side: false,
            fetch_strategy: FetchStrategy::Lazy,
        }
    }

    pub fn has_many(
        name: impl Into<String>,
        target_type: impl Into<String>,
        remote_field: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            target_type: target_type.into(),
            local_field: None,
            remote_field: remote_field.into(),
            cardinality: RelationshipCardinality::ToMany,
            owning_side: false,
            fetch_strategy: FetchStrategy::Lazy,
        }
    }

    pub fn eager(mut self) -> Self {
        self.fetch_strategy = FetchStrategy::Eager;
        self
    }

    pub fn query_field(&self) -> &str {
        self.local_field.as_deref().unwrap_or(&self.remote_field)
    }
}

impl EntityConfig {
    pub fn new(resource_key: impl Into<String>) -> Self {
        Self {
            resource_key: resource_key.into(),
            operations: Operations::all(),
            relationships: Vec::new(),
            read_only: false,
            filterable_fields: Vec::new(),
            sortable_fields: Vec::new(),
            search_strategies: std::collections::HashMap::new(),
        }
    }

    pub fn operations(mut self, ops: Operations) -> Self {
        self.operations = ops;
        self
    }

    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self.operations = Operations::read_only();
        self
    }

    pub fn filterable(mut self, fields: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.filterable_fields = fields.into_iter().map(Into::into).collect();
        self
    }

    pub fn sortable(mut self, fields: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.sortable_fields = fields.into_iter().map(Into::into).collect();
        self
    }

    pub fn searchable<I, S>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = (S, super::query::SearchStrategy)>,
        S: Into<String>,
    {
        for (field, strategy) in entries {
            self.search_strategies.insert(field.into(), strategy);
        }
        self
    }

    pub fn relationship(mut self, meta: RelationshipMeta) -> Self {
        self.relationships.push(meta);
        self
    }

    pub fn with_relationships(mut self, metas: Vec<RelationshipMeta>) -> Self {
        self.relationships.extend(metas);
        self
    }

    pub fn belongs_to(
        mut self,
        name: impl Into<String>,
        target_type: impl Into<String>,
        local_field: impl Into<String>,
    ) -> Self {
        self.relationships
            .push(RelationshipMeta::belongs_to(name, target_type, local_field));
        self
    }

    pub fn has_one(
        mut self,
        name: impl Into<String>,
        target_type: impl Into<String>,
        remote_field: impl Into<String>,
    ) -> Self {
        self.relationships
            .push(RelationshipMeta::has_one(name, target_type, remote_field));
        self
    }

    pub fn has_many(
        mut self,
        name: impl Into<String>,
        target_type: impl Into<String>,
        remote_field: impl Into<String>,
    ) -> Self {
        self.relationships
            .push(RelationshipMeta::has_many(name, target_type, remote_field));
        self
    }
}

#[derive(Clone)]
pub struct JsonApiConfigurator {
    prefix: String,
    entities: HashMap<String, EntityConfig>,
}

impl JsonApiConfigurator {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            entities: HashMap::new(),
        }
    }

    pub fn entity(&mut self, name: impl Into<String>, config: EntityConfig) -> &mut Self {
        self.entities.insert(name.into(), config);
        self
    }

    pub fn entities(&self) -> &HashMap<String, EntityConfig> {
        &self.entities
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn generate_route_list(&self) -> Vec<String> {
        let mut routes = Vec::new();

        for config in self.entities.values() {
            let resource_key = &config.resource_key;
            let ops = if config.read_only {
                &Operations::read_only()
            } else {
                &config.operations
            };

            if ops.index {
                routes.push(format!(
                    "GET    {}/{}              api.{}.index",
                    self.prefix, resource_key, resource_key
                ));
            }

            if ops.show {
                routes.push(format!(
                    "GET    {}/{}/:id          api.{}.show",
                    self.prefix, resource_key, resource_key
                ));
            }

            if ops.create {
                routes.push(format!(
                    "POST   {}/{}              api.{}.create",
                    self.prefix, resource_key, resource_key
                ));
            }

            if ops.update {
                routes.push(format!(
                    "PATCH  {}/{}/:id          api.{}.update",
                    self.prefix, resource_key, resource_key
                ));
            }

            if ops.delete {
                routes.push(format!(
                    "DELETE {}/{}/:id          api.{}.delete",
                    self.prefix, resource_key, resource_key
                ));
            }

            for relationship in &config.relationships {
                routes.push(format!(
                    "GET    {}/{}/:id/{}   api.{}.related.{}",
                    self.prefix, resource_key, relationship.name, resource_key, relationship.name
                ));
            }
        }

        routes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operations_all() {
        let ops = Operations::all();
        assert!(ops.index);
        assert!(ops.show);
        assert!(ops.create);
        assert!(ops.update);
        assert!(ops.delete);
    }

    #[test]
    fn test_operations_read_only() {
        let ops = Operations::read_only();
        assert!(ops.index);
        assert!(ops.show);
        assert!(!ops.create);
        assert!(!ops.update);
        assert!(!ops.delete);
    }

    #[test]
    fn test_operations_custom() {
        let ops = Operations::custom().index().show().create().build();
        assert!(ops.index);
        assert!(ops.show);
        assert!(ops.create);
        assert!(!ops.update);
        assert!(!ops.delete);
    }

    #[test]
    fn test_entity_config() {
        let config = EntityConfig::new("users")
            .belongs_to("organization", "organizations", "organization_id")
            .has_many("posts", "posts", "user_id");

        assert_eq!(config.resource_key, "users");
        assert_eq!(config.relationships.len(), 2);
        assert!(!config.read_only);
    }

    #[test]
    fn test_entity_config_read_only() {
        let config = EntityConfig::new("audit_logs").read_only();

        assert!(config.read_only);
        assert!(config.operations.index);
        assert!(config.operations.show);
        assert!(!config.operations.create);
    }
}
