use crate::error::AppError;
use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    pub status: u16,
    pub title: String,
    pub detail: Option<String>,

    pub errors: Option<HashMap<String, Vec<String>>>,
}

impl ErrorData {
    pub fn new(status: u16, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status,
            title: title.into(),
            detail: Some(detail.into()),
            errors: None,
        }
    }

    pub fn with_errors(mut self, errors: HashMap<String, Vec<String>>) -> Self {
        self.errors = Some(errors);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorFormat {
    JsonApi,
    Json,
    PlainText,
}

impl ErrorFormat {
    pub fn from_accept_header(accept: Option<&str>) -> Self {
        let accept = match accept {
            Some(h) => h,
            None => return Self::JsonApi,
        };

        if accept.contains("application/vnd.api+json") {
            return Self::JsonApi;
        }

        if accept.contains("application/json") {
            return Self::Json;
        }

        if accept.contains("text/plain") {
            return Self::PlainText;
        }

        if accept.contains("*/*") || accept.contains("application/*") {
            return Self::JsonApi;
        }

        Self::JsonApi
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::JsonApi => "application/vnd.api+json",
            Self::Json => "application/json",
            Self::PlainText => "text/plain; charset=utf-8",
        }
    }
}

pub struct ErrorFormatter;

impl ErrorFormatter {
    pub fn json_api(data: ErrorData) -> Response {
        let status = StatusCode::from_u16(data.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let mut error = json!({
            "status": data.status.to_string(),
            "title": data.title,
        });

        if let Some(detail) = data.detail {
            error["detail"] = json!(detail);
        }

        let errors = if let Some(field_errors) = data.errors {
            field_errors
                .into_iter()
                .flat_map(|(field, messages)| {
                    messages.into_iter().map(move |message| {
                        json!({
                            "status": "422",
                            "title": "Validation Error",
                            "detail": message,
                            "source": {
                                "pointer": format!("/data/attributes/{}", field)
                            }
                        })
                    })
                })
                .collect()
        } else {
            vec![error]
        };

        let body = json!({
            "jsonapi": { "version": "1.0" },
            "errors": errors,
        });

        (
            status,
            [(header::CONTENT_TYPE, ErrorFormat::JsonApi.mime_type())],
            Json(body),
        )
            .into_response()
    }

    pub fn json(data: ErrorData) -> Response {
        let status = StatusCode::from_u16(data.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let mut body = json!({
            "status": data.status,
            "title": data.title,
        });

        if let Some(detail) = data.detail {
            body["detail"] = json!(detail);
        }

        if let Some(errors) = data.errors {
            body["errors"] = json!(errors);
        }

        (
            status,
            [(header::CONTENT_TYPE, ErrorFormat::Json.mime_type())],
            Json(body),
        )
            .into_response()
    }

    pub fn plain_text(data: ErrorData) -> Response {
        let status = StatusCode::from_u16(data.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let text = if let Some(detail) = data.detail {
            format!("{}: {}", data.title, detail)
        } else {
            data.title
        };

        (
            status,
            [(header::CONTENT_TYPE, ErrorFormat::PlainText.mime_type())],
            text,
        )
            .into_response()
    }

    pub fn format(data: ErrorData, format: ErrorFormat) -> Response {
        match format {
            ErrorFormat::JsonApi => Self::json_api(data),
            ErrorFormat::Json => Self::json(data),
            ErrorFormat::PlainText => Self::plain_text(data),
        }
    }
}

pub struct ExceptionHandler;

impl ExceptionHandler {
    pub fn error_to_data(error: &AppError) -> ErrorData {
        match error {
            AppError::AuthenticationFailed(msg) => {
                ErrorData::new(401, "Authentication Failed", msg)
            }
            AppError::AuthorizationFailed(msg) => ErrorData::new(403, "Authorization Failed", msg),
            AppError::ValidationFailed(msg) => ErrorData::new(400, "Validation Failed", msg),
            AppError::ValidationError { message, errors } => {
                ErrorData::new(422, "Validation Failed", message).with_errors(errors.clone())
            }
            AppError::NotFound(msg) => ErrorData::new(404, "Not Found", msg),
            AppError::BadRequest(msg) => ErrorData::new(400, "Bad Request", msg),
            AppError::Conflict(msg) => ErrorData::new(409, "Conflict", msg),
            AppError::RateLimitExceeded => ErrorData::new(
                429,
                "Rate Limit Exceeded",
                "Too many requests. Please try again later.",
            ),
            AppError::MaintenanceMode => ErrorData::new(
                503,
                "Service Unavailable",
                "Service is currently under maintenance.",
            ),
            AppError::PayloadTooLarge(msg) => ErrorData::new(413, "Payload Too Large", msg),
            AppError::UnprocessableEntity(msg) => ErrorData::new(422, "Unprocessable Entity", msg),
            AppError::MethodNotAllowed(msg) => ErrorData::new(405, "Method Not Allowed", msg),
            AppError::UnsupportedMediaType(msg) => {
                ErrorData::new(415, "Unsupported Media Type", msg)
            }
            AppError::ServiceUnavailable(msg) => ErrorData::new(503, "Service Unavailable", msg),
            AppError::GatewayTimeout(msg) => ErrorData::new(504, "Gateway Timeout", msg),
            AppError::DatabaseError(_) => {
                ErrorData::new(500, "Internal Server Error", "A database error occurred.")
            }
            AppError::DieselError(_) => {
                ErrorData::new(500, "Internal Server Error", "A database error occurred.")
            }
            AppError::InternalServerError(_) | AppError::InternalError(_) => {
                ErrorData::new(500, "Internal Server Error", "An internal error occurred.")
            }
            AppError::NotImplemented => {
                ErrorData::new(501, "Not Implemented", "This operation is not implemented.")
            }
            AppError::NotAcceptable(msg) => ErrorData::new(406, "Not Acceptable", msg),
        }
    }

    pub fn handle(error: &AppError, accept_header: Option<&str>) -> Response {
        let data = Self::error_to_data(error);
        let format = ErrorFormat::from_accept_header(accept_header);
        ErrorFormatter::format(data, format)
    }

    pub fn handle_with_format(error: &AppError, format: ErrorFormat) -> Response {
        let data = Self::error_to_data(error);
        ErrorFormatter::format(data, format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_accept_header() {
        assert_eq!(
            ErrorFormat::from_accept_header(Some("application/vnd.api+json")),
            ErrorFormat::JsonApi
        );
        assert_eq!(
            ErrorFormat::from_accept_header(Some("application/json")),
            ErrorFormat::Json
        );
        assert_eq!(
            ErrorFormat::from_accept_header(Some("text/plain")),
            ErrorFormat::PlainText
        );
        assert_eq!(ErrorFormat::from_accept_header(None), ErrorFormat::JsonApi);
        assert_eq!(
            ErrorFormat::from_accept_header(Some("*/*")),
            ErrorFormat::JsonApi
        );
    }

    #[test]
    fn test_error_to_data() {
        let error = AppError::NotFound("User not found".to_string());
        let data = ExceptionHandler::error_to_data(&error);
        assert_eq!(data.status, 404);
        assert_eq!(data.title, "Not Found");
        assert_eq!(data.detail, Some("User not found".to_string()));
    }

    #[test]
    fn test_validation_error_with_fields() {
        let mut errors = HashMap::new();
        errors.insert("email".to_string(), vec!["Invalid email".to_string()]);
        errors.insert("password".to_string(), vec!["Too short".to_string()]);

        let error = AppError::ValidationError {
            message: "Validation failed".to_string(),
            errors: errors.clone(),
        };

        let data = ExceptionHandler::error_to_data(&error);
        assert_eq!(data.status, 422);
        assert_eq!(data.title, "Validation Failed");
        assert!(data.errors.is_some());
        assert_eq!(data.errors.unwrap(), errors);
    }
}
