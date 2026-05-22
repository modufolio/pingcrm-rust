use crate::error::AppError;
use crate::jsonapi::{configurator::EntityConfig, QueryParams};
use async_trait::async_trait;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct JsonApiState {
    pub registry: Arc<HashMap<String, RegisteredEntity>>,
}

impl JsonApiState {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(HashMap::new()),
        }
    }

    pub fn from_registry(registry: HashMap<String, RegisteredEntity>) -> Self {
        Self {
            registry: Arc::new(registry),
        }
    }
}

impl Default for JsonApiState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RegisteredEntity {
    pub config: EntityConfig,

    pub handler: Arc<dyn ResourceHandler>,
}

#[async_trait]
pub trait ResourceHandler: Send + Sync + 'static {
    fn resource_type(&self) -> &'static str;

    fn allowed_fields(&self) -> &'static [&'static str];

    fn allowed_includes(&self) -> &'static [&'static str];

    fn is_field_allowed(&self, field: &str) -> bool {
        self.allowed_fields().contains(&field)
    }

    fn is_include_allowed(&self, include: &str) -> bool {
        self.allowed_includes().contains(&include)
    }

    async fn index(&self, params: QueryParams) -> Result<Value, AppError>;

    async fn show(&self, id: i32, params: QueryParams) -> Result<Value, AppError>;

    async fn create(&self, payload: Value) -> Result<Value, AppError>;

    async fn update(&self, id: i32, payload: Value, params: QueryParams)
        -> Result<Value, AppError>;

    async fn delete(&self, id: i32) -> Result<(), AppError>;

    async fn related(
        &self,
        id: i32,
        relationship: &str,
        params: QueryParams,
    ) -> Result<Value, AppError> {
        let _ = (id, relationship, params);
        Err(AppError::NotImplemented)
    }
}

pub struct JsonApiController;

impl JsonApiController {
    pub async fn index(
        Path(resource): Path<String>,
        Query(query_map): Query<HashMap<String, String>>,
        State(state): State<JsonApiState>,
        headers: HeaderMap,
    ) -> Result<Json<Value>, AppError> {
        Self::validate_accept_header(&headers)?;

        let params = QueryParams::from_query_map(&query_map);
        let entity = state
            .registry
            .get(&resource)
            .ok_or_else(|| AppError::resource_not_found(&resource))?;

        if !entity.config.operations.index {
            return Err(AppError::method_not_allowed());
        }

        Self::validate_query_params(&params, entity)?;
        let document = entity.handler.index(params).await?;

        Ok(Json(document))
    }

    pub async fn show(
        Path((resource, id_str)): Path<(String, String)>,
        Query(query_map): Query<HashMap<String, String>>,
        State(state): State<JsonApiState>,
        headers: HeaderMap,
    ) -> Result<Json<Value>, AppError> {
        Self::validate_accept_header(&headers)?;

        let params = QueryParams::from_query_map(&query_map);
        let entity = state
            .registry
            .get(&resource)
            .ok_or_else(|| AppError::resource_not_found(&resource))?;

        if !entity.config.operations.show {
            return Err(AppError::method_not_allowed());
        }

        let id: i32 = id_str.parse().map_err(|_| AppError::invalid_id(&id_str))?;

        Self::validate_query_params(&params, entity)?;
        let document = entity.handler.show(id, params).await?;

        Ok(Json(document))
    }

    pub async fn create(
        Path(resource): Path<String>,
        State(state): State<JsonApiState>,
        headers: HeaderMap,
        Json(payload): Json<Value>,
    ) -> Result<impl IntoResponse, AppError> {
        Self::require_jsonapi_content_type(&headers)?;

        let entity = state
            .registry
            .get(&resource)
            .ok_or_else(|| AppError::resource_not_found(&resource))?;

        if !entity.config.operations.create {
            return Err(AppError::method_not_allowed());
        }

        Self::validate_payload_type(&payload, entity.handler.resource_type())?;

        let document = entity.handler.create(payload).await?;

        Ok((StatusCode::CREATED, Json(document)))
    }

