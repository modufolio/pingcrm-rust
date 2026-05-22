use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Validation failed: {message}")]
    ValidationError {
        message: String,
        errors: HashMap<String, Vec<String>>,
    },

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Diesel error: {0}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Internal error")]
    InternalError(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Maintenance mode")]
    MaintenanceMode,

    #[error("Payload too large: {0}")]
    PayloadTooLarge(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),

    #[error("Unsupported media type: {0}")]
    UnsupportedMediaType(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Gateway timeout: {0}")]
    GatewayTimeout(String),

    #[error("Not implemented")]
    NotImplemented,

    #[error("Not acceptable: {0}")]
    NotAcceptable(String),
}

impl From<Vec<crate::jsonapi::ErrorObject>> for AppError {
    fn from(errors: Vec<crate::jsonapi::ErrorObject>) -> Self {
        let message = errors
            .first()
            .and_then(|e| e.detail.clone())
            .or_else(|| errors.first().and_then(|e| e.title.clone()))
            .unwrap_or_else(|| "Validation failed".to_string());

        let mut field_errors: HashMap<String, Vec<String>> = HashMap::new();
        for error in errors {
            if let Some(source) = error.source {
                if let Some(pointer) = source.pointer {
                    let field = pointer.split('/').last().unwrap_or("unknown").to_string();

                    let error_message = error
                        .detail
                        .or(error.title)
                        .unwrap_or_else(|| "Invalid value".to_string());

                    field_errors
                        .entry(field)
                        .or_insert_with(Vec::new)
                        .push(error_message);
                }
            }
        }

        if field_errors.is_empty() {
            AppError::ValidationFailed(message)
        } else {
            AppError::ValidationError {
                message,
                errors: field_errors,
            }
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum RouteError {
    #[error("Route '{0}' not found")]
    RouteNotFound(String),

    #[error("Parameter '{param}' not found in route '{route}'")]
    ParameterNotFound { param: String, route: String },

    #[error("Missing parameters for route '{route}': {pattern}")]
    MissingParameters { route: String, pattern: String },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            AppError::DatabaseError(msg) => tracing::error!("Database error: {}", msg),
            AppError::DieselError(e) => tracing::error!("Diesel error: {}", e),
            AppError::InternalServerError(msg) => tracing::error!("Internal server error: {}", msg),
            AppError::InternalError(msg) => tracing::error!("Internal error: {}", msg),
            _ => {}
        }

        crate::error_handler::ExceptionHandler::handle_with_format(
            &self,
            crate::error_handler::ErrorFormat::JsonApi,
        )
    }
}

impl AppError {
    pub fn into_response_with_negotiation(self, accept_header: Option<&str>) -> Response {
        crate::error_handler::ExceptionHandler::handle(&self, accept_header)
    }

    pub fn with_context(self, request_type: crate::inertia::RequestType) -> Response {
        use crate::inertia::RequestType;

        match request_type {
            RequestType::JsonApi => crate::error_handler::ExceptionHandler::handle_with_format(
                &self,
                crate::error_handler::ErrorFormat::JsonApi,
            ),

            RequestType::Inertia => self.format_for_inertia(),

            RequestType::Web => crate::error_handler::ExceptionHandler::handle_with_format(
                &self,
                crate::error_handler::ErrorFormat::JsonApi,
            ),
        }
    }

    fn format_for_inertia(&self) -> Response {
        let (status, title, detail) = self.to_error_data();

        if let AppError::ValidationError { message: _, errors } = self {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                axum::Json(json!({
                    "message": title,
                    "errors": errors
                })),
            )
                .into_response();
        }

        (
            status,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            axum::Json(json!({
                "message": title,
                "detail": detail,
            })),
        )
            .into_response()
    }

    pub(crate) fn to_error_data(&self) -> (StatusCode, String, String) {
        match self {
            AppError::AuthenticationFailed(msg) => (
                StatusCode::UNAUTHORIZED,
                "Authentication Failed".to_string(),
                msg.clone(),
            ),
            AppError::AuthorizationFailed(msg) => (
                StatusCode::FORBIDDEN,
                "Authorization Failed".to_string(),
                msg.clone(),
            ),
            AppError::ValidationFailed(msg) => (
                StatusCode::BAD_REQUEST,
                "Validation Failed".to_string(),
                msg.clone(),
            ),
            AppError::ValidationError { message, .. } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation Failed".to_string(),
                message.clone(),
            ),
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "Not Found".to_string(), msg.clone())
            }
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "Bad Request".to_string(),
                msg.clone(),
            ),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "Conflict".to_string(), msg.clone()),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate Limit Exceeded".to_string(),
                "Too many requests. Please try again later.".to_string(),
            ),
            AppError::MaintenanceMode => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable".to_string(),
                "Service is currently under maintenance.".to_string(),
            ),
            AppError::PayloadTooLarge(msg) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "Payload Too Large".to_string(),
                msg.clone(),
            ),
            AppError::UnprocessableEntity(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Unprocessable Entity".to_string(),
                msg.clone(),
            ),
            AppError::MethodNotAllowed(msg) => (
                StatusCode::METHOD_NOT_ALLOWED,
                "Method Not Allowed".to_string(),
                msg.clone(),
            ),
            AppError::UnsupportedMediaType(msg) => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Unsupported Media Type".to_string(),
                msg.clone(),
            ),
            AppError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable".to_string(),
                msg.clone(),
            ),
            AppError::GatewayTimeout(msg) => (
                StatusCode::GATEWAY_TIMEOUT,
                "Gateway Timeout".to_string(),
                msg.clone(),
            ),
            AppError::DatabaseError(_) | AppError::DieselError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
                "A database error occurred.".to_string(),
            ),
            AppError::InternalServerError(_) | AppError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
                "An internal error occurred.".to_string(),
            ),
            AppError::NotImplemented => (
                StatusCode::NOT_IMPLEMENTED,
                "Not Implemented".to_string(),
                "This operation is not implemented.".to_string(),
            ),
            AppError::NotAcceptable(msg) => (
                StatusCode::NOT_ACCEPTABLE,
                "Not Acceptable".to_string(),
                msg.clone(),
            ),
        }
    }

    pub fn resource_not_found(resource: &str) -> Self {
        AppError::NotFound(format!("Resource '{}' not found", resource))
    }

    pub fn method_not_allowed() -> Self {
        AppError::MethodNotAllowed("This operation is not allowed for this resource".to_string())
    }

    pub fn invalid_id(id: &str) -> Self {
        AppError::BadRequest(format!("Invalid ID: '{}'", id))
    }

    pub fn relationship_not_found(relationship: &str) -> Self {
        AppError::NotFound(format!("Relationship '{}' not found", relationship))
    }

    pub fn invalid_field(field: &str, resource_type: &str) -> Self {
        AppError::BadRequest(format!(
            "Invalid field '{}' for resource type '{}'",
            field, resource_type
        ))
    }

    pub fn invalid_include(include: &str) -> Self {
        AppError::BadRequest(format!("Invalid include: '{}'", include))
    }

    pub fn invalid_sort_field(field: &str) -> Self {
        AppError::BadRequest(format!("Invalid sort field: '{}'", field))
    }

    pub fn invalid_filter_field(field: &str) -> Self {
        AppError::BadRequest(format!("Invalid filter field: '{}'", field))
    }

    pub fn type_mismatch(actual: &str, expected: &str) -> Self {
        AppError::BadRequest(format!(
            "Type mismatch: expected '{}', got '{}'",
            expected, actual
        ))
    }

    pub fn invalid_content_type() -> Self {
        AppError::UnsupportedMediaType(
            "Content-Type must be 'application/vnd.api+json'".to_string(),
        )
    }

    pub fn missing_content_type() -> Self {
        AppError::BadRequest("Content-Type header is required".to_string())
    }

    pub fn database_error<E: std::fmt::Display>(error: E) -> Self {
        tracing::error!("Database error: {}", error);

        AppError::InternalServerError("An internal error occurred".to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
