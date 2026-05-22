use axum::{body::Body, extract::State, http::Request, response::Response};
use serde_json::json;
use tower_sessions::Session;

use crate::app::App;
use crate::inertia::DefaultProps;
use appkit_core::security::user::User;
use appkit_core::{error::AppResult, response::AppResponse};

pub async fn orders_index(
    State(state): State<App>,
    session: Session,
    request: Request<Body>,
) -> AppResult<Response> {
    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state,
        json!({
            "filters": { "search": null, "trashed": null },
            "orders": {
                "data": [],
                "meta": { "total": 0, "per_page": 15, "current_page": 1, "last_page": 1 }
            },
        }),
    )
    .await;

    Ok(AppResponse::inertia("Orders/Index")
        .with_props(props)
        .render(&request))
}

pub async fn reports_index(
    State(state): State<App>,
    session: Session,
    request: Request<Body>,
) -> AppResult<Response> {
    let props =
        DefaultProps::merge(request.extensions().get::<User>(), &session, &state, json!({ "reports": [] })).await;

    Ok(AppResponse::inertia("Reports/Index")
        .with_props(props)
        .render(&request))
}

pub async fn upload_index(
    State(state): State<App>,
    session: Session,
    request: Request<Body>,
) -> AppResult<Response> {
    let props =
        DefaultProps::merge(request.extensions().get::<User>(), &session, &state, json!({ "uploads": [] })).await;

    Ok(AppResponse::inertia("Upload/Index")
        .with_props(props)
        .render(&request))
}