    pub async fn update(
        Path((resource, id_str)): Path<(String, String)>,
        Query(query_map): Query<HashMap<String, String>>,
        State(state): State<JsonApiState>,
        headers: HeaderMap,
        Json(payload): Json<Value>,
    ) -> Result<Json<Value>, AppError> {
        Self::require_jsonapi_content_type(&headers)?;

        let params = QueryParams::from_query_map(&query_map);
        let entity = state
            .registry
            .get(&resource)
            .ok_or_else(|| AppError::resource_not_found(&resource))?;

        if !entity.config.operations.update {
            return Err(AppError::method_not_allowed());
        }

        let id: i32 = id_str.parse().map_err(|_| AppError::invalid_id(&id_str))?;

        Self::validate_payload_type(&payload, entity.handler.resource_type())?;
        Self::validate_query_params(&params, entity)?;

        let document = entity.handler.update(id, payload, params).await?;

        Ok(Json(document))
    }

    pub async fn delete(
        Path((resource, id_str)): Path<(String, String)>,
        State(state): State<JsonApiState>,
    ) -> Result<impl IntoResponse, AppError> {
        let entity = state
            .registry
            .get(&resource)
            .ok_or_else(|| AppError::resource_not_found(&resource))?;

        if !entity.config.operations.delete {
            return Err(AppError::method_not_allowed());
        }

        let id: i32 = id_str.parse().map_err(|_| AppError::invalid_id(&id_str))?;

        entity.handler.delete(id).await?;

        Ok(StatusCode::NO_CONTENT)
    }

    pub async fn related(
        Path((resource, id_str, relationship)): Path<(String, String, String)>,
        Query(query_map): Query<HashMap<String, String>>,
        State(state): State<JsonApiState>,
    ) -> Result<Json<Value>, AppError> {
        let params = QueryParams::from_query_map(&query_map);
        let entity = state
            .registry
            .get(&resource)
            .ok_or_else(|| AppError::resource_not_found(&resource))?;

        if !entity
            .config
            .relationships
            .iter()
            .any(|r| r.name == relationship)
        {
            return Err(AppError::relationship_not_found(&relationship));
        }

        let id: i32 = id_str.parse().map_err(|_| AppError::invalid_id(&id_str))?;

        let document = entity.handler.related(id, &relationship, params).await?;

        Ok(Json(document))
    }

    fn validate_query_params(
        params: &QueryParams,
        entity: &RegisteredEntity,
    ) -> Result<(), AppError> {
        let handler = &entity.handler;

        for (typ, fields) in &params.fields {
            if typ == handler.resource_type() {
                for field in fields {
                    if !handler.is_field_allowed(field) {
                        return Err(AppError::invalid_field(field, typ));
                    }
                }
            }
        }

        for include in &params.includes {
            if !handler.is_include_allowed(include) {
                return Err(AppError::invalid_include(include));
            }
        }

        for (field, _) in &params.sort {
            if !handler.is_field_allowed(field) {
                return Err(AppError::invalid_sort_field(field));
            }
        }

        Ok(())
    }

    fn validate_payload_type(payload: &Value, expected_type: &str) -> Result<(), AppError> {
        if let Some(data) = payload.get("data") {
            if let Some(typ) = data.get("type").and_then(|t| t.as_str()) {
                if typ != expected_type {
                    return Err(AppError::type_mismatch(typ, expected_type));
                }
            }
        }
        Ok(())
    }

    fn require_jsonapi_content_type(headers: &HeaderMap) -> Result<(), AppError> {
        if let Some(content_type) = headers.get("content-type") {
            let value = content_type
                .to_str()
                .map_err(|_| AppError::invalid_content_type())?
                .trim();

            if value != "application/vnd.api+json" {
                return Err(AppError::invalid_content_type());
            }
        } else {
            return Err(AppError::missing_content_type());
        }
        Ok(())
    }

    fn validate_accept_header(headers: &HeaderMap) -> Result<(), AppError> {
        if let Some(accept) = headers.get("accept") {
            let accept_str = accept
                .to_str()
                .map_err(|_| AppError::BadRequest("Invalid Accept header".to_string()))?;

            for media_type in accept_str.split(',') {
                let media_type = media_type.trim();

                if media_type.starts_with("application/vnd.api+json") {
                    if media_type.contains(';') {
                        return Err(AppError::NotAcceptable(
                            "JSON:API media type in Accept header must not have parameters"
                                .to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = JsonApiState::new();
        assert_eq!(state.registry.len(), 0);
    }

    #[test]
    fn test_state_default() {
        let state = JsonApiState::default();
        assert_eq!(state.registry.len(), 0);
    }
}
