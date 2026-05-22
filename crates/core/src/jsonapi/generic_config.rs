use std::collections::HashMap;

#[derive(Clone)]
pub struct ResourceConfig {
    pub resource_type: &'static str,

    pub allowed_fields: &'static [&'static str],

    pub allowed_includes: &'static [&'static str],

    pub relationships: HashMap<String, RelationshipConfig>,
}

#[derive(Clone)]
pub struct RelationshipConfig {
    pub target_type: &'static str,

    pub foreign_key: &'static str,

    pub cardinality: Cardinality,

    pub join_table: Option<&'static str>,

    pub inverse_foreign_key: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cardinality {
    BelongsTo,

    HasMany,

    ManyToMany,
}

impl ResourceConfig {
    pub fn new(
        resource_type: &'static str,
        allowed_fields: &'static [&'static str],
        allowed_includes: &'static [&'static str],
    ) -> Self {
        Self {
            resource_type,
            allowed_fields,
            allowed_includes,
            relationships: HashMap::new(),
        }
    }

    pub fn with_relationship(
        mut self,
        name: impl Into<String>,
        config: RelationshipConfig,
    ) -> Self {
        self.relationships.insert(name.into(), config);
        self
    }
}

impl RelationshipConfig {
    pub fn belongs_to(target_type: &'static str, foreign_key: &'static str) -> Self {
        Self {
            target_type,
            foreign_key,
            cardinality: Cardinality::BelongsTo,
            join_table: None,
            inverse_foreign_key: None,
        }
    }

    pub fn has_many(target_type: &'static str, foreign_key: &'static str) -> Self {
        Self {
            target_type,
            foreign_key,
            cardinality: Cardinality::HasMany,
            join_table: None,
            inverse_foreign_key: None,
        }
    }

