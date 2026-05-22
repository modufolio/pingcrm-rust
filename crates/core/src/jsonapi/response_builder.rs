use crate::jsonapi::{QueryParams, ResourceObject};
use serde_json::{json, Value};

pub fn build_index_response(
    resources: Vec<ResourceObject>,
    included: Vec<ResourceObject>,
    total_count: i64,
    params: &QueryParams,
    base_url: &str,
) -> Value {
    let total_pages = ((total_count as f64 / params.page.size as f64).ceil() as i64).max(1);

    let primary_type = resources
        .first()
        .map(|r| r.resource_type.clone())
        .unwrap_or_default();
    let filtered_resources = if !primary_type.is_empty() {
        apply_sparse_fieldsets(resources, params, &primary_type)
    } else {
        resources
    };

    let filtered_included = filter_included_by_sparse_fieldsets(included, params);

    let mut response = json!({
        "data": filtered_resources,
        "meta": {
            "total": total_count,
            "page": {
                "number": params.page.number,
                "size": params.page.size,
                "total_pages": total_pages
            }
        },
        "links": {
            "self": format!(
                "{}?page[number]={}&page[size]={}",
                base_url, params.page.number, params.page.size
            ),
            "first": format!(
                "{}?page[number]=1&page[size]={}",
                base_url, params.page.size
            ),
            "last": format!(
                "{}?page[number]={}&page[size]={}",
                base_url, total_pages, params.page.size
            )
        }
    });

    if params.page.number > 1 {
        response["links"]["prev"] = json!(format!(
            "{}?page[number]={}&page[size]={}",
            base_url,
            params.page.number - 1,
            params.page.size
        ));
    }

    if params.page.number < total_pages {
        response["links"]["next"] = json!(format!(
            "{}?page[number]={}&page[size]={}",
            base_url,
            params.page.number + 1,
            params.page.size
        ));
    }

    if !filtered_included.is_empty() {
        response["included"] = json!(filtered_included);
    }

    response
}

pub fn build_show_response(
    resource: ResourceObject,
    included: Vec<ResourceObject>,
    self_url: &str,
) -> Value {
    let mut response = json!({
        "data": resource,
        "links": {
            "self": self_url
        }
    });

    if !included.is_empty() {
        response["included"] = json!(included);
    }

    response
}

pub fn build_show_response_with_params(
    resource: ResourceObject,
    included: Vec<ResourceObject>,
    params: &QueryParams,
    self_url: &str,
) -> Value {
    let resource_type = &resource.resource_type.clone();
    let filtered_resource = apply_sparse_fieldsets(vec![resource], params, resource_type)
        .into_iter()
        .next()
        .unwrap();

    let filtered_included = filter_included_by_sparse_fieldsets(included, params);

    let mut response = json!({
        "data": filtered_resource,
        "links": {
            "self": self_url
        }
    });

    if !filtered_included.is_empty() {
        response["included"] = json!(filtered_included);
    }

    response
}

pub fn build_mutation_response(resource: ResourceObject, self_url: &str) -> Value {
    json!({
        "data": resource,
        "links": {
            "self": self_url
        }
    })
}

fn filter_included_by_sparse_fieldsets(
    included: Vec<ResourceObject>,
    params: &QueryParams,
) -> Vec<ResourceObject> {
    use std::collections::HashMap;

    let mut by_type: HashMap<String, Vec<ResourceObject>> = HashMap::new();
    for resource in included {
        by_type
            .entry(resource.resource_type.clone())
            .or_insert_with(Vec::new)
            .push(resource);
    }

    by_type
        .into_iter()
        .flat_map(|(resource_type, resources)| {
            apply_sparse_fieldsets(resources, params, &resource_type)
        })
        .collect()
}

pub fn apply_sparse_fieldsets(
    resources: Vec<ResourceObject>,
    params: &QueryParams,
    resource_type: &str,
) -> Vec<ResourceObject> {
    if let Some(fields) = params.fields.get(resource_type) {
        let field_strings: Vec<String> = fields.iter().map(|f| f.to_string()).collect();
        resources
            .into_iter()
            .map(|resource| resource.apply_sparse_fieldset(Some(&field_strings)))
            .collect()
    } else {
        resources
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonapi::{PageParams, QueryParams};

    #[test]
    fn test_build_index_response() {
        let params = QueryParams {
            page: PageParams {
                number: 1,
                size: 10,
            },
            ..Default::default()
        };

        let resources = vec![];
        let included = vec![];
        let response = build_index_response(resources, included, 25, &params, "/api/v1/products");

        assert_eq!(response["meta"]["total"], 25);
        assert_eq!(response["meta"]["page"]["number"], 1);
        assert_eq!(response["meta"]["page"]["size"], 10);
        assert_eq!(response["meta"]["page"]["total_pages"], 3);
    }

    #[test]
    fn test_pagination_links() {
        let params = QueryParams {
            page: PageParams {
                number: 2,
                size: 10,
            },
            ..Default::default()
        };

        let response = build_index_response(vec![], vec![], 30, &params, "/api/v1/products");

        assert!(response["links"]["prev"].is_string());
        assert!(response["links"]["next"].is_string());
    }
}
