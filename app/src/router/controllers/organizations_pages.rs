use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::Request,
    response::Response,
    Extension, Json,
};
use serde::Deserialize;
use serde_json::json;
use tower_sessions::Session;

use crate::app::App;
use crate::database::{
    ContactRepository, NewOrganization, OrganizationRepository, OrganizationUpdate,
};
use crate::inertia::DefaultProps;
use appkit_core::jsonapi::{QueryParams, SortDirection};
use appkit_core::security::user::User;
use appkit_core::{
    error::{AppError, AppResult},
    response::AppResponse,
};

#[derive(Debug, Deserialize)]
pub struct OrganizationListQuery {
    pub search: Option<String>,
    pub trashed: Option<String>,
    #[serde(rename = "sort")]
    pub sort_column: Option<String>,
    #[serde(rename = "direction")]
    pub sort_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
}

pub async fn index(
    State(state): State<App>,
    session: Session,
    Query(query): Query<OrganizationListQuery>,
    request: Request<Body>,
) -> AppResult<Response> {
    let org_repo = OrganizationRepository::new(state.db_pool.clone());

    let account_id = request
        .extensions()
        .get::<User>()
        .and_then(|u| u.account_id)
        .ok_or_else(|| AppError::InternalServerError("User account not found".to_string()))?;

    let mut params = QueryParams::default();
    params.page.size = 15;
    if let Some(ref col) = query.sort_column {
        let dir = match query.sort_direction.as_deref() {
            Some("desc") => SortDirection::Descending,
            _ => SortDirection::Ascending,
        };
        params.sort.push((col.clone(), dir));
    }

    let result = org_repo
        .find_with_params(account_id, query.search.as_deref(), &params)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let organizations_data: Vec<_> = result
        .items
        .iter()
        .map(|o| {
            json!({
                "id": o.id,
                "name": o.name,
                "phone": o.phone,
                "city": o.city,
            })
        })
        .collect();

    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state,
        json!({
            "filters": {
                "search": query.search,
                "trashed": query.trashed,
                "sortColumn": query.sort_column,
                "sortDirection": query.sort_direction.as_deref().unwrap_or("asc"),
            },
            "organizations": {
                "data": organizations_data,
                "meta": {
                    "total": result.total,
                    "per_page": result.per_page,
                    "current_page": result.page,
                    "last_page": result.last_page(),
                }
            },
        }),
    )
    .await;

    let response = AppResponse::inertia("Organizations/Index")
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

    let response = AppResponse::inertia("Organizations/Create")
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
    let org_repo = OrganizationRepository::new(state.db_pool.clone());
    let contact_repo = ContactRepository::new(state.db_pool.clone());

    let organization = org_repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Organization {} not found", id)))?;

    let account_id = request
        .extensions()
        .get::<User>()
        .and_then(|u| u.account_id)
        .unwrap_or(0);

    let all_contacts = contact_repo
        .find_by_account(account_id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let org_contacts: Vec<_> = all_contacts
        .iter()
        .filter(|c| c.organization_id == Some(id))
        .map(|c| {
            json!({
                "id": c.id,
                "name": format!("{} {}", c.first_name, c.last_name),
                "city": c.city,
                "phone": c.phone,
            })
        })
        .collect();

    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state,
        json!({
            "organization": {
                "id": organization.id,
                "name": organization.name,
                "email": organization.email,
                "phone": organization.phone,
                "address": organization.address,
                "city": organization.city,
                "region": organization.region,
                "country": organization.country,
                "postal_code": organization.postal_code,
                "contacts": org_contacts,
            },
        }),
    )
    .await;

    let response = AppResponse::inertia("Organizations/Edit")
        .with_props(props)
        .render(&request);

    Ok(response)
}

pub async fn store(
    State(state): State<App>,
    Extension(user): Extension<User>,
    Json(data): Json<CreateOrganizationRequest>,
) -> AppResult<Response> {
    let org_repo = OrganizationRepository::new(state.db_pool.clone());

    let account_id = user
        .account_id
        .ok_or_else(|| AppError::InternalServerError("User account not found".to_string()))?;

    let mut new_org = NewOrganization::new(data.name).with_account(account_id);
    new_org.email = data.email;
    new_org.phone = data.phone;
    new_org.address = data.address;
    new_org.city = data.city;
    new_org.region = data.region;
    new_org.country = data.country;
    new_org.postal_code = data.postal_code;

    org_repo
        .create(new_org)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect("/organizations"))
}

pub async fn update(
    State(state): State<App>,
    Path(id): Path<i32>,
    Json(data): Json<UpdateOrganizationRequest>,
) -> AppResult<Response> {
    let org_repo = OrganizationRepository::new(state.db_pool.clone());

    let _existing = org_repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Organization {} not found", id)))?;

    let mut org_update = OrganizationUpdate::new();
    if let Some(name) = data.name {
        org_update = org_update.name(name);
    }
    if let Some(email) = data.email {
        org_update = org_update.email(email);
    }
    if let Some(phone) = data.phone {
        org_update = org_update.phone(phone);
    }
    if let Some(address) = data.address {
        org_update = org_update.address(address);
    }
    if let Some(city) = data.city {
        org_update = org_update.city(city);
    }
    if let Some(region) = data.region {
        org_update = org_update.region(region);
    }
    if let Some(country) = data.country {
        org_update = org_update.country(country);
    }
    if let Some(postal_code) = data.postal_code {
        org_update = org_update.postal_code(postal_code);
    }

    org_repo
        .update(id, org_update)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect(&format!(
        "/organizations/{}/edit",
        id
    )))
}

pub async fn destroy(State(state): State<App>, Path(id): Path<i32>) -> AppResult<Response> {
    let org_repo = OrganizationRepository::new(state.db_pool.clone());

    let _existing = org_repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Organization {} not found", id)))?;

    org_repo
        .delete(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect("/organizations"))
}

pub async fn restore(State(_state): State<App>, Path(id): Path<i32>) -> AppResult<Response> {
    Ok(AppResponse::redirect(&format!(
        "/organizations/{}/edit",
        id
    )))
}
