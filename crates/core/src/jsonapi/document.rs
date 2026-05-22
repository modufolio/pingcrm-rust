use super::error::ErrorObject;
use super::resource::ResourceObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub type Links = HashMap<String, String>;

pub type Meta = HashMap<String, Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonApiDocument {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonapi: Option<JsonApiVersion>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonApiData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ErrorObject>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub included: Option<Vec<ResourceObject>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonApiVersion {
    pub version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}

impl Default for JsonApiVersion {
    fn default() -> Self {
        Self {
            version: "1.1".to_string(),
            meta: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonApiData {
    Null,

    One(Box<ResourceObject>),

    Many(Vec<ResourceObject>),
}

impl JsonApiDocument {
    pub fn new() -> Self {
        Self {
            jsonapi: Some(JsonApiVersion::default()),
            data: None,
            errors: None,
            included: None,
            meta: None,
            links: None,
        }
    }

    pub fn with_resource(mut self, resource: ResourceObject) -> Self {
        self.data = Some(JsonApiData::One(Box::new(resource)));
        self.errors = None;
        self
    }

    pub fn with_data(mut self, resources: Vec<ResourceObject>) -> Self {
        self.data = Some(JsonApiData::Many(resources));
        self.errors = None;
        self
    }

    pub fn with_null_data(mut self) -> Self {
        self.data = Some(JsonApiData::Null);
        self.errors = None;
        self
    }

    pub fn with_errors(mut self, errors: Vec<ErrorObject>) -> Self {
        self.errors = Some(errors);
        self.data = None;
        self.included = None;
        self
    }

    pub fn with_error(mut self, error: ErrorObject) -> Self {
        self.errors = Some(vec![error]);
        self.data = None;
        self.included = None;
        self
    }

    pub fn with_included(mut self, included: Vec<ResourceObject>) -> Self {
        self.included = Some(included);
        self
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: Value) -> Self {
        self.meta
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value);
        self
    }

    pub fn with_meta_map(mut self, meta: Meta) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn with_link(mut self, key: impl Into<String>, url: impl Into<String>) -> Self {
        self.links
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), url.into());
        self
    }

    pub fn with_links(mut self, links: Links) -> Self {
        self.links = Some(links);
        self
    }

    pub fn with_jsonapi(mut self, version: JsonApiVersion) -> Self {
        self.jsonapi = Some(version);
        self
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn has_errors(&self) -> bool {
        self.errors.is_some()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.data.is_some() && self.errors.is_some() {
            return Err("Document cannot have both 'data' and 'errors'".to_string());
        }

        if self.included.is_some() && self.data.is_none() {
            return Err("Document cannot have 'included' without 'data'".to_string());
        }

        if self.data.is_none() && self.errors.is_none() && self.meta.is_none() {
            return Err(
                "Document must have at least one of: 'data', 'errors', or 'meta'".to_string(),
            );
        }

        Ok(())
    }
}

impl Default for JsonApiDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_document_with_single_resource() {
        let resource = ResourceObject::new("users", "1").set_attribute("name", json!("John"));

        let doc = JsonApiDocument::new().with_resource(resource);

        assert!(doc.has_data());
        assert!(!doc.has_errors());
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn test_document_with_collection() {
        let resources = vec![
            ResourceObject::new("users", "1"),
            ResourceObject::new("users", "2"),
        ];

        let doc = JsonApiDocument::new().with_data(resources);

        assert!(doc.has_data());
        assert!(!doc.has_errors());
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn test_document_with_errors() {
        let error = ErrorObject::new()
            .with_status(404)
            .with_title("Not Found")
            .with_detail("Resource not found");

        let doc = JsonApiDocument::new().with_error(error);

        assert!(!doc.has_data());
        assert!(doc.has_errors());
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn test_document_cannot_have_both_data_and_errors() {
        let mut doc = JsonApiDocument::new();
        doc.data = Some(JsonApiData::Null);
        doc.errors = Some(vec![]);

        assert!(doc.validate().is_err());
    }

    #[test]
    fn test_document_cannot_have_included_without_data() {
        let mut doc = JsonApiDocument::new();
        doc.included = Some(vec![]);

        assert!(doc.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let resource = ResourceObject::new("users", "1").set_attribute("name", json!("John"));

        let doc = JsonApiDocument::new()
            .with_resource(resource)
            .with_meta("total", json!(1))
            .with_link("self", "/api/users/1");

        let json_str = serde_json::to_string(&doc).unwrap();
        assert!(json_str.contains("\"jsonapi\""));
        assert!(json_str.contains("\"data\""));
        assert!(json_str.contains("\"meta\""));
        assert!(json_str.contains("\"links\""));
    }
}
