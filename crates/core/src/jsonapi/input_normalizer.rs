use crate::error::{AppError, AppResult};
use crate::negotiation::Negotiator;
use serde_json::{Map, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipValue {
    One(i32),

    Many(Vec<i32>),

    Null,
}

impl RelationshipValue {
    pub fn is_one(&self) -> bool {
        matches!(self, Self::One(_))
    }

    pub fn is_many(&self) -> bool {
        matches!(self, Self::Many(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn as_one(&self) -> Option<i32> {
        match self {
            Self::One(id) => Some(*id),
            _ => None,
        }
    }

    pub fn as_many(&self) -> Option<&[i32]> {
        match self {
            Self::Many(ids) => Some(ids),
            _ => None,
        }
    }

    pub fn to_json(&self) -> Value {
        match self {
            Self::One(id) => Value::Number((*id).into()),
            Self::Many(ids) => {
                Value::Array(ids.iter().map(|id| Value::Number((*id).into())).collect())
            }
            Self::Null => Value::Null,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NormalizedPayload {
    pub attributes: Value,

    pub relationships: HashMap<String, RelationshipValue>,
}

impl NormalizedPayload {
    pub fn empty() -> Self {
        Self {
            attributes: Value::Object(Map::new()),
            relationships: HashMap::new(),
        }
    }

    pub fn merge(&self) -> HashMap<String, Value> {
        let mut merged = HashMap::new();

        if let Some(attrs) = self.attributes.as_object() {
            for (key, value) in attrs {
                merged.insert(key.clone(), value.clone());
            }
        }

        for (key, value) in &self.relationships {
            merged.insert(key.clone(), value.to_json());
        }

        merged
    }

    pub fn attribute(&self, key: &str) -> Option<&Value> {
        self.attributes.as_object().and_then(|obj| obj.get(key))
    }

    pub fn relationship(&self, key: &str) -> Option<&RelationshipValue> {
        self.relationships.get(key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestFormat {
    JsonApi,

    Json,

    Form,
}

pub struct InputNormalizer {
    negotiator: Negotiator,
}

impl InputNormalizer {
    const SUPPORTED_FORMATS: &'static [&'static str] = &[
        "application/vnd.api+json",
        "application/json",
        "application/x-www-form-urlencoded",
    ];

    pub fn new() -> Self {
        Self {
            negotiator: Negotiator::new(),
        }
    }

    pub fn normalize(
        &self,
        payload: &Value,
        content_type: &str,
        expected_type: &str,
    ) -> AppResult<NormalizedPayload> {
        let format = self.detect_format(content_type);

        match format {
            RequestFormat::JsonApi => self.normalize_jsonapi(payload, expected_type),
            RequestFormat::Json => self.normalize_json(payload),
            RequestFormat::Form => self.normalize_form(payload),
        }
    }

    pub fn detect_format(&self, content_type: &str) -> RequestFormat {
        if content_type.contains("application/vnd.api+json") {
            RequestFormat::JsonApi
        } else if content_type.contains("application/json") {
            RequestFormat::Json
        } else if content_type.contains("application/x-www-form-urlencoded") {
            RequestFormat::Form
        } else {
            RequestFormat::Json
        }
    }

    pub fn is_supported(&self, content_type: &str) -> bool {
        self.negotiator
            .negotiate(content_type, Self::SUPPORTED_FORMATS)
            .is_some()
    }

    fn normalize_jsonapi(
        &self,
        payload: &Value,
        expected_type: &str,
    ) -> AppResult<NormalizedPayload> {
        let data = payload.get("data").ok_or_else(|| {
            AppError::BadRequest("JSON:API request must have a 'data' member".to_string())
        })?;

        if !data.is_object() {
            return Err(AppError::BadRequest(
                "JSON:API 'data' member must be an object".to_string(),
            ));
        }

        let resource_type = data
            .get("type")
            .and_then(|t: &Value| t.as_str())
            .ok_or_else(|| {
                AppError::BadRequest(
                    "JSON:API resource object must have a 'type' member".to_string(),
                )
            })?;

        if resource_type != expected_type {
            return Err(AppError::BadRequest(format!(
                "Expected resource type '{}', got '{}'",
                expected_type, resource_type
            )));
        }

        let attributes = data
            .get("attributes")
            .cloned()
            .unwrap_or(Value::Object(Map::new()));

        if !attributes.is_object() {
            return Err(AppError::BadRequest(
                "JSON:API 'attributes' member must be an object".to_string(),
            ));
        }

        let mut relationships: HashMap<String, RelationshipValue> = HashMap::new();

        if let Some(rels) = data.get("relationships") {
            if !rels.is_object() {
                return Err(AppError::BadRequest(
                    "JSON:API 'relationships' member must be an object".to_string(),
                ));
            }

            if let Some(rels_obj) = rels.as_object() {
                for (name, rel_data) in rels_obj {
                    let value = Self::normalize_relationship(rel_data, name)?;
                    relationships.insert(name.to_string(), value);
                }
            }
        }

        Ok(NormalizedPayload {
            attributes,
            relationships,
        })
    }

    fn normalize_relationship(rel_data: &Value, name: &str) -> AppResult<RelationshipValue> {
        let data = rel_data.get("data").ok_or_else(|| {
            AppError::BadRequest(format!("Relationship '{}' must have a 'data' member", name))
        })?;

        if data.is_null() {
            return Ok(RelationshipValue::Null);
        }

        if let Some(arr) = data.as_array() {
            let mut ids: Vec<i32> = Vec::new();
            for (index, item) in arr.iter().enumerate() {
                if !item.is_object() {
                    return Err(AppError::BadRequest(format!(
                        "Relationship '{}' array item at index {} must be an object",
                        name, index
                    )));
                }

                let id = Self::extract_id_from_resource_identifier(item, name)?;
                ids.push(id);
            }
            return Ok(RelationshipValue::Many(ids));
        }

        if data.is_object() {
            let id = Self::extract_id_from_resource_identifier(data, name)?;
            return Ok(RelationshipValue::One(id));
        }

        Err(AppError::BadRequest(format!(
            "Relationship '{}' data must be null, an object, or an array",
            name
        )))
    }

    fn extract_id_from_resource_identifier(obj: &Value, rel_name: &str) -> AppResult<i32> {
        let has_type = obj.get("type").is_some();
        let id_value = obj.get("id");

        if !has_type || id_value.is_none() {
            return Err(AppError::BadRequest(format!(
                "Relationship '{}' resource identifier must have 'type' and 'id' members",
                rel_name
            )));
        }

        Self::normalize_id(id_value.unwrap())
    }

    fn normalize_id(id: &Value) -> AppResult<i32> {
        if let Some(num) = id.as_i64() {
            return Ok(num as i32);
        }

        if let Some(s) = id.as_str() {
            if let Ok(num) = s.parse::<i32>() {
                return Ok(num);
            }
        }

        Err(AppError::BadRequest(format!(
            "Resource ID must be numeric, got {:?}",
            id
        )))
    }

    fn normalize_json(&self, payload: &Value) -> AppResult<NormalizedPayload> {
        let obj = payload.as_object().ok_or_else(|| {
            AppError::BadRequest("Plain JSON payload must be an object".to_string())
        })?;

        let mut attributes: Map<String, Value> = Map::new();
        let mut relationships: HashMap<String, RelationshipValue> = HashMap::new();

        for (key, value) in obj {
            if key.ends_with("_ids") {
                let rel_name = &key[..key.len() - 4];

                let arr = value.as_array().ok_or_else(|| {
                    AppError::BadRequest(format!("Field '{}' must be an array", key))
                })?;

                let ids: Result<Vec<i32>, _> = arr
                    .iter()
                    .map(|v: &Value| {
                        v.as_i64().map(|i| i as i32).ok_or_else(|| {
                            AppError::BadRequest(format!("Field '{}' must contain integers", key))
                        })
                    })
                    .collect();

                relationships.insert(rel_name.to_string(), RelationshipValue::Many(ids?));
            } else if key.ends_with("_id") {
                let rel_name = &key[..key.len() - 3];

                if value.is_null() {
                    relationships.insert(rel_name.to_string(), RelationshipValue::Null);
                } else {
                    let id = value.as_i64().ok_or_else(|| {
                        AppError::BadRequest(format!("Field '{}' must be an integer", key))
                    })? as i32;

                    relationships.insert(rel_name.to_string(), RelationshipValue::One(id));
                }
            } else {
                attributes.insert(key.clone(), value.clone());
            }
        }

        Ok(NormalizedPayload {
            attributes: Value::Object(attributes),
            relationships,
        })
    }

    fn normalize_form(&self, _payload: &Value) -> AppResult<NormalizedPayload> {
        Err(AppError::BadRequest(
            "Form data normalization not yet implemented".to_string(),
        ))
    }
}

impl Default for InputNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_normalize_jsonapi_single_resource() {
        let normalizer = InputNormalizer::new();
        let payload = json!({
            "data": {
                "type": "products",
                "attributes": {
                    "name": "Test Product",
                    "price": 99.99
                }
            }
        });

        let result = normalizer.normalize(&payload, "application/vnd.api+json", "products");
        assert!(result.is_ok());

        let normalized = result.unwrap();
        assert_eq!(normalized.attribute("name"), Some(&json!("Test Product")));
        assert_eq!(normalized.attribute("price"), Some(&json!(99.99)));
    }

    #[test]
    fn test_normalize_jsonapi_with_to_one_relationship() {
        let normalizer = InputNormalizer::new();
        let payload = json!({
            "data": {
                "type": "products",
                "attributes": {
                    "name": "Test Product"
                },
                "relationships": {
                    "brand": {
                        "data": {"type": "brands", "id": "5"}
                    }
                }
            }
        });

        let result = normalizer.normalize(&payload, "application/vnd.api+json", "products");
        assert!(result.is_ok());

        let normalized = result.unwrap();
        assert_eq!(
            normalized.relationship("brand"),
            Some(&RelationshipValue::One(5))
        );
    }

    #[test]
    fn test_normalize_jsonapi_with_to_many_relationship() {
        let normalizer = InputNormalizer::new();
        let payload = json!({
            "data": {
                "type": "products",
                "attributes": {
                    "name": "Test Product"
                },
                "relationships": {
                    "categories": {
                        "data": [
                            {"type": "categories", "id": "1"},
                            {"type": "categories", "id": "2"}
                        ]
                    }
                }
            }
        });

        let result = normalizer.normalize(&payload, "application/vnd.api+json", "products");
        assert!(result.is_ok());

        let normalized = result.unwrap();
        assert_eq!(
            normalized.relationship("categories"),
            Some(&RelationshipValue::Many(vec![1, 2]))
        );
    }

    #[test]
    fn test_normalize_plain_json() {
        let normalizer = InputNormalizer::new();
        let payload = json!({
            "name": "Test Product",
            "price": 99.99,
            "brand_id": 5,
            "category_ids": [1, 2, 3]
        });

        let result = normalizer.normalize(&payload, "application/json", "products");
        assert!(result.is_ok());

        let normalized = result.unwrap();
        assert_eq!(normalized.attribute("name"), Some(&json!("Test Product")));
        assert_eq!(normalized.attribute("price"), Some(&json!(99.99)));
        assert_eq!(
            normalized.relationship("brand"),
            Some(&RelationshipValue::One(5))
        );

        assert_eq!(
            normalized.relationship("category"),
            Some(&RelationshipValue::Many(vec![1, 2, 3]))
        );
    }

    #[test]
    fn test_detect_format() {
        let normalizer = InputNormalizer::new();

        assert_eq!(
            normalizer.detect_format("application/vnd.api+json"),
            RequestFormat::JsonApi
        );
        assert_eq!(
            normalizer.detect_format("application/json"),
            RequestFormat::Json
        );
        assert_eq!(
            normalizer.detect_format("application/x-www-form-urlencoded"),
            RequestFormat::Form
        );
    }

    #[test]
    fn test_merge_data() {
        let payload = NormalizedPayload {
            attributes: json!({"name": "Test", "price": 99.99}),
            relationships: {
                let mut map = HashMap::new();
                map.insert("brand".to_string(), RelationshipValue::One(5));
                map
            },
        };

        let merged = payload.merge();
        assert_eq!(merged.get("name"), Some(&json!("Test")));
        assert_eq!(merged.get("price"), Some(&json!(99.99)));
        assert_eq!(merged.get("brand"), Some(&json!(5)));
    }

    #[test]
    fn test_invalid_type_mismatch() {
        let normalizer = InputNormalizer::new();
        let payload = json!({
            "data": {
                "type": "users",
                "attributes": {"name": "Test"}
            }
        });

        let result = normalizer.normalize(&payload, "application/vnd.api+json", "products");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_data_member() {
        let normalizer = InputNormalizer::new();
        let payload = json!({
            "attributes": {"name": "Test"}
        });

        let result = normalizer.normalize(&payload, "application/vnd.api+json", "products");
        assert!(result.is_err());
    }
}
