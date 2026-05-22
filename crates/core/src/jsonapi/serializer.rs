use super::document::JsonApiDocument;
use super::error::ErrorObject;
use super::pagination::{Pagination, PaginationLinks, PaginationMeta};
use super::resource::ResourceObject;

pub struct JsonApiSerializer;

impl JsonApiSerializer {
    pub fn serialize_resource(
        resource: ResourceObject,
        included: Option<Vec<ResourceObject>>,
    ) -> JsonApiDocument {
        let mut doc = JsonApiDocument::new().with_resource(resource);

        if let Some(inc) = included {
            doc = doc.with_included(inc);
        }

        doc
    }

    pub fn serialize_resource_with_fields(
        mut resource: ResourceObject,
        fields: Option<&std::collections::HashMap<String, Vec<String>>>,
        included: Option<Vec<ResourceObject>>,
    ) -> JsonApiDocument {
        if let Some(field_map) = fields {
            if let Some(allowed_fields) = field_map.get(resource.resource_type.as_str()) {
                resource = resource.apply_sparse_fieldset(Some(allowed_fields));
            }
        }

        let mut doc = JsonApiDocument::new().with_resource(resource);

        if let Some(inc) = included {
            let filtered_included = Self::apply_sparse_fieldsets_to_resources(inc, fields);
            doc = doc.with_included(filtered_included);
        }

        doc
    }

    pub fn serialize_collection_with_fields(
        resources: Vec<ResourceObject>,
        total: i64,
        current_page: i64,
        per_page: i64,
        base_url: &str,
        fields: Option<&std::collections::HashMap<String, Vec<String>>>,
        included: Option<Vec<ResourceObject>>,
    ) -> JsonApiDocument {
        let pagination = Pagination::new(total, current_page, per_page, base_url);

        let filtered_resources = resources
            .into_iter()
            .map(|mut resource| {
                if let Some(field_map) = fields {
                    if let Some(allowed_fields) = field_map.get(resource.resource_type.as_str()) {
                        resource = resource.apply_sparse_fieldset(Some(allowed_fields));
                    }
                }
                resource
            })
            .collect();

        let mut doc = JsonApiDocument::new()
            .with_data(filtered_resources)
            .with_meta_map(pagination.meta.to_map())
            .with_links(pagination.links.to_map());

        if let Some(inc) = included {
            let filtered_included = Self::apply_sparse_fieldsets_to_resources(inc, fields);
            doc = doc.with_included(filtered_included);
        }

        doc
    }

    fn apply_sparse_fieldsets_to_resources(
        resources: Vec<ResourceObject>,
        fields: Option<&std::collections::HashMap<String, Vec<String>>>,
    ) -> Vec<ResourceObject> {
        resources
            .into_iter()
            .map(|mut resource| {
                if let Some(field_map) = fields {
                    if let Some(allowed_fields) = field_map.get(resource.resource_type.as_str()) {
                        resource = resource.apply_sparse_fieldset(Some(allowed_fields));
                    }
                }
                resource
            })
            .collect()
    }

    pub fn serialize_collection(
        resources: Vec<ResourceObject>,
        total: i64,
        current_page: i64,
        per_page: i64,
        base_url: &str,
        included: Option<Vec<ResourceObject>>,
    ) -> JsonApiDocument {
        let pagination = Pagination::new(total, current_page, per_page, base_url);

        let mut doc = JsonApiDocument::new()
            .with_data(resources)
            .with_meta_map(pagination.meta.to_map())
            .with_links(pagination.links.to_map());

        if let Some(inc) = included {
            doc = doc.with_included(inc);
        }

        doc
    }

    pub fn serialize_error(error: ErrorObject) -> JsonApiDocument {
        JsonApiDocument::new().with_error(error)
    }

    pub fn serialize_errors(errors: Vec<ErrorObject>) -> JsonApiDocument {
        JsonApiDocument::new().with_errors(errors)
    }

