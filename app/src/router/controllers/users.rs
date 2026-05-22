use crate::app::App;
use crate::database::UserRepository;
use crate::middleware::CurrentUser;
use crate::presenter::{present_users, UserPresenter};
use appkit_core::error::{AppError, AppResult};
use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    #[serde(default = "default_page")]
    pub page: usize,

    #[serde(default = "default_per_page")]
    pub per_page: usize,

    pub search: Option<String>,
}

fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    20
}

pub async fn get_profile(CurrentUser(user): CurrentUser) -> Response {
    let user_data = UserPresenter::from(&user);
    (
        [(header::CONTENT_TYPE, "application/vnd.api+json")],
        Json(serde_json::json!({
            "data": {
                "id": user_data.id.to_string(),
                "type": "users",
                "attributes": {
                    "email": user_data.email,
                    "first_name": user_data.first_name,
                    "last_name": user_data.last_name,
                    "role": user_data.role,
                    "status": user_data.status,
                    "created_at": user_data.created_at,
                    "updated_at": user_data.updated_at,
                    "last_login_at": user_data.last_login_at,
                }
            }
        })),
    )
        .into_response()
}

pub async fn admin_list_users(
    State(state): State<App>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<ListUsersQuery>,
) -> AppResult<Response> {
    if !user.is_admin() {
        return Err(AppError::AuthorizationFailed(
            "Admin access required".to_string(),
        ));
    }

    let repo = UserRepository::new(state.db_pool.clone());

    let mut query_params = appkit_core::jsonapi::query::QueryParams::default();
    query_params.page.number = query.page as i64;
    query_params.page.size = query.per_page as i64;

    if let Some(search_term) = query.search {
        query_params
            .filters
            .push(appkit_core::jsonapi::query::FilterCondition {
                field: "email".to_string(),
                operator: appkit_core::jsonapi::FilterOperator::Like,
                value: Some(format!("%{}%", search_term)),
            });
    }

    let result = repo
        .find_with_params(&query_params)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let users_data: Vec<_> = result.items.iter().map(|u| u.to_security_user()).collect();
    let users_data = present_users(&users_data);

    Ok((
        [(header::CONTENT_TYPE, "application/vnd.api+json")],
        Json(serde_json::json!({
            "data": users_data,
            "pagination": {
                "page": result.page,
                "per_page": result.per_page,
                "total": result.total,
                "total_pages": result.last_page(),
            }
        })),
    )
        .into_response())
}

pub async fn list_users(
    State(state): State<App>,
    CurrentUser(_user): CurrentUser,
    query_params: appkit_core::jsonapi::query::QueryParams,
) -> AppResult<Response> {
    let repo = UserRepository::new(state.db_pool.clone());

    let result = repo
        .find_with_params(&query_params)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let users_data: Vec<_> = result
        .items
        .iter()
        .map(|u| {
            let user_data = UserPresenter::from(&u.to_security_user());
            serde_json::json!({
                "id": user_data.id.to_string(),
                "type": "users",
                "attributes": {
                    "email": user_data.email,
                    "first_name": user_data.first_name,
                    "last_name": user_data.last_name,
                    "role": user_data.role,
                    "status": user_data.status,
                    "created_at": user_data.created_at,
                    "updated_at": user_data.updated_at,
                    "last_login_at": user_data.last_login_at,
                }
            })
        })
        .collect();

    Ok((
        [(header::CONTENT_TYPE, "application/vnd.api+json")],
        Json(serde_json::json!({
            "data": users_data,
            "meta": {
                "total": result.total,
                "page": {
                    "number": result.page,
                    "size": result.per_page,
                }
            }
        })),
    )
        .into_response())
}

pub async fn get_user_by_id(
    State(state): State<App>,
    CurrentUser(_user): CurrentUser,
    Path(id): Path<i32>,
) -> AppResult<Response> {
    let repo = UserRepository::new(state.db_pool.clone());

    let user = repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User not found: {}", id)))?;

    let user_data = UserPresenter::from(&user.to_security_user());

    Ok((
        [(header::CONTENT_TYPE, "application/vnd.api+json")],
        Json(serde_json::json!({
            "data": {
                "id": user_data.id.to_string(),
                "type": "users",
                "attributes": {
                    "email": user_data.email,
                    "first_name": user_data.first_name,
                    "last_name": user_data.last_name,
                }
            }
        })),
    )
        .into_response())
}
