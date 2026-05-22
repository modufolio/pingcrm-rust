use super::document::{JsonApiData, JsonApiDocument};
use super::resource::{ResourceIdentifier, ResourceObject};
use crate::error::{AppError, AppResult};
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct JsonApiRequest {
    document: JsonApiDocument,
}

impl JsonApiRequest {
    pub fn new(document: JsonApiDocument) -> Self {
        Self { document }
    }

    pub fn validate(&self) -> AppResult<()> {
        if self.document.data.is_none() {
            return Err(AppError::BadRequest(
                "JSON:API request must contain 'data'".to_string(),
            ));
        }

        if self.document.errors.is_some() {
            return Err(AppError::BadRequest(
                "JSON:API request cannot contain 'errors'".to_string(),
            ));
        }

        Ok(())
    }

    pub fn resource(&self) -> AppResult<&ResourceObject> {
        match &self.document.data {
            Some(JsonApiData::One(resource)) => Ok(resource),
            Some(JsonApiData::Many(resources)) if resources.len() == 1 => Ok(&resources[0]),
            Some(JsonApiData::Many(_)) => Err(AppError::BadRequest(
                "Expected single resource but got multiple".to_string(),
            )),
            Some(JsonApiData::Null) | None => Err(AppError::BadRequest(
                "Request data is null or missing".to_string(),
            )),
        }
    }

    pub fn resources(&self) -> AppResult<Vec<&ResourceObject>> {
        match &self.document.data {
            Some(JsonApiData::Many(resources)) => Ok(resources.iter().collect()),
            Some(JsonApiData::One(resource)) => Ok(vec![resource.as_ref()]),
            Some(JsonApiData::Null) | None => Ok(Vec::new()),
        }
    }

    pub fn resource_type(&self) -> AppResult<&str> {
        let resource = self.resource()?;
        Ok(&resource.resource_type)
    }

    pub fn resource_id(&self) -> AppResult<Option<&str>> {
        let resource = self.resource()?;
        Ok(resource.id.as_deref())
    }

    pub fn attributes_raw(&self) -> AppResult<&HashMap<String, Value>> {
        let resource = self.resource()?;
        resource
            .attributes
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("Resource has no attributes".to_string()))
    }

    pub fn attributes<T: DeserializeOwned>(&self) -> AppResult<T> {
        let attrs = self.attributes_raw()?;
        let value = serde_json::to_value(attrs)
            .map_err(|e| AppError::BadRequest(format!("Failed to serialize attributes: {}", e)))?;
        serde_json::from_value(value)
            .map_err(|e| AppError::BadRequest(format!("Failed to deserialize attributes: {}", e)))
    }

    pub fn attribute(&self, name: &str) -> AppResult<Option<&Value>> {
        let attrs = self.attributes_raw()?;
        Ok(attrs.get(name))
    }

    pub fn relationships(
        &self,
    ) -> AppResult<Option<&HashMap<String, super::resource::Relationship>>> {
        let resource = self.resource()?;
        Ok(resource.relationships.as_ref())
    }

    pub fn relationship(&self, name: &str) -> AppResult<Option<&super::resource::Relationship>> {
        let relationships = self.relationships()?;
        Ok(relationships.and_then(|r| r.get(name)))
    }

    pub fn relationship_identifiers(&self, name: &str) -> AppResult<Vec<ResourceIdentifier>> {
        let relationship = self
            .relationship(name)?
            .ok_or_else(|| AppError::BadRequest(format!("Relationship '{}' not found", name)))?;

        match &relationship.data {
            Some(super::resource::RelationshipData::One(id)) => Ok(vec![id.clone()]),
            Some(super::resource::RelationshipData::Many(ids)) => Ok(ids.clone()),
            Some(super::resource::RelationshipData::Null) | None => Ok(Vec::new()),
        }
    }

    pub fn document(&self) -> &JsonApiDocument {
        &self.document
    }

    pub fn meta(&self) -> Option<&HashMap<String, Value>> {
        self.document.meta.as_ref()
    }
}