    pub fn serialize_validation_errors(
        errors: &std::collections::HashMap<String, Vec<String>>,
    ) -> JsonApiDocument {
        let error_objects = ErrorObject::validation_errors(errors);
        Self::serialize_errors(error_objects)
    }

    pub fn pagination_meta(total: i64, current_page: i64, per_page: i64) -> PaginationMeta {
        PaginationMeta::new(total, current_page, per_page)
    }

    pub fn pagination_links(
        base_url: &str,
        current_page: i64,
        last_page: i64,
        per_page: i64,
    ) -> PaginationLinks {
        PaginationLinks::new(base_url, current_page, last_page, per_page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonapi::JsonApiData;
    use serde_json::json;

    #[test]
    fn test_serialize_resource() {
        let resource = ResourceObject::new("users", "1")
            .set_attribute("name", json!("John"))
            .set_attribute("email", json!("john@example.com"));

        let doc = JsonApiSerializer::serialize_resource(resource, None);

        assert!(doc.has_data());
        assert!(!doc.has_errors());
        assert!(doc.validate().is_ok());
    }

    #[test]
    fn test_serialize_collection() {
        let resources = vec![
            ResourceObject::new("users", "1"),
            ResourceObject::new("users", "2"),
            ResourceObject::new("users", "3"),
        ];

        let doc =
            JsonApiSerializer::serialize_collection(resources, 100, 2, 25, "/api/users", None);

        assert!(doc.has_data());
        assert!(doc.meta.is_some());
        assert!(doc.links.is_some());

        let meta = doc.meta.as_ref().unwrap();
        assert_eq!(meta.get("total").unwrap(), &json!(100));
        assert_eq!(meta.get("current_page").unwrap(), &json!(2));
        assert_eq!(meta.get("per_page").unwrap(), &json!(25));
        assert_eq!(meta.get("last_page").unwrap(), &json!(4));

        let links = doc.links.as_ref().unwrap();
        assert!(links.contains_key("first"));
        assert!(links.contains_key("last"));
        assert!(links.contains_key("prev"));
        assert!(links.contains_key("next"));
        assert!(links.contains_key("self"));
    }

    #[test]
    fn test_serialize_error() {
        let error = ErrorObject::from_status(404, "Not Found", "User not found");
        let doc = JsonApiSerializer::serialize_error(error);

        assert!(!doc.has_data());
        assert!(doc.has_errors());

        let errors = doc.errors.as_ref().unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].status, Some("404".to_string()));
        assert_eq!(errors[0].title, Some("Not Found".to_string()));
    }

    #[test]
    fn test_serialize_validation_errors() {
        let mut field_errors = std::collections::HashMap::new();
        field_errors.insert("email".to_string(), vec!["Invalid format".to_string()]);
        field_errors.insert(
            "password".to_string(),
            vec!["Too short".to_string(), "No special chars".to_string()],
        );

        let doc = JsonApiSerializer::serialize_validation_errors(&field_errors);

        assert!(doc.has_errors());
        let errors = doc.errors.as_ref().unwrap();
        assert_eq!(errors.len(), 3);

        for error in errors {
            assert_eq!(error.status, Some("422".to_string()));
            assert_eq!(error.title, Some("Validation Error".to_string()));
            assert!(error.source.is_some());
        }
    }

