use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct JsonApiDocument<T> {
    pub data: T,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub included: Option<Vec<JsonApiResource>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<JsonApiLinks>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<JsonApiError>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub jsonapi: Option<JsonApiVersion>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct JsonApiVersion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct JsonApiResource {
    #[serde(rename = "type")]
    pub resource_type: String,

    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationships: Option<HashMap<String, JsonApiRelationship>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<JsonApiLinks>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct JsonApiRelationship {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonApiRelationshipData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<JsonApiRelationshipLinks>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct JsonApiRelationshipLinks {
    #[serde(rename = "self", skip_serializing_if = "Option::is_none")]
    pub self_link: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub related: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(untagged)]
pub enum JsonApiRelationshipData {
    One(JsonApiResourceIdentifier),

    Many(Vec<JsonApiResourceIdentifier>),
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct JsonApiResourceIdentifier {
    #[serde(rename = "type")]
    pub resource_type: String,

    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct JsonApiLinks {
    #[serde(rename = "self", skip_serializing_if = "Option::is_none")]
    pub self_link: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub related: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct JsonApiError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<JsonApiErrorLinks>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<JsonApiErrorSource>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct JsonApiErrorLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct JsonApiErrorSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_serialization() {
        let resource = JsonApiResource {
            resource_type: "contacts".to_string(),
            id: "1".to_string(),
            attributes: Some(serde_json::json!({
                "first_name": "John",
                "last_name": "Doe"
            })),
            relationships: None,
            links: None,
            meta: None,
        };

        let json = serde_json::to_string(&resource).unwrap();
        assert!(json.contains("\"type\":\"contacts\""));
        assert!(json.contains("\"id\":\"1\""));
    }

    #[test]
    fn test_document_serialization() {
        let doc = JsonApiDocument {
            data: vec![JsonApiResource {
                resource_type: "contacts".to_string(),
                id: "1".to_string(),
                attributes: None,
                relationships: None,
                links: None,
                meta: None,
            }],
            included: None,
            meta: None,
            links: None,
            errors: None,
            jsonapi: Some(JsonApiVersion {
                version: Some("1.0".to_string()),
                meta: None,
            }),
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("\"data\""));
        assert!(json.contains("\"jsonapi\""));
    }

    #[test]
    fn test_relationship_one() {
        let rel = JsonApiRelationship {
            data: Some(JsonApiRelationshipData::One(JsonApiResourceIdentifier {
                resource_type: "organizations".to_string(),
                id: "42".to_string(),
                meta: None,
            })),
            links: None,
            meta: None,
        };

        let json = serde_json::to_string(&rel).unwrap();
        assert!(json.contains("\"type\":\"organizations\""));
        assert!(json.contains("\"id\":\"42\""));
    }

    #[test]
    fn test_relationship_many() {
        let rel = JsonApiRelationship {
            data: Some(JsonApiRelationshipData::Many(vec![
                JsonApiResourceIdentifier {
                    resource_type: "contacts".to_string(),
                    id: "1".to_string(),
                    meta: None,
                },
                JsonApiResourceIdentifier {
                    resource_type: "contacts".to_string(),
                    id: "2".to_string(),
                    meta: None,
                },
            ])),
            links: None,
            meta: None,
        };

        let json = serde_json::to_string(&rel).unwrap();
        assert!(json.contains("["));
        assert!(json.contains("\"id\":\"1\""));
        assert!(json.contains("\"id\":\"2\""));
    }
}