    pub fn many_to_many(
        target_type: &'static str,
        join_table: &'static str,
        source_foreign_key: &'static str,
        target_foreign_key: &'static str,
    ) -> Self {
        Self {
            target_type,
            foreign_key: source_foreign_key,
            cardinality: Cardinality::ManyToMany,
            join_table: Some(join_table),
            inverse_foreign_key: Some(target_foreign_key),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DynamicResourceConfig {
    pub resource_type: String,

    pub allowed_fields: Vec<String>,

    pub allowed_includes: Vec<String>,

    pub relationships: HashMap<String, DynamicRelationshipConfig>,

    pub field_mappings: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct DynamicRelationshipConfig {
    pub target_type: String,

    pub foreign_key: String,

    pub cardinality: Cardinality,

    pub join_table: Option<String>,

    pub inverse_foreign_key: Option<String>,
}

impl DynamicResourceConfig {
    pub fn builder(resource_type: impl Into<String>) -> DynamicResourceConfigBuilder {
        DynamicResourceConfigBuilder::new(resource_type)
    }

    pub fn from_static(config: &ResourceConfig) -> Self {
        Self {
            resource_type: config.resource_type.to_string(),
            allowed_fields: config
                .allowed_fields
                .iter()
                .map(|s| s.to_string())
                .collect(),
            allowed_includes: config
                .allowed_includes
                .iter()
                .map(|s| s.to_string())
                .collect(),
            relationships: config
                .relationships
                .iter()
                .map(|(k, v)| (k.clone(), DynamicRelationshipConfig::from_static(v)))
                .collect(),
            field_mappings: HashMap::new(),
        }
    }

    pub fn allow_field(&mut self, field: impl Into<String>) -> &mut Self {
        let field = field.into();
        if !self.allowed_fields.contains(&field) {
            self.allowed_fields.push(field);
        }
        self
    }

    pub fn deny_field(&mut self, field: &str) -> &mut Self {
        self.allowed_fields.retain(|f| f != field);
        self
    }

    pub fn allow_include(&mut self, include: impl Into<String>) -> &mut Self {
        let include = include.into();
        if !self.allowed_includes.contains(&include) {
            self.allowed_includes.push(include);
        }
        self
    }

    pub fn deny_include(&mut self, include: &str) -> &mut Self {
        self.allowed_includes.retain(|i| i != include);
        self
    }

    pub fn map_field(
        &mut self,
        api_name: impl Into<String>,
        db_name: impl Into<String>,
    ) -> &mut Self {
        self.field_mappings.insert(api_name.into(), db_name.into());
        self
    }

    pub fn get_column_name<'a>(&'a self, api_field: &'a str) -> &'a str {
        self.field_mappings
            .get(api_field)
            .map(|s| s.as_str())
            .unwrap_or(api_field)
    }

    pub fn with_relationship(
        mut self,
        name: impl Into<String>,
        config: DynamicRelationshipConfig,
    ) -> Self {
        self.relationships.insert(name.into(), config);
        self
    }
}

impl DynamicRelationshipConfig {
    pub fn from_static(config: &RelationshipConfig) -> Self {
        Self {
            target_type: config.target_type.to_string(),
            foreign_key: config.foreign_key.to_string(),
            cardinality: config.cardinality,
            join_table: config.join_table.map(|s| s.to_string()),
            inverse_foreign_key: config.inverse_foreign_key.map(|s| s.to_string()),
        }
    }

    pub fn to_static(&self) -> RelationshipConfig {
        RelationshipConfig {
            target_type: Box::leak(self.target_type.clone().into_boxed_str()) as &'static str,
            foreign_key: Box::leak(self.foreign_key.clone().into_boxed_str()) as &'static str,
            cardinality: self.cardinality,
            join_table: self
                .join_table
                .as_ref()
                .map(|s| Box::leak(s.clone().into_boxed_str()) as &'static str),
            inverse_foreign_key: self
                .inverse_foreign_key
                .as_ref()
                .map(|s| Box::leak(s.clone().into_boxed_str()) as &'static str),
        }
    }
}

pub struct DynamicResourceConfigBuilder {
    resource_type: String,
    allowed_fields: Vec<String>,
    allowed_includes: Vec<String>,
    relationships: HashMap<String, DynamicRelationshipConfig>,
    field_mappings: HashMap<String, String>,
}

impl DynamicResourceConfigBuilder {
    pub fn new(resource_type: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            allowed_fields: Vec::new(),
            allowed_includes: Vec::new(),
            relationships: HashMap::new(),
            field_mappings: HashMap::new(),
        }
    }

    pub fn allow_fields(mut self, fields: Vec<String>) -> Self {
        self.allowed_fields = fields;
        self
    }

    pub fn allow_field(mut self, field: impl Into<String>) -> Self {
        self.allowed_fields.push(field.into());
        self
    }

    pub fn allow_includes(mut self, includes: Vec<String>) -> Self {
        self.allowed_includes = includes;
        self
    }

    pub fn allow_include(mut self, include: impl Into<String>) -> Self {
        self.allowed_includes.push(include.into());
        self
    }

    pub fn map_field(mut self, api_name: impl Into<String>, db_name: impl Into<String>) -> Self {
        self.field_mappings.insert(api_name.into(), db_name.into());
        self
    }

    pub fn with_relationship(
        mut self,
        name: impl Into<String>,
        config: DynamicRelationshipConfig,
    ) -> Self {
        self.relationships.insert(name.into(), config);
        self
    }

    pub fn build(self) -> DynamicResourceConfig {
        DynamicResourceConfig {
            resource_type: self.resource_type,
            allowed_fields: self.allowed_fields,
            allowed_includes: self.allowed_includes,
            relationships: self.relationships,
            field_mappings: self.field_mappings,
        }
    }
}

pub struct RelationshipBuilder {
    target_type: String,
    foreign_key: Option<String>,
    cardinality: Cardinality,
    join_table: Option<String>,
    inverse_foreign_key: Option<String>,
}

impl RelationshipBuilder {
    pub fn to_one(name: impl Into<String>, target_type: impl Into<String>) -> Self {
        let name = name.into();
        let foreign_key = format!("{}_id", name);

        Self {
            target_type: target_type.into(),
            foreign_key: Some(foreign_key),
            cardinality: Cardinality::BelongsTo,
            join_table: None,
            inverse_foreign_key: None,
        }
    }

    pub fn to_many(name: impl Into<String>, target_type: impl Into<String>) -> Self {
        let _ = name.into();
        Self {
            target_type: target_type.into(),
            foreign_key: None,
            cardinality: Cardinality::HasMany,
            join_table: None,
            inverse_foreign_key: None,
        }
    }

    pub fn many_to_many(name: impl Into<String>, target_type: impl Into<String>) -> Self {
        let _ = name.into();
        Self {
            target_type: target_type.into(),
            foreign_key: None,
            cardinality: Cardinality::ManyToMany,
            join_table: None,
            inverse_foreign_key: None,
        }
    }

    pub fn foreign_key(mut self, key: impl Into<String>) -> Self {
        self.foreign_key = Some(key.into());
        self
    }

    pub fn join_table(mut self, table: impl Into<String>) -> Self {
        self.join_table = Some(table.into());
        self
    }

    pub fn source_key(mut self, key: impl Into<String>) -> Self {
        self.foreign_key = Some(key.into());
        self
    }

    pub fn target_key(mut self, key: impl Into<String>) -> Self {
        self.inverse_foreign_key = Some(key.into());
        self
    }

    pub fn build(self) -> DynamicRelationshipConfig {
        match self.cardinality {
            Cardinality::BelongsTo => DynamicRelationshipConfig {
                target_type: self.target_type,
                foreign_key: self
                    .foreign_key
                    .expect("BelongsTo relationship requires foreign_key"),
                cardinality: Cardinality::BelongsTo,
                join_table: None,
                inverse_foreign_key: None,
            },
            Cardinality::HasMany => DynamicRelationshipConfig {
                target_type: self.target_type,
                foreign_key: self
                    .foreign_key
                    .expect("HasMany relationship requires foreign_key"),
                cardinality: Cardinality::HasMany,
                join_table: None,
                inverse_foreign_key: None,
            },
            Cardinality::ManyToMany => DynamicRelationshipConfig {
                target_type: self.target_type,
                foreign_key: self
                    .foreign_key
                    .expect("ManyToMany relationship requires source_key"),
                cardinality: Cardinality::ManyToMany,
                join_table: Some(
                    self.join_table
                        .expect("ManyToMany relationship requires join_table"),
                ),
                inverse_foreign_key: Some(
                    self.inverse_foreign_key
                        .expect("ManyToMany relationship requires target_key"),
                ),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_config_builder() {
        let config = DynamicResourceConfig::builder("products")
            .allow_fields(vec![
                "id".to_string(),
                "name".to_string(),
                "price".to_string(),
            ])
            .allow_includes(vec!["brand".to_string()])
            .map_field("createdAt", "created_at")
            .build();

        assert_eq!(config.resource_type, "products");
        assert_eq!(config.allowed_fields.len(), 3);
        assert_eq!(config.get_column_name("createdAt"), "created_at");
        assert_eq!(config.get_column_name("id"), "id");
    }

    #[test]
    fn test_relationship_builder_to_one() {
        let rel = RelationshipBuilder::to_one("account", "accounts").build();

        assert_eq!(rel.target_type, "accounts");
        assert_eq!(rel.foreign_key, "account_id");
        assert_eq!(rel.cardinality, Cardinality::BelongsTo);
    }

    #[test]
    fn test_relationship_builder_to_one_custom_fk() {
        let rel = RelationshipBuilder::to_one("organization", "organizations")
            .foreign_key("org_id")
            .build();

        assert_eq!(rel.foreign_key, "org_id");
    }

    #[test]
    fn test_relationship_builder_to_many() {
        let rel = RelationshipBuilder::to_many("contacts", "contacts")
            .foreign_key("user_id")
            .build();

        assert_eq!(rel.target_type, "contacts");
        assert_eq!(rel.foreign_key, "user_id");
        assert_eq!(rel.cardinality, Cardinality::HasMany);
    }

    #[test]
    fn test_relationship_builder_many_to_many() {
        let rel = RelationshipBuilder::many_to_many("roles", "roles")
            .join_table("user_roles")
            .source_key("user_id")
            .target_key("role_id")
            .build();

        assert_eq!(rel.target_type, "roles");
        assert_eq!(rel.foreign_key, "user_id");
        assert_eq!(rel.join_table, Some("user_roles".to_string()));
        assert_eq!(rel.inverse_foreign_key, Some("role_id".to_string()));
        assert_eq!(rel.cardinality, Cardinality::ManyToMany);
    }

    #[test]
    fn test_dynamic_config_field_mutations() {
        let mut config = DynamicResourceConfig::builder("users")
            .allow_fields(vec!["id".to_string(), "email".to_string()])
            .build();

        config.allow_field("name");
        assert_eq!(config.allowed_fields.len(), 3);
        assert!(config.allowed_fields.contains(&"name".to_string()));

        config.deny_field("email");
        assert_eq!(config.allowed_fields.len(), 2);
        assert!(!config.allowed_fields.contains(&"email".to_string()));
    }

    #[test]
    fn test_from_static_conversion() {
        let static_config = ResourceConfig::new("users", &["id", "email"], &["account"]);
        let dynamic_config = DynamicResourceConfig::from_static(&static_config);

        assert_eq!(dynamic_config.resource_type, "users");
        assert_eq!(dynamic_config.allowed_fields, vec!["id", "email"]);
        assert_eq!(dynamic_config.allowed_includes, vec!["account"]);
    }
}
