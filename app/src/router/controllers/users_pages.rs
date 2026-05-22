use axum::{
    body::{Body, Bytes},
    extract::{FromRequest, Path, Query, State},
    http::{header, Request},
    response::Response,
};
use axum_extra::extract::Multipart;
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};
use tower_sessions::Session;

use crate::database::{NewUser, UserRepository, UserUpdate};
use crate::inertia::DefaultProps;
use crate::{presenter::UserListPresenter, App};
use appkit_core::security::user::User;
use appkit_core::{
    error::{AppError, AppResult},
    response::AppResponse,
};

#[derive(Default)]
struct UserFormFields {
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    password: Option<String>,
    owner: Option<bool>,
    photo: Option<(String, Bytes)>,
}

async fn extract_user_form(
    state: &App,
    request: Request<Body>,
) -> AppResult<UserFormFields> {
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if content_type.starts_with("multipart/form-data") {
        let multipart = Multipart::from_request(request, state)
            .await
            .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?;
        parse_multipart_form(multipart).await
    } else {
        #[derive(Deserialize)]
        struct JsonForm {
            #[serde(default)]
            first_name: Option<String>,
            #[serde(default)]
            last_name: Option<String>,
            #[serde(default)]
            email: Option<String>,
            #[serde(default)]
            password: Option<String>,
            #[serde(default)]
            owner: Option<bool>,
        }

        let bytes = axum::body::to_bytes(request.into_body(), usize::MAX)
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read body: {}", e)))?;
        let parsed: JsonForm = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::BadRequest(format!("Invalid JSON body: {}", e)))?;

        Ok(UserFormFields {
            first_name: parsed.first_name,
            last_name: parsed.last_name,
            email: parsed.email,
            password: parsed.password,
            owner: parsed.owner,
            photo: None,
        })
    }
}

async fn parse_multipart_form(mut multipart: Multipart) -> AppResult<UserFormFields> {
    let mut fields = UserFormFields::default();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "first_name" => fields.first_name = field.text().await.ok(),
            "last_name" => fields.last_name = field.text().await.ok(),
            "email" => fields.email = field.text().await.ok(),
            "password" => fields.password = field.text().await.ok(),
            "owner" => {
                let value = field.text().await.unwrap_or_default();
                fields.owner = Some(matches!(
                    value.as_str(),
                    "true" | "1" | "on" | "yes"
                ));
            }
            "photo" => {
                let filename = field.file_name().map(|s| s.to_string()).unwrap_or_default();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read photo: {}", e)))?;
                if !bytes.is_empty() {
                    fields.photo = Some((filename, bytes));
                }
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    Ok(fields)
}

async fn save_photo(filename: &str, bytes: &Bytes) -> AppResult<String> {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let unique = format!("{}.{}", uuid::Uuid::new_v4(), ext);
    let dir = PathBuf::from("public/uploads/users");
    fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create upload dir: {}", e)))?;
    let path = dir.join(&unique);
    let mut file = fs::File::create(&path)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to write photo: {}", e)))?;
    file.write_all(bytes)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to write photo: {}", e)))?;
    Ok(unique)
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

    let photo_url = user_record
        .photo_filename
        .as_ref()
        .map(|f| format!("/uploads/users/{}", f));

    let user_json = json!({
        "id": user_presenter.id,
        "first_name": user_presenter.first_name,
        "last_name": user_presenter.last_name,
        "email": user_presenter.email,
        "owner": user_record.owner,
        "photo": photo_url,
        "deleted_at": user_record.deleted_at,
    });

    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state,
        json!({ "user": user_json }),
    )
    .await;

    let response = AppResponse::inertia("Users/Edit")
        .with_props(props)
        .render(&request);

    Ok(response)
}

pub async fn store(State(state): State<App>, request: Request<Body>) -> AppResult<Response> {
    let repo = UserRepository::new(state.db_pool.clone());

    let fields = extract_user_form(&state, request).await?;

    let email = fields
        .email
        .ok_or_else(|| AppError::BadRequest("Email is required".to_string()))?;
    let password = fields
        .password
        .ok_or_else(|| AppError::BadRequest("Password is required".to_string()))?;
    let first_name = fields.first_name.unwrap_or_default();
    let last_name = fields.last_name.unwrap_or_default();

    let password_hash = appkit_core::security::hash_password(&password)
        .map_err(|e| AppError::InternalServerError(format!("Password hashing error: {}", e)))?;

    let mut new_user = NewUser::new(email, password_hash, first_name, last_name)
        .with_roles(vec!["ROLE_USER".to_string()]);
    new_user.owner = fields.owner.unwrap_or(false);

    if let Some((filename, bytes)) = fields.photo.as_ref() {
        let stored = save_photo(filename, bytes).await?;
        new_user.photo_filename = Some(stored);
    }

    repo.create(new_user)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect("/users"))
}

pub async fn update(
    State(state): State<App>,
    Path(id): Path<i32>,
    request: Request<Body>,
) -> AppResult<Response> {
    let repo = UserRepository::new(state.db_pool.clone());

    let _existing = repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    let fields = extract_user_form(&state, request).await?;

    let mut user_update = UserUpdate::new();

    if let Some(first_name) = fields.first_name {
        user_update = user_update.first_name(first_name);
    }
    if let Some(last_name) = fields.last_name {
        user_update = user_update.last_name(last_name);
    }
    if let Some(email) = fields.email {
        user_update = user_update.email(email);
    }
    if let Some(password) = fields.password {
        if !password.is_empty() {
            let password_hash = appkit_core::security::hash_password(&password).map_err(|e| {
                AppError::InternalServerError(format!("Password hashing error: {}", e))
            })?;
            user_update = user_update.password(password_hash);
        }
    }
    if let Some(owner) = fields.owner {
        user_update = user_update.owner(owner);
    }
    if let Some((filename, bytes)) = fields.photo.as_ref() {
        let stored = save_photo(filename, bytes).await?;
        user_update = user_update.photo_filename(stored);
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
