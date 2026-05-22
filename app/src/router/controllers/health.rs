use crate::app::App;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

pub async fn health_check(State(state): State<App>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now(),
        "maintenance_mode": state.config.maintenance_mode,
    }))
}

pub async fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "Not Found",
            "message": "The requested resource was not found"
        })),
    )
        .into_response()
}