    #[test]
    fn test_serialize_with_included() {
        let resource = ResourceObject::new("posts", "1");
        let included = vec![
            ResourceObject::new("users", "1"),
            ResourceObject::new("users", "2"),
        ];

        let doc = JsonApiSerializer::serialize_resource(resource, Some(included));

        assert!(doc.has_data());
        assert!(doc.included.is_some());
        assert_eq!(doc.included.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_pagination_meta() {
        let meta = JsonApiSerializer::pagination_meta(150, 3, 25);

        assert_eq!(meta.total, 150);
        assert_eq!(meta.current_page, 3);
        assert_eq!(meta.per_page, 25);
        assert_eq!(meta.last_page, 6);
        assert_eq!(meta.from, 51);
        assert_eq!(meta.to, 75);
    }

    #[test]
    fn test_pagination_links() {
        let links = JsonApiSerializer::pagination_links("/api/users", 2, 4, 10);

        assert_eq!(links.first, "/api/users?page[number]=1&page[size]=10");
        assert_eq!(links.last, "/api/users?page[number]=4&page[size]=10");
        assert_eq!(links.self_link, "/api/users?page[number]=2&page[size]=10");
        assert!(links.prev.is_some());
        assert!(links.next.is_some());
    }

    #[test]
    fn test_serialize_resource_with_sparse_fieldsets() {
        let resource = ResourceObject::new("users", "1")
            .set_attribute("name", json!("John"))
            .set_attribute("email", json!("john@example.com"))
            .set_attribute("status", json!("active"));

        let mut fields = std::collections::HashMap::new();
        fields.insert(
            "users".to_string(),
            vec!["name".to_string(), "email".to_string()],
        );

        let doc = JsonApiSerializer::serialize_resource_with_fields(resource, Some(&fields), None);

        assert!(doc.has_data());
        if let Some(JsonApiData::One(data)) = doc.data {
            let attrs = data.attributes.as_ref().unwrap();
            assert_eq!(attrs.len(), 2);
            assert!(attrs.contains_key("name"));
            assert!(attrs.contains_key("email"));
            assert!(!attrs.contains_key("status"));
        } else {
            panic!("Expected single resource in data");
        }
    }

    #[test]
    fn test_serialize_collection_with_sparse_fieldsets() {
        let resources = vec![
            ResourceObject::new("users", "1")
                .set_attribute("name", json!("John"))
                .set_attribute("email", json!("john@example.com"))
                .set_attribute("status", json!("active")),
            ResourceObject::new("users", "2")
                .set_attribute("name", json!("Jane"))
                .set_attribute("email", json!("jane@example.com"))
                .set_attribute("status", json!("inactive")),
        ];

        let mut fields = std::collections::HashMap::new();
        fields.insert("users".to_string(), vec!["name".to_string()]);

        let doc = JsonApiSerializer::serialize_collection_with_fields(
            resources,
            100,
            1,
            25,
            "/api/users",
            Some(&fields),
            None,
        );

        assert!(doc.has_data());
        if let Some(JsonApiData::Many(data)) = doc.data {
            assert_eq!(data.len(), 2);
            for resource in data {
                let attrs = resource.attributes.as_ref().unwrap();
                assert_eq!(attrs.len(), 1);
                assert!(attrs.contains_key("name"));
            }
        } else {
            panic!("Expected collection of resources in data");
        }
    }

    #[test]
    fn test_serialize_with_sparse_fieldsets_and_included() {
        let resource = ResourceObject::new("posts", "1")
            .set_attribute("title", json!("My Post"))
            .set_attribute("body", json!("Content"));

        let included = vec![ResourceObject::new("users", "1")
            .set_attribute("name", json!("John"))
            .set_attribute("email", json!("john@example.com"))];

        let mut fields = std::collections::HashMap::new();
        fields.insert("posts".to_string(), vec!["title".to_string()]);
        fields.insert("users".to_string(), vec!["name".to_string()]);

        let doc = JsonApiSerializer::serialize_resource_with_fields(
            resource,
            Some(&fields),
            Some(included),
        );

        assert!(doc.has_data());
        assert!(doc.included.is_some());

        if let Some(JsonApiData::One(data)) = doc.data {
            let attrs = data.attributes.as_ref().unwrap();
            assert_eq!(attrs.len(), 1);
            assert!(attrs.contains_key("title"));
            assert!(!attrs.contains_key("body"));
        } else {
            panic!("Expected single resource in data");
        }

        let inc = doc.included.as_ref().unwrap();
        assert_eq!(inc.len(), 1);
        let inc_attrs = inc[0].attributes.as_ref().unwrap();
        assert_eq!(inc_attrs.len(), 1);
        assert!(inc_attrs.contains_key("name"));
        assert!(!inc_attrs.contains_key("email"));
    }
}
