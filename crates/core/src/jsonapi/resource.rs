use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceIdentifier {
    #[serde(rename = "type")]
    pub resource_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, Value>>,
}

impl ResourceIdentifier {
    pub fn new(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: Some(id.into()),
            lid: None,
            meta: None,
        }
    }

    pub fn with_lid(resource_type: impl Into<String>, lid: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: None,
            lid: Some(lid.into()),
            meta: None,
        }
    }

    pub fn with_meta(mut self, meta: HashMap<String, Value>) -> Self {
        self.meta = Some(meta);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RelationshipData {
    Null,

    One(ResourceIdentifier),

    Many(Vec<ResourceIdentifier>),
}

impl RelationshipData {
    pub fn one(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::One(ResourceIdentifier::new(resource_type, id))
    }

    pub fn many(identifiers: Vec<ResourceIdentifier>) -> Self {
        Self::Many(identifiers)
    }

    pub fn null() -> Self {
        Self::Null
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relationship {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<RelationshipData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, Value>>,
}

impl Relationship {
    pub fn to_one(related: Option<ResourceIdentifier>) -> Self {
        Self {
            data: Some(match related {
                Some(identifier) => RelationshipData::One(identifier),
                None => RelationshipData::Null,
            }),
            links: None,
            meta: None,
        }
    }

    pub fn to_many(related: Vec<ResourceIdentifier>) -> Self {
        Self {
            data: Some(RelationshipData::Many(related)),
            links: None,
            meta: None,
        }
    }

    pub fn with_links(links: HashMap<String, String>) -> Self {
        Self {
            data: None,
            links: Some(links),
            meta: None,
        }
    }

    pub fn add_link(mut self, key: impl Into<String>, url: impl Into<String>) -> Self {
        self.links
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), url.into());
        self
    }

    pub fn with_meta(mut self, meta: HashMap<String, Value>) -> Self {
        self.meta = Some(meta);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceObject {
    #[serde(rename = "type")]
    pub resource_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub lid: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationships: Option<HashMap<String, Relationship>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, Value>>,
}

impl ResourceObject {
    pub fn new(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: Some(id.into()),
            lid: None,
            attributes: None,
            relationships: None,
            links: None,
            meta: None,
        }
    }

    pub fn with_type(resource_type: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: None,
            lid: None,
            attributes: None,
            relationships: None,
            links: None,
            meta: None,
        }
    }

    pub fn set_attribute(mut self, key: impl Into<String>, value: Value) -> Self {
        self.attributes
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value);
        self
    }

    pub fn with_attributes(mut self, attributes: HashMap<String, Value>) -> Self {
        self.attributes = Some(attributes);
        self
    }

    pub fn set_to_one_relationship(
        mut self,
        name: impl Into<String>,
        related: Option<ResourceIdentifier>,
        links: Option<HashMap<String, String>>,
    ) -> Self {
        let mut relationship = Relationship::to_one(related);
        if let Some(l) = links {
            relationship.links = Some(l);
        }
        self.relationships
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), relationship);
        self
    }

    pub fn set_to_many_relationship(
        mut self,
        name: impl Into<String>,
        related: Vec<ResourceIdentifier>,
        links: Option<HashMap<String, String>>,
    ) -> Self {
        let mut relationship = Relationship::to_many(related);
        if let Some(l) = links {
            relationship.links = Some(l);
        }
        self.relationships
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), relationship);
        self
    }

    pub fn add_link(mut self, key: impl Into<String>, url: impl Into<String>) -> Self {
        self.links
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), url.into());
        self
    }

    pub fn with_meta(mut self, meta: HashMap<String, Value>) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn apply_sparse_fieldset(mut self, fields: Option<&[String]>) -> Self {
        if let Some(allowed_fields) = fields {
            if let Some(ref mut attrs) = self.attributes {
                let filtered_attrs: HashMap<String, Value> = attrs
                    .iter()
                    .filter(|(key, _)| allowed_fields.contains(key))
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                self.attributes = Some(filtered_attrs);
            }
        }
        self
    }

    pub fn to_identifier(&self) -> Option<ResourceIdentifier> {
        match (&self.id, &self.lid) {
            (Some(id), _) => Some(ResourceIdentifier {
                resource_type: self.resource_type.clone(),
                id: Some(id.clone()),
                lid: None,
                meta: self.meta.clone(),
            }),
            (None, Some(lid)) => Some(ResourceIdentifier {
                resource_type: self.resource_type.clone(),
                id: None,
                lid: Some(lid.clone()),
                meta: self.meta.clone(),
            }),
            _ => None,
        }
    }

    pub fn relationship_links(
        &self,
        base_path: &str,
        relationship_name: &str,
    ) -> HashMap<String, String> {
        let mut links = HashMap::new();
        if let Some(ref id) = self.id {
            links.insert(
                "self".to_string(),
                format!(
                    "{}/{}/{}/relationships/{}",
                    base_path, self.resource_type, id, relationship_name
                ),
            );
            links.insert(
                "related".to_string(),
                format!(
                    "{}/{}/{}/{}",
                    base_path, self.resource_type, id, relationship_name
                ),
            );
        }
        links
    }

    pub fn add_to_one_with_links(
        self,
        relationship_name: impl Into<String>,
        related: Option<ResourceIdentifier>,
        base_path: &str,
    ) -> Self {
        let name = relationship_name.into();
        let links = self.relationship_links(base_path, &name);
        self.set_to_one_relationship(name, related, Some(links))
    }

    pub fn add_to_many_with_links(
        self,
        relationship_name: impl Into<String>,
        related: Vec<ResourceIdentifier>,
        base_path: &str,
    ) -> Self {
        let name = relationship_name.into();
        let links = self.relationship_links(base_path, &name);
        self.set_to_many_relationship(name, related, Some(links))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resource_identifier() {
        let identifier = ResourceIdentifier::new("users", "1");
        assert_eq!(identifier.resource_type, "users");
        assert_eq!(identifier.id, Some("1".to_string()));
        assert!(identifier.lid.is_none());
    }

    #[test]
    fn test_resource_object_builder() {
        let resource = ResourceObject::new("users", "1")
            .set_attribute("name", json!("John Doe"))
            .set_attribute("email", json!("john@example.com"))
            .add_link("self", "/api/users/1");

        assert_eq!(resource.resource_type, "users");
        assert_eq!(resource.id, Some("1".to_string()));
        assert!(resource.attributes.is_some());
        assert!(resource.links.is_some());
    }

    #[test]
    fn test_to_one_relationship() {
        let related = ResourceIdentifier::new("organizations", "5");
        let relationship = Relationship::to_one(Some(related));

        match relationship.data {
            Some(RelationshipData::One(ref identifier)) => {
                assert_eq!(identifier.resource_type, "organizations");
                assert_eq!(identifier.id, Some("5".to_string()));
            }
            _ => panic!("Expected to-one relationship"),
        }
    }

    #[test]
    fn test_to_many_relationship() {
        let related = vec![
            ResourceIdentifier::new("tags", "1"),
            ResourceIdentifier::new("tags", "2"),
        ];
        let relationship = Relationship::to_many(related.clone());

        match relationship.data {
            Some(RelationshipData::Many(ref identifiers)) => {
                assert_eq!(identifiers.len(), 2);
            }
            _ => panic!("Expected to-many relationship"),
        }
    }

    #[test]
    fn test_apply_sparse_fieldset() {
        let resource = ResourceObject::new("users", "1")
            .set_attribute("name", json!("John"))
            .set_attribute("email", json!("john@example.com"))
            .set_attribute("status", json!("active"));

        let allowed_fields = vec!["name".to_string(), "email".to_string()];
        let filtered = resource.apply_sparse_fieldset(Some(&allowed_fields));

        let attrs = filtered.attributes.unwrap();
        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains_key("name"));
        assert!(attrs.contains_key("email"));
        assert!(!attrs.contains_key("status"));
    }

    #[test]
    fn test_apply_sparse_fieldset_none() {
        let resource = ResourceObject::new("users", "1")
            .set_attribute("name", json!("John"))
            .set_attribute("email", json!("john@example.com"))
            .set_attribute("status", json!("active"));

        let filtered = resource.apply_sparse_fieldset(None);

        let attrs = filtered.attributes.unwrap();
        assert_eq!(attrs.len(), 3);
    }
}
