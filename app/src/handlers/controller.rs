use crate::app::App;
use appkit_core::error::AppError;
use appkit_core::jsonapi::QueryParams;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::Value;

pub struct V1Controller;

impl V1Controller {
    pub async fn index(
        State(state): State<App>,
        Path(resource): Path<String>,
        query: QueryParams,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;
        let handler = registry.get(&resource)?;

        let response: Value = handler.index(query).await?;

        Ok((
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(response),
        ))
    }

    pub async fn create(
        State(state): State<App>,
        Path(resource): Path<String>,
        Json(payload): Json<Value>,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;
        let handler = registry.get(&resource)?;

        let response: Value = handler.create(payload).await?;

        Ok((
            StatusCode::CREATED,
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(response),
        ))
    }

    pub async fn show(
        State(state): State<App>,
        Path((resource, id)): Path<(String, i32)>,
        query: QueryParams,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;
        let handler = registry.get(&resource)?;

        let response: Value = handler.show(id, query).await?;

        Ok((
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(response),
        ))
    }

    pub async fn update(
        State(state): State<App>,
        Path((resource, id)): Path<(String, i32)>,
        query: QueryParams,
        Json(payload): Json<Value>,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;
        let handler = registry.get(&resource)?;

        let response: Value = handler.update(id, payload, query).await?;

        Ok((
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(response),
        ))
    }

    pub async fn delete(
        State(state): State<App>,
        Path((resource, id)): Path<(String, i32)>,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;
        let handler = registry.get(&resource)?;

        handler.delete(id).await?;

        Ok(StatusCode::NO_CONTENT)
    }

    pub async fn index_for_resource(
        State(state): State<App>,
        resource: String,
        mut query: QueryParams,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;


        if let Some(cfg) = registry.entity_config(&resource) {
            let filterable: Vec<&str> =
                cfg.filterable_fields.iter().map(String::as_str).collect();
            let sortable: Vec<&str> = cfg.sortable_fields.iter().map(String::as_str).collect();
            query.validate_filters(&filterable)?;
            query.validate_sorts(&sortable)?;

            for filter in &mut query.filters {
                if filter.operator != appkit_core::jsonapi::FilterOperator::Eq {
                    continue;
                }
                let Some(strategy) = cfg.search_strategies.get(&filter.field) else {
                    continue;
                };
                let Some(raw) = filter.value.as_deref() else {
                    continue;
                };
                let (operator, value) = strategy.apply(raw);
                filter.operator = operator;
                filter.value = Some(value);
            }
        }

        let handler = registry.get(&resource)?;
        let response: Value = handler.index(query).await?;
        Ok((
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(response),
        ))
    }

    pub async fn create_for_resource(
        State(state): State<App>,
        resource: String,
        Json(payload): Json<Value>,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;
        let handler = registry.get(&resource)?;
        let response: Value = handler.create(payload).await?;
        Ok((
            StatusCode::CREATED,
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(response),
        ))
    }

    pub async fn show_for_resource(
        State(state): State<App>,
        resource: String,
        Path(id): Path<i32>,
        query: QueryParams,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;
        let handler = registry.get(&resource)?;
        let response: Value = handler.show(id, query).await?;
        Ok((
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(response),
        ))
    }

    pub async fn update_for_resource(
        State(state): State<App>,
        resource: String,
        Path(id): Path<i32>,
        query: QueryParams,
        Json(payload): Json<Value>,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;
        let handler = registry.get(&resource)?;
        let response: Value = handler.update(id, payload, query).await?;
        Ok((
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(response),
        ))
    }

    pub async fn delete_for_resource(
        State(state): State<App>,
        resource: String,
        Path(id): Path<i32>,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;
        let handler = registry.get(&resource)?;
        handler.delete(id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    pub async fn related_for_resource(
        State(state): State<App>,
        resource: String,
        Path(id): Path<i32>,
        relationship: String,
        query: QueryParams,
    ) -> Result<impl IntoResponse, AppError> {
        let registry = state
            .v1_registry
            .as_ref()
            .ok_or_else(|| AppError::NotFound("V1 API not configured".to_string()))?;

        let rel_meta = registry.get_relationship(&resource, &relationship)?;

        let handler = registry.get(&resource)?;

        let main_resource_response = handler.show(id, QueryParams::default()).await?;

        let foreign_key_value = if let Some(local_field) = &rel_meta.local_field {
            if let Some(data) = main_resource_response.get("data") {
                if let Some(attributes) = data.get("attributes") {
                    attributes
                        .get(local_field)
                        .and_then(|v| v.as_i64())
                        .map(|i| i as i32)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            if let Some(data) = main_resource_response.get("data") {
                if let Some(relationships) = data.get("relationships") {
                    if let Some(rel_data) = relationships.get(&relationship) {
                        if let Some(rel_obj) = rel_data.get("data") {
                            rel_obj.get("id").and_then(|v| {
                                v.as_str()
                                    .and_then(|s| s.parse::<i32>().ok())
                                    .or_else(|| v.as_i64().map(|i| i as i32))
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(fk_id) = foreign_key_value {
            let related_handler = registry.get(&rel_meta.target_type)?;

            let related_response = related_handler.show(fk_id, query).await?;
            Ok((
                [(header::CONTENT_TYPE, "application/vnd.api+json")],
                Json(related_response),
            ))
        } else {
            Ok((
                [(header::CONTENT_TYPE, "application/vnd.api+json")],
                Json(serde_json::json!({
                    "data": null,
                    "jsonapi": { "version": "1.0" }
                })),
            ))
        }
    }
}
