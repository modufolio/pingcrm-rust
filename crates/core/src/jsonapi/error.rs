use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ErrorSource>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
}

impl ErrorObject {
    pub fn new() -> Self {
        Self {
            id: None,
            links: None,
            status: None,
            code: None,
            title: None,
            detail: None,
            source: None,
            meta: None,
        }
    }

    pub fn from_status(status: u16, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new()
            .with_status(status)
            .with_title(title)
            .with_detail(detail)
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_link(mut self, key: impl Into<String>, url: impl Into<String>) -> Self {
        self.links
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), url.into());
        self
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = Some(status.to_string());
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_pointer(mut self, pointer: impl Into<String>) -> Self {
        self.source = Some(ErrorSource {
            pointer: Some(pointer.into()),
            parameter: None,
            header: None,
        });
        self
    }

    pub fn with_parameter(mut self, parameter: impl Into<String>) -> Self {
        self.source = Some(ErrorSource {
            pointer: None,
            parameter: Some(parameter.into()),
            header: None,
        });
        self
    }

    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.source = Some(ErrorSource {
            pointer: None,
            parameter: None,
            header: Some(header.into()),
        });
        self
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: Value) -> Self {
        self.meta
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value);
        self
    }

    pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        let field_name = field.into();
        Self::new()
            .with_status(422)
            .with_title("Validation Error")
            .with_detail(message)
            .with_pointer(format!("/data/attributes/{}", field_name))
    }

    pub fn validation_errors(errors: &HashMap<String, Vec<String>>) -> Vec<ErrorObject> {
        errors
            .iter()
            .flat_map(|(field, messages)| {
                let field = field.clone();
                messages
                    .iter()
                    .map(move |message| Self::validation_error(&field, message))
            })
            .collect()
    }
}

impl Default for ErrorObject {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_error_builder() {
        let error = ErrorObject::new()
            .with_status(404)
            .with_title("Not Found")
            .with_detail("Resource not found")
            .with_code("RESOURCE_NOT_FOUND");

        assert_eq!(error.status, Some("404".to_string()));
        assert_eq!(error.title, Some("Not Found".to_string()));
        assert_eq!(error.code, Some("RESOURCE_NOT_FOUND".to_string()));
    }

    #[test]
    fn test_validation_error() {
        let error = ErrorObject::validation_error("email", "Invalid email format");

        assert_eq!(error.status, Some("422".to_string()));
        assert_eq!(error.title, Some("Validation Error".to_string()));
        assert_eq!(error.detail, Some("Invalid email format".to_string()));
        assert!(error.source.is_some());
        assert_eq!(
            error.source.unwrap().pointer,
            Some("/data/attributes/email".to_string())
        );
    }

    #[test]
    fn test_validation_errors_from_map() {
        let mut errors = HashMap::new();
        errors.insert("email".to_string(), vec!["Invalid format".to_string()]);
        errors.insert(
            "password".to_string(),
            vec!["Too short".to_string(), "No special characters".to_string()],
        );

        let error_objects = ErrorObject::validation_errors(&errors);

        assert_eq!(error_objects.len(), 3);
        assert!(error_objects
            .iter()
            .all(|e| e.status == Some("422".to_string())));
    }

    #[test]
    fn test_serialization() {
        let error = ErrorObject::new()
            .with_status(500)
            .with_title("Internal Server Error")
            .with_detail("Something went wrong")
            .with_meta("timestamp", json!("2025-01-16T12:00:00Z"));

        let json_str = serde_json::to_string(&error).unwrap();
        assert!(json_str.contains("\"status\":\"500\""));
        assert!(json_str.contains("\"title\""));
        assert!(json_str.contains("\"meta\""));
    }
}