impl<S> FromRequest<S> for JsonApiRequest
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let body = axum::body::to_bytes(req.into_body(), usize::MAX)
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read request body: {}", e)))?;

        let document: JsonApiDocument = serde_json::from_slice(&body)
            .map_err(|e| AppError::BadRequest(format!("Invalid JSON:API document: {}", e)))?;

        let request = JsonApiRequest::new(document);
        request.validate()?;

        Ok(request)
    }
}

pub struct JsonApiRequestBuilder {
    resource: ResourceObject,
}

impl JsonApiRequestBuilder {
    pub fn new(resource_type: impl Into<String>) -> Self {
        Self {
            resource: ResourceObject::with_type(resource_type),
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.resource.id = Some(id.into());
        self
    }

    pub fn attribute(mut self, name: impl Into<String>, value: Value) -> Self {
        if self.resource.attributes.is_none() {
            self.resource.attributes = Some(HashMap::new());
        }

        if let Some(ref mut map) = self.resource.attributes {
            map.insert(name.into(), value);
        }

        self
    }

    pub fn attributes(mut self, attrs: HashMap<String, Value>) -> Self {
        self.resource.attributes = Some(attrs);
        self
    }

    pub fn relationship_one(
        mut self,
        name: impl Into<String>,
        resource_type: impl Into<String>,
        id: impl Into<String>,
    ) -> Self {
        let identifier = ResourceIdentifier::new(resource_type, id);
        let relationship = super::resource::Relationship {
            data: Some(super::resource::RelationshipData::One(identifier)),
            links: None,
            meta: None,
        };

        if self.resource.relationships.is_none() {
            self.resource.relationships = Some(HashMap::new());
        }

        if let Some(ref mut rels) = self.resource.relationships {
            rels.insert(name.into(), relationship);
        }

        self
    }

    pub fn relationship_many(
        mut self,
        name: impl Into<String>,
        identifiers: Vec<ResourceIdentifier>,
    ) -> Self {
        let relationship = super::resource::Relationship {
            data: Some(super::resource::RelationshipData::Many(identifiers)),
            links: None,
            meta: None,
        };

        if self.resource.relationships.is_none() {
            self.resource.relationships = Some(HashMap::new());
        }

        if let Some(ref mut rels) = self.resource.relationships {
            rels.insert(name.into(), relationship);
        }

        self
    }

    pub fn build(self) -> JsonApiRequest {
        let document = JsonApiDocument {
            jsonapi: Some(super::document::JsonApiVersion::default()),
            data: Some(JsonApiData::One(Box::new(self.resource))),
            errors: None,
            included: None,
            meta: None,
            links: None,
        };

        JsonApiRequest::new(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_builder() {
        let request = JsonApiRequestBuilder::new("users")
            .id("123")
            .attribute("name", json!("John"))
            .attribute("email", json!("john@example.com"))
            .build();

        assert_eq!(request.resource_type().unwrap(), "users");
        assert_eq!(request.resource_id().unwrap(), Some("123"));
    }

    #[test]
    fn test_request_attributes() {
        let request = JsonApiRequestBuilder::new("users")
            .attribute("name", json!("Jane"))
            .build();

        let name = request.attribute("name").unwrap();
        assert_eq!(name, Some(&json!("Jane")));
    }

    #[test]
    fn test_request_relationship() {
        let request = JsonApiRequestBuilder::new("posts")
            .relationship_one("author", "users", "123")
            .build();

        let ids = request.relationship_identifiers("author").unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0].resource_type, "users");
        assert_eq!(ids[0].id, Some("123".to_string()));
    }

    #[test]
    fn test_validate_missing_data() {
        let document = JsonApiDocument {
            jsonapi: Some(super::super::document::JsonApiVersion::default()),
            data: None,
            errors: None,
            included: None,
            meta: None,
            links: None,
        };

        let request = JsonApiRequest::new(document);
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_validate_with_errors() {
        let document = JsonApiDocument {
            jsonapi: Some(super::super::document::JsonApiVersion::default()),
            data: Some(JsonApiData::One(Box::new(ResourceObject::new(
                "users", "1",
            )))),
            errors: Some(vec![]),
            included: None,
            meta: None,
            links: None,
        };

        let request = JsonApiRequest::new(document);
        assert!(request.validate().is_err());
    }
}
