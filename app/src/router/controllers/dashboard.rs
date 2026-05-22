use axum::{body::Body, extract::State, http::Request, response::Response};
use serde_json::json;
use tower_sessions::Session;

use crate::app::App;
use crate::database::{AccountRepository, UserRepository};
use crate::inertia::DefaultProps;
use appkit_core::security::user::User;
use appkit_core::{error::AppResult, response::AppResponse};

pub async fn index(
    State(state): State<App>,
    session: Session,
    request: Request<Body>,
) -> AppResult<Response> {
    let user_repo = UserRepository::new(state.db_pool.clone());
    let account_repo = AccountRepository::new(state.db_pool.clone());

    let total_users = user_repo.count().await.unwrap_or(0);
    let total_organizations = account_repo.count().await.unwrap_or(0);
    let total_contacts = 0i64;

    let stats = json!({
        "totalUsers": total_users,
        "totalOrganizations": total_organizations,
        "totalContacts": total_contacts,
        "userGrowth": 0.0,
        "organizationGrowth": 0.0,
        "contactGrowth": 0.0,
    });

    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state, json!({ "stats": stats })).await;

    let response = AppResponse::inertia("Dashboard/Index")
        .with_props(props)
        .render(&request);

    Ok(response)
}
