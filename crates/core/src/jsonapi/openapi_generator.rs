use super::configurator::{EntityConfig, JsonApiConfigurator};
use schemars::schema_for;
use serde_json::{json, Value};

pub struct OpenApiGenerator {
    configurator: JsonApiConfigurator,
    title: String,
    version: String,
    description: Option<String>,
}

impl OpenApiGenerator {
    pub fn new(configurator: JsonApiConfigurator) -> Self {
        Self {
            configurator,
            title: "JSON:API".to_string(),
            version: "1.0.0".to_string(),
            description: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn generate(&self) -> Value {
        json!({
            "openapi": "3.1.0",
            "info": self.generate_info(),
            "servers": self.generate_servers(),
            "paths": self.generate_paths(),
            "components": {
                "schemas": self.generate_schemas(),
                "securitySchemes": self.generate_security_schemes()
            },
            "tags": self.generate_tags()
        })
    }

    fn generate_info(&self) -> Value {
        let info = json!({
            "title": self.title,
            "version": self.version,
            "description": self.description.as_ref().unwrap_or(&format!(
                "JSON:API v1.0 compliant REST API\n\nContent-Type: application/vnd.api+json"
            ))
        });

        info
    }

    fn generate_servers(&self) -> Vec<Value> {
        vec![json!({
            "url": self.configurator.prefix(),
            "description": "JSON:API v1 endpoints"
        })]
    }

    fn generate_paths(&self) -> Value {
        let mut paths = serde_json::Map::new();

        for (_, config) in self.configurator.entities() {
            let resource_key = &config.resource_key;

            if config.operations.index || config.operations.create {
                let collection_path = format!("/{}", resource_key);
                paths.insert(collection_path, self.generate_collection_operations(config));
            }

            if config.operations.show || config.operations.update || config.operations.delete {
                let item_path = format!("/{}/{{id}}", resource_key);
                paths.insert(item_path, self.generate_item_operations(config));
            }

            for rel in &config.relationships {
                let rel_path = format!("/{}/{{id}}/{}", resource_key, rel.name);
                paths.insert(
                    rel_path,
                    self.generate_relationship_operation(config, &rel.name),
                );
            }
        }

        Value::Object(paths)
    }

    fn generate_collection_operations(&self, config: &EntityConfig) -> Value {
        let mut ops = serde_json::Map::new();
        let resource_singular = self.to_singular(&config.resource_key);
        let resource_title = self.to_title_case(&resource_singular);

        if config.operations.index {
            ops.insert(
                "get".to_string(),
                json!({
                    "summary": format!("List {}", config.resource_key),
                    "description": format!("Retrieve a paginated list of {} resources", config.resource_key),
                    "operationId": format!("list_{}", config.resource_key),
                    "tags": [config.resource_key],
                    "parameters": self.generate_list_parameters(config),
                    "responses": {
                        "200": {
                            "description": "Successful response",
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {
                                        "$ref": format!("#/components/schemas/{}Collection", self.to_pascal_case(&resource_singular))
                                    }
                                }
                            }
                        },
                        "400": {
                            "description": "Bad request",
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {"$ref": "#/components/schemas/ErrorDocument"}
                                }
                            }
                        }
                    }
                }),
            );
        }

        if config.operations.create {
            ops.insert(
                "post".to_string(),
                json!({
                    "summary": format!("Create {}", resource_singular),
                    "description": format!("Create a new {} resource", resource_singular),
                    "operationId": format!("create_{}", resource_singular),
                    "tags": [config.resource_key],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/vnd.api+json": {
                                "schema": {
                                    "$ref": format!("#/components/schemas/Create{}Request", self.to_pascal_case(&resource_singular))
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": format!("{} created successfully", resource_title),
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {
                                        "$ref": format!("#/components/schemas/{}Response", self.to_pascal_case(&resource_singular))
                                    }
                                }
                            }
                        },
                        "400": {
                            "description": "Bad request - Invalid JSON:API format",
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {"$ref": "#/components/schemas/ErrorDocument"}
                                }
                            }
                        },
                        "422": {
                            "description": "Validation error",
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {"$ref": "#/components/schemas/ErrorDocument"}
                                }
                            }
                        }
                    }
                }),
            );
        }

        Value::Object(ops)
    }

    fn generate_item_operations(&self, config: &EntityConfig) -> Value {
        let mut ops = serde_json::Map::new();
        let resource_singular = self.to_singular(&config.resource_key);
        let resource_title = self.to_title_case(&resource_singular);

        let id_param = json!({
            "name": "id",
            "in": "path",
            "required": true,
            "description": format!("The {} ID", resource_singular),
            "schema": {
                "type": "string"
            }
        });

        if config.operations.show {
            ops.insert(
                "get".to_string(),
                json!({
                    "summary": format!("Get {}", resource_singular),
                    "description": format!("Retrieve a single {} resource by ID", resource_singular),
                    "operationId": format!("get_{}", resource_singular),
                    "tags": [config.resource_key],
                    "parameters": [
                        id_param.clone(),
                        {
                            "name": "include",
                            "in": "query",
                            "description": "Comma-separated list of related resources to include",
                            "schema": {"type": "string"},
                            "example": self.generate_include_example(config)
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Successful response",
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {
                                        "$ref": format!("#/components/schemas/{}Response", self.to_pascal_case(&resource_singular))
                                    }
                                }
                            }
                        },
                        "404": {
                            "description": format!("{} not found", resource_title),
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {"$ref": "#/components/schemas/ErrorDocument"}
                                }
                            }
                        }
                    }
                }),
            );
        }

        if config.operations.update {
            ops.insert(
                "patch".to_string(),
                json!({
                    "summary": format!("Update {}", resource_singular),
                    "description": format!("Update an existing {} resource", resource_singular),
                    "operationId": format!("update_{}", resource_singular),
                    "tags": [config.resource_key],
                    "parameters": [id_param.clone()],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/vnd.api+json": {
                                "schema": {
                                    "$ref": format!("#/components/schemas/Update{}Request", self.to_pascal_case(&resource_singular))
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": format!("{} updated successfully", resource_title),
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {
                                        "$ref": format!("#/components/schemas/{}Response", self.to_pascal_case(&resource_singular))
                                    }
                                }
                            }
                        },
                        "404": {
                            "description": format!("{} not found", resource_title),
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {"$ref": "#/components/schemas/ErrorDocument"}
                                }
                            }
                        },
                        "422": {
                            "description": "Validation error",
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {"$ref": "#/components/schemas/ErrorDocument"}
                                }
                            }
                        }
                    }
                }),
            );
        }

        if config.operations.delete {
            ops.insert(
                "delete".to_string(),
                json!({
                    "summary": format!("Delete {}", resource_singular),
                    "description": format!("Delete a {} resource", resource_singular),
                    "operationId": format!("delete_{}", resource_singular),
                    "tags": [config.resource_key],
                    "parameters": [id_param],
                    "responses": {
                        "204": {
                            "description": format!("{} deleted successfully", resource_title)
                        },
                        "404": {
                            "description": format!("{} not found", resource_title),
                            "content": {
                                "application/vnd.api+json": {
                                    "schema": {"$ref": "#/components/schemas/ErrorDocument"}
                                }
                            }
                        }
                    }
                }),
            );
        }

        Value::Object(ops)
    }

    fn generate_relationship_operation(&self, config: &EntityConfig, rel_name: &str) -> Value {
        let resource_singular = self.to_singular(&config.resource_key);

        json!({
            "get": {
                "summary": format!("Get {} {}", resource_singular, rel_name),
                "description": format!("Retrieve the {} relationship for a {}", rel_name, resource_singular),
                "operationId": format!("get_{}_{}", resource_singular, rel_name),
                "tags": [config.resource_key],
                "parameters": [
                    {
                        "name": "id",
                        "in": "path",
                        "required": true,
                        "description": format!("The {} ID", resource_singular),
                        "schema": {"type": "string"}
                    }
                ],
                "responses": {
                    "200": {
                        "description": "Successful response",
                        "content": {
                            "application/vnd.api+json": {
                                "schema": {"$ref": "#/components/schemas/JsonApiDocument"}
                            }
                        }
                    },
                    "404": {
                        "description": "Resource not found",
                        "content": {
                            "application/vnd.api+json": {
                                "schema": {"$ref": "#/components/schemas/ErrorDocument"}
                            }
                        }
                    }
                }
            }
        })
    }

    fn generate_list_parameters(&self, config: &EntityConfig) -> Vec<Value> {
        let mut params = vec![
            json!({
                "name": "page[number]",
                "in": "query",
                "description": "Page number to retrieve",
                "schema": {
                    "type": "integer",
                    "minimum": 1,
                    "default": 1
                }
            }),
            json!({
                "name": "page[size]",
                "in": "query",
                "description": "Number of items per page",
                "schema": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 100,
                    "default": 20
                }
            }),
            json!({
                "name": "filter",
                "in": "query",
                "description": "Filter results (implementation-specific)",
                "schema": {"type": "string"},
                "example": "name:John"
            }),
            json!({
                "name": "sort",
                "in": "query",
                "description": "Sort results by field(s). Prefix with '-' for descending",
                "schema": {"type": "string"},
                "example": "-created_at,name"
            }),
        ];

        if !config.relationships.is_empty() {
            params.push(json!({
                "name": "include",
                "in": "query",
                "description": "Comma-separated list of related resources to include",
                "schema": {"type": "string"},
                "example": self.generate_include_example(config)
            }));
        }

        params
    }

    fn generate_include_example(&self, config: &EntityConfig) -> String {
        if config.relationships.is_empty() {
            "".to_string()
        } else {
            config
                .relationships
                .iter()
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>()
                .join(",")
        }
    }

    fn generate_schemas(&self) -> Value {
        let mut schemas = serde_json::Map::new();

        self.add_base_schemas(&mut schemas);

        for (_, config) in self.configurator.entities() {
            self.add_resource_schemas(&mut schemas, config);
        }

        Value::Object(schemas)
    }

    fn add_base_schemas(&self, schemas: &mut serde_json::Map<String, Value>) {
        use super::types::*;

        let resource_schema = schema_for!(JsonApiResource);
        let links_schema = schema_for!(JsonApiLinks);
        let error_schema = schema_for!(JsonApiError);
        let relationship_schema = schema_for!(JsonApiRelationship);

        schemas.insert(
            "JsonApiResource".to_string(),
            serde_json::to_value(&resource_schema).unwrap(),
        );
        schemas.insert(
            "JsonApiLinks".to_string(),
            serde_json::to_value(&links_schema).unwrap(),
        );
        schemas.insert(
            "JsonApiError".to_string(),
            serde_json::to_value(&error_schema).unwrap(),
        );
        schemas.insert(
            "JsonApiRelationship".to_string(),
            serde_json::to_value(&relationship_schema).unwrap(),
        );

        schemas.insert(
            "JsonApiDocument".to_string(),
            json!({
                "type": "object",
                "required": ["data"],
                "properties": {
                    "data": {
                        "oneOf": [
                            {"$ref": "#/components/schemas/JsonApiResource"},
                            {
                                "type": "array",
                                "items": {"$ref": "#/components/schemas/JsonApiResource"}
                            }
                        ]
                    },
                    "included": {
                        "type": "array",
                        "items": {"$ref": "#/components/schemas/JsonApiResource"}
                    },
                    "meta": {"type": "object"},
                    "links": {"$ref": "#/components/schemas/JsonApiLinks"},
                    "jsonapi": {
                        "type": "object",
                        "properties": {
                            "version": {"type": "string", "example": "1.0"}
                        }
                    }
                }
            }),
        );

        schemas.insert(
            "ErrorDocument".to_string(),
            json!({
                "type": "object",
                "required": ["errors"],
                "properties": {
                    "errors": {
                        "type": "array",
                        "items": {"$ref": "#/components/schemas/JsonApiError"}
                    },
                    "jsonapi": {
                        "type": "object",
                        "properties": {
                            "version": {"type": "string", "example": "1.0"}
                        }
                    }
                }
            }),
        );
    }

    fn add_resource_schemas(
        &self,
        schemas: &mut serde_json::Map<String, Value>,
        config: &EntityConfig,
    ) {
        let resource_singular = self.to_singular(&config.resource_key);
        let pascal_name = self.to_pascal_case(&resource_singular);

        schemas.insert(
            format!("{}Response", pascal_name),
            json!({
                "allOf": [
                    {"$ref": "#/components/schemas/JsonApiDocument"},
                    {
                        "type": "object",
                        "properties": {
                            "data": {
                                "allOf": [
                                    {"$ref": "#/components/schemas/JsonApiResource"},
                                    {
                                        "type": "object",
                                        "properties": {
                                            "type": {"type": "string", "enum": [&config.resource_key]}
                                        }
                                    }
                                ]
                            }
                        }
                    }
                ]
            }),
        );

        schemas.insert(
            format!("{}Collection", pascal_name),
            json!({
                "allOf": [
                    {"$ref": "#/components/schemas/JsonApiDocument"},
                    {
                        "type": "object",
                        "properties": {
                            "data": {
                                "type": "array",
                                "items": {
                                    "allOf": [
                                        {"$ref": "#/components/schemas/JsonApiResource"},
                                        {
                                            "type": "object",
                                            "properties": {
                                                "type": {"type": "string", "enum": [&config.resource_key]}
                                            }
                                        }
                                    ]
                                }
                            }
                        }
                    }
                ]
            }),
        );

        if config.operations.create {
            schemas.insert(
                format!("Create{}Request", pascal_name),
                json!({
                    "type": "object",
                    "required": ["data"],
                    "properties": {
                        "data": {
                            "type": "object",
                            "required": ["type", "attributes"],
                            "properties": {
                                "type": {"type": "string", "enum": [&config.resource_key]},
                                "attributes": {
                                    "type": "object",
                                    "description": format!("Attributes for creating a {}", resource_singular)
                                },
                                "relationships": {
                                    "type": "object",
                                    "description": "Optional relationships"
                                }
                            }
                        }
                    }
                }),
            );
        }

        if config.operations.update {
            schemas.insert(
                format!("Update{}Request", pascal_name),
                json!({
                    "type": "object",
                    "required": ["data"],
                    "properties": {
                        "data": {
                            "type": "object",
                            "required": ["type", "id"],
                            "properties": {
                                "type": {"type": "string", "enum": [&config.resource_key]},
                                "id": {"type": "string"},
                                "attributes": {
                                    "type": "object",
                                    "description": format!("Attributes to update on {}", resource_singular)
                                },
                                "relationships": {
                                    "type": "object",
                                    "description": "Optional relationships to update"
                                }
                            }
                        }
                    }
                }),
            );
        }
    }

    fn generate_security_schemes(&self) -> Value {
        json!({
            "cookieAuth": {
                "type": "apiKey",
                "in": "cookie",
                "name": "session",
                "description": "Session cookie authentication"
            }
        })
    }

    fn generate_tags(&self) -> Vec<Value> {
        self.configurator
            .entities()
            .iter()
            .map(|(_, config)| {
                let description =
                    format!("Operations for managing {} resources", config.resource_key);

                json!({
                    "name": config.resource_key,
                    "description": description
                })
            })
            .collect()
    }

    fn to_pascal_case(&self, s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }

    fn to_title_case(&self, s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        let mut result: String = first.to_uppercase().collect();
                        result.push_str(&chars.collect::<String>());
                        result
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn to_singular(&self, s: &str) -> String {
        s.strip_suffix("s").unwrap_or(s).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonapi::Operations;

    #[test]
    fn test_basic_generation() {
        let mut config = JsonApiConfigurator::new("/api/v1");
        config.entity("contacts", EntityConfig::new("contacts"));

        let generator = OpenApiGenerator::new(config)
            .title("Test API")
            .version("1.0.0");

        let spec = generator.generate();

        assert_eq!(spec["openapi"], "3.1.0");
        assert_eq!(spec["info"]["title"], "Test API");
        assert_eq!(spec["info"]["version"], "1.0.0");
    }

    #[test]
    fn test_paths_generation() {
        let mut config = JsonApiConfigurator::new("/api/v1");
        config.entity("contacts", EntityConfig::new("contacts"));

        let generator = OpenApiGenerator::new(config);
        let spec = generator.generate();

        let paths = spec["paths"].as_object().unwrap();
        assert!(paths.contains_key("/contacts"));
        assert!(paths.contains_key("/contacts/{id}"));
    }

    #[test]
    fn test_operations_filtering() {
        let mut config = JsonApiConfigurator::new("/api/v1");
        config.entity(
            "audit_logs",
            EntityConfig::new("audit_logs").operations(Operations::read_only()),
        );

        let generator = OpenApiGenerator::new(config);
        let spec = generator.generate();

        let paths = spec["paths"].as_object().unwrap();
        let collection = paths["/audit_logs"].as_object().unwrap();

        assert!(collection.contains_key("get"));
        assert!(!collection.contains_key("post"));
    }

    #[test]
    fn test_relationships_in_paths() {
        let mut config = JsonApiConfigurator::new("/api/v1");
        config.entity(
            "contacts",
            EntityConfig::new("contacts").belongs_to(
                "organization",
                "organizations",
                "organization_id",
            ),
        );

        let generator = OpenApiGenerator::new(config);
        let spec = generator.generate();

        let paths = spec["paths"].as_object().unwrap();
        assert!(paths.contains_key("/contacts/{id}/organization"));
    }

    #[test]
    fn test_schemas_generation() {
        let mut config = JsonApiConfigurator::new("/api/v1");
        config.entity("contacts", EntityConfig::new("contacts"));

        let generator = OpenApiGenerator::new(config);
        let spec = generator.generate();

        let schemas = spec["components"]["schemas"].as_object().unwrap();

        assert!(schemas.contains_key("JsonApiResource"));
        assert!(schemas.contains_key("JsonApiError"));
        assert!(schemas.contains_key("ErrorDocument"));

        assert!(schemas.contains_key("ContactResponse"));
        assert!(schemas.contains_key("ContactCollection"));
        assert!(schemas.contains_key("CreateContactRequest"));
        assert!(schemas.contains_key("UpdateContactRequest"));
    }

    #[test]
    fn test_helper_methods() {
        let config = JsonApiConfigurator::new("/api");
        let generator = OpenApiGenerator::new(config);

        assert_eq!(generator.to_pascal_case("snake_case"), "SnakeCase");
        assert_eq!(generator.to_pascal_case("single"), "Single");
        assert_eq!(generator.to_singular("contacts"), "contact");
        assert_eq!(generator.to_singular("data"), "data");
        assert_eq!(generator.to_title_case("first_name"), "First Name");
    }
}
