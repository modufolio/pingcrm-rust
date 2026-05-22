use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::Request,
    response::Response,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use tower_sessions::Session;

use crate::database::{NewUser, UserRepository, UserUpdate};
use crate::inertia::DefaultProps;
use crate::{presenter::UserListPresenter, App};
use appkit_core::security::user::User;
use appkit_core::{
    error::{AppError, AppResult},
    response::AppResponse,
};

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub owner: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub owner: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    pub search: Option<String>,
    pub role: Option<String>,
    pub trashed: Option<String>,
    #[serde(rename = "sort")]
    pub sort_column: Option<String>,
    #[serde(rename = "direction")]
    pub sort_direction: Option<String>,
}

pub async fn index(
    State(state): State<App>,
    session: Session,
    Query(query): Query<UserListQuery>,
    request: Request<Body>,
) -> AppResult<Response> {
    let repo = UserRepository::new(state.db_pool.clone());

    let mut query_params = appkit_core::jsonapi::query::QueryParams::default();
    query_params.page.size = 50;

    if let Some(ref search_term) = query.search {
        if !search_term.is_empty() {
            query_params
                .filters
                .push(appkit_core::jsonapi::query::FilterCondition {
                    field: "email".to_string(),
                    operator: appkit_core::jsonapi::FilterOperator::Like,
                    value: Some(format!("%{}%", search_term)),
                });
        }
    }

    let result = repo
        .find_with_params(&query_params)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let users_data: Vec<UserListPresenter> = result
        .items
        .iter()
        .map(|u| UserListPresenter::from(&u.to_security_user()))
        .collect();

    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state,
        json!({
            "filters": {
                "search": query.search,
                "role": query.role,
                "trashed": query.trashed,
                "sortColumn": query.sort_column,
                "sortDirection": query.sort_direction.unwrap_or_else(|| "asc".to_string()),
            },
            "users": users_data,
        }),
    )
    .await;

    let response = AppResponse::inertia("Users/Index")
        .with_props(props)
        .render(&request);

    Ok(response)
}

pub async fn create(
    State(state): State<App>,
    session: Session,
    request: Request<Body>,
) -> AppResult<Response> {
    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state, json!({})).await;

    let response = AppResponse::inertia("Users/Create")
        .with_props(props)
        .render(&request);

    Ok(response)
}

pub async fn edit(
    State(state): State<App>,
    session: Session,
    Path(id): Path<i32>,
    request: Request<Body>,
) -> AppResult<Response> {
    let repo = UserRepository::new(state.db_pool.clone());

    let user_record = repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    let user_entity = user_record.to_security_user();
    let user_presenter = crate::presenter::UserPresenter::from(&user_entity);

    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state,
        json!({ "user": user_presenter }),
    )
    .await;

    let response = AppResponse::inertia("Users/Edit")
        .with_props(props)
        .render(&request);

    Ok(response)
}

pub async fn store(
    State(state): State<App>,
    Json(request): Json<CreateUserRequest>,
) -> AppResult<Response> {
    let repo = UserRepository::new(state.db_pool.clone());

    let password_hash = appkit_core::security::hash_password(&request.password)
        .map_err(|e| AppError::InternalServerError(format!("Password hashing error: {}", e)))?;

    let new_user = NewUser::new(
        request.email,
        password_hash,
        request.first_name,
        request.last_name,
    )
    .with_roles(vec!["ROLE_USER".to_string()]);

    repo.create(new_user)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect("/users"))
}

pub async fn update(
    State(state): State<App>,
    Path(id): Path<i32>,
    Json(request): Json<UpdateUserRequest>,
) -> AppResult<Response> {
    let repo = UserRepository::new(state.db_pool.clone());

    let _existing = repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    let mut user_update = UserUpdate::new();

    if let Some(first_name) = request.first_name {
        user_update = user_update.first_name(first_name);
    }
    if let Some(last_name) = request.last_name {
        user_update = user_update.last_name(last_name);
    }

    if let Some(email) = request.email {
        user_update = user_update.email(email);
    }

    if let Some(password) = request.password {
        if !password.is_empty() {
            let password_hash = appkit_core::security::hash_password(&password).map_err(|e| {
                AppError::InternalServerError(format!("Password hashing error: {}", e))
            })?;
            user_update = user_update.password(password_hash);
        }
    }

    repo.update(id, user_update)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect(&format!("/users/{}/edit", id)))
}

pub async fn destroy(State(state): State<App>, Path(id): Path<i32>) -> AppResult<Response> {
    let repo = UserRepository::new(state.db_pool.clone());

    let _existing = repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    let user_update = UserUpdate::new().enabled(false);

    repo.update(id, user_update)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect(&format!("/users/{}/edit", id)))
}

pub async fn restore(State(state): State<App>, Path(id): Path<i32>) -> AppResult<Response> {
    let repo = UserRepository::new(state.db_pool.clone());

    let _existing = repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    let user_update = UserUpdate::new().enabled(true);

    repo.update(id, user_update)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect(&format!("/users/{}/edit", id)))
}
