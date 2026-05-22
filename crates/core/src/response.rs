use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;

use crate::inertia::InertiaResponse;
use crate::jsonapi::{ErrorObject, JsonApiDocument, JsonApiSerializer, ResourceObject};

pub struct AppResponse;

impl AppResponse {
    pub fn json<T: Serialize>(data: T) -> Response {
        Json(data).into_response()
    }

    pub fn json_pretty<T: Serialize>(data: T) -> Response {
        match serde_json::to_string_pretty(&data) {
            Ok(json_string) => {
                ([(header::CONTENT_TYPE, "application/json")], json_string).into_response()
            }
            Err(_) => Self::error(
                "Failed to serialize response",
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    }

    pub fn json_with_status<T: Serialize>(data: T, status: StatusCode) -> Response {
        (status, Json(data)).into_response()
    }

    pub fn success<T: Serialize>(data: T) -> Response {
        Json(json!({
            "success": true,
            "data": data
        }))
        .into_response()
    }

    pub fn success_message(message: impl Into<String>) -> Response {
        Json(json!({
            "success": true,
            "message": message.into()
        }))
        .into_response()
    }

    pub fn error(message: impl Into<String>, status: StatusCode) -> Response {
        (
            status,
            Json(json!({
                "success": false,
                "error": message.into()
            })),
        )
            .into_response()
    }

    pub fn not_found(message: impl Into<String>) -> Response {
        Self::error(message, StatusCode::NOT_FOUND)
    }

    pub fn unauthorized(message: impl Into<String>) -> Response {
        Self::error(message, StatusCode::UNAUTHORIZED)
    }

    pub fn forbidden(message: impl Into<String>) -> Response {
        Self::error(message, StatusCode::FORBIDDEN)
    }

    pub fn bad_request(message: impl Into<String>) -> Response {
        Self::error(message, StatusCode::BAD_REQUEST)
    }

    pub fn too_many_requests(message: impl Into<String>) -> Response {
        Self::error(message, StatusCode::TOO_MANY_REQUESTS)
    }

    pub fn unavailable(message: impl Into<String>) -> Response {
        Self::error(message, StatusCode::SERVICE_UNAVAILABLE)
    }

    pub fn redirect(uri: &str) -> Response {
        Redirect::to(uri).into_response()
    }

    pub fn redirect_with_status(uri: &str, status: u16) -> Response {
        let status_code = match status {
            301 => StatusCode::MOVED_PERMANENTLY,
            302 => StatusCode::FOUND,
            _ => StatusCode::FOUND,
        };

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset="UTF-8" />
        <meta http-equiv="refresh" content="0;url='{}'" />
        <title>Redirecting to {}</title>
    </head>
    <body>
        Redirecting to <a href="{}">{}</a>.
    </body>
</html>"#,
            uri, uri, uri, uri
        );

        (
            status_code,
            [
                (header::LOCATION, uri),
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            ],
            html_body,
        )
            .into_response()
    }

    pub fn redirect_temporary(uri: &str) -> Response {
        Redirect::temporary(uri).into_response()
    }

    pub fn redirect_permanent(uri: &str) -> Response {
        Redirect::permanent(uri).into_response()
    }

    pub fn html(content: impl Into<String>) -> Response {
        Self::html_with_status(content, StatusCode::OK)
    }

    pub fn html_with_status(content: impl Into<String>, status: StatusCode) -> Response {
        (
            status,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            content.into(),
        )
            .into_response()
    }

    pub fn empty() -> Response {
        StatusCode::NO_CONTENT.into_response()
    }

    pub fn no_content() -> Response {
        StatusCode::NO_CONTENT.into_response()
    }

    pub fn created<T: Serialize>(data: T, location: Option<String>) -> Response {
        let mut response = (StatusCode::CREATED, Json(data)).into_response();

        if let Some(loc) = location {
            response
                .headers_mut()
                .insert(header::LOCATION, loc.parse().unwrap());
        }

        response
    }

    pub fn jsonapi_resource(
        resource: ResourceObject,
        included: Option<Vec<ResourceObject>>,
    ) -> Response {
        let doc = JsonApiSerializer::serialize_resource(resource, included);
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(doc),
        )
            .into_response()
    }

    pub fn jsonapi_collection(
        resources: Vec<ResourceObject>,
        total: i64,
        current_page: i64,
        per_page: i64,
        base_url: &str,
        included: Option<Vec<ResourceObject>>,
    ) -> Response {
        let doc = JsonApiSerializer::serialize_collection(
            resources,
            total,
            current_page,
            per_page,
            base_url,
            included,
        );
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(doc),
        )
            .into_response()
    }

    pub fn jsonapi_error(error: ErrorObject) -> Response {
        let status = error
            .status
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(500);

        let status_code = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let doc = JsonApiSerializer::serialize_error(error);

        (
            status_code,
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(doc),
        )
            .into_response()
    }

    pub fn jsonapi_validation_errors(
        errors: &std::collections::HashMap<String, Vec<String>>,
    ) -> Response {
        let doc = JsonApiSerializer::serialize_validation_errors(errors);
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(doc),
        )
            .into_response()
    }

    pub fn jsonapi_app_error(error: &crate::error::AppError) -> Response {
        let error_data = crate::error_handler::ExceptionHandler::error_to_data(error);
        let error_object = ErrorObject::from_status(
            error_data.status,
            error_data.title,
            error_data.detail.unwrap_or_default(),
        );

        Self::jsonapi_error(error_object)
    }

    pub fn jsonapi_document(doc: JsonApiDocument, status: StatusCode) -> Response {
        (
            status,
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(doc),
        )
            .into_response()
    }

    pub fn jsonapi_not_found(detail: impl Into<String>) -> Response {
        let error = ErrorObject::from_status(404, "Not Found", detail);
        Self::jsonapi_error(error)
    }

    pub fn jsonapi_created(resource: ResourceObject, location: &str) -> Response {
        let doc = JsonApiSerializer::serialize_resource(resource, None);
        let mut response = (
            StatusCode::CREATED,
            [(header::CONTENT_TYPE, "application/vnd.api+json")],
            Json(doc),
        )
            .into_response();

        response
            .headers_mut()
            .insert(header::LOCATION, location.parse().unwrap());

        response
    }

    pub fn jsonapi_no_content() -> Response {
        StatusCode::NO_CONTENT.into_response()
    }

    pub fn jsonapi_ok(doc: JsonApiDocument) -> Response {
        Self::jsonapi_document(doc, StatusCode::OK)
    }

    pub fn inertia(component: impl Into<String>) -> InertiaResponse {
        InertiaResponse::new(component)
    }
}
