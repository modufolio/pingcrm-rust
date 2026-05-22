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
use crate::database::{ContactRepository, ContactUpdate, NewContact, OrganizationRepository};
use crate::inertia::DefaultProps;
use appkit_core::jsonapi::{QueryParams, SortDirection};
use appkit_core::security::user::User;
use appkit_core::{
    error::{AppError, AppResult},
    response::AppResponse,
};

#[derive(Debug, Deserialize)]
pub struct ContactListQuery {
    pub search: Option<String>,
    pub trashed: Option<String>,
    #[serde(rename = "sort")]
    pub sort_column: Option<String>,
    #[serde(rename = "direction")]
    pub sort_direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContactRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub organization_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub organization_id: Option<i32>,
}

pub async fn index(
    State(state): State<App>,
    session: Session,
    Query(query): Query<ContactListQuery>,
    request: Request<Body>,
) -> AppResult<Response> {
    let contact_repo = ContactRepository::new(state.db_pool.clone());

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

    let result = contact_repo
        .find_with_params(account_id, query.search.as_deref(), &params)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let org_repo = OrganizationRepository::new(state.db_pool.clone());
    let organizations = org_repo
        .find_by_account(account_id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;
    let org_by_id: std::collections::HashMap<i32, &str> = organizations
        .iter()
        .map(|o| (o.id, o.name.as_str()))
        .collect();

    let contacts_data: Vec<_> = result
        .items
        .iter()
        .map(|c| {
            let organization = c.organization_id.and_then(|id| {
                org_by_id
                    .get(&id)
                    .map(|name| json!({ "id": id, "name": name }))
            });
            json!({
                "id": c.id,
                "name": format!("{} {}", c.first_name, c.last_name),
                "phone": c.phone,
                "city": c.city,
                "organization": organization,
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
            "contacts": {
                "data": contacts_data,
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

    let response = AppResponse::inertia("Contacts/Index")
        .with_props(props)
        .render(&request);

    Ok(response)
}

pub async fn create(
    State(state): State<App>,
    session: Session,
    request: Request<Body>,
) -> AppResult<Response> {
    let org_repo = OrganizationRepository::new(state.db_pool.clone());

    let account_id = request
        .extensions()
        .get::<User>()
        .and_then(|u| u.account_id)
        .unwrap_or(0);

    let organizations = org_repo
        .find_by_account(account_id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let organizations_data: Vec<_> = organizations
        .iter()
        .map(|o| {
            json!({
                "id": o.id,
                "name": o.name,
            })
        })
        .collect();

    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state,
        json!({ "organizations": organizations_data }),
    )
    .await;

    let response = AppResponse::inertia("Contacts/Create")
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
    let contact_repo = ContactRepository::new(state.db_pool.clone());
    let org_repo = OrganizationRepository::new(state.db_pool.clone());

    let contact = contact_repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Contact {} not found", id)))?;

    let account_id = request
        .extensions()
        .get::<User>()
        .and_then(|u| u.account_id)
        .unwrap_or(0);

    let organizations = org_repo
        .find_by_account(account_id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let organizations_data: Vec<_> = organizations
        .iter()
        .map(|o| {
            json!({
                "id": o.id,
                "name": o.name,
            })
        })
        .collect();

    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state,
        json!({
            "contact": {
                "id": contact.id,
                "first_name": contact.first_name,
                "last_name": contact.last_name,
                "email": contact.email,
                "phone": contact.phone,
                "address": contact.address,
                "city": contact.city,
                "region": contact.region,
                "country": contact.country,
                "postal_code": contact.postal_code,
                "organization_id": contact.organization_id,
            },
            "organizations": organizations_data,
        }),
    )
    .await;

    let response = AppResponse::inertia("Contacts/Edit")
        .with_props(props)
        .render(&request);

    Ok(response)
}

pub async fn store(
    State(state): State<App>,
    Extension(user): Extension<User>,
    Json(data): Json<CreateContactRequest>,
) -> AppResult<Response> {
    let contact_repo = ContactRepository::new(state.db_pool.clone());

    let account_id = user
        .account_id
        .ok_or_else(|| AppError::InternalServerError("User account not found".to_string()))?;

    let mut new_contact = NewContact::new(data.first_name, data.last_name).with_account(account_id);
    new_contact.email = data.email;
    new_contact.phone = data.phone;
    new_contact.address = data.address;
    new_contact.city = data.city;
    new_contact.region = data.region;
    new_contact.country = data.country;
    new_contact.postal_code = data.postal_code;
    if let Some(org_id) = data.organization_id {
        new_contact = new_contact.with_organization(org_id);
    }

    contact_repo
        .create(new_contact)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect("/contacts"))
}

pub async fn update(
    State(state): State<App>,
    Path(id): Path<i32>,
    Json(data): Json<UpdateContactRequest>,
) -> AppResult<Response> {
    let contact_repo = ContactRepository::new(state.db_pool.clone());

    let _existing = contact_repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Contact {} not found", id)))?;

    let mut contact_update = ContactUpdate::new();
    if let Some(first_name) = data.first_name {
        contact_update = contact_update.first_name(first_name);
    }
    if let Some(last_name) = data.last_name {
        contact_update = contact_update.last_name(last_name);
    }
    if let Some(email) = data.email {
        contact_update = contact_update.email(email);
    }
    if let Some(phone) = data.phone {
        contact_update = contact_update.phone(phone);
    }
    if let Some(address) = data.address {
        contact_update = contact_update.address(address);
    }
    if let Some(city) = data.city {
        contact_update = contact_update.city(city);
    }
    if let Some(region) = data.region {
        contact_update = contact_update.region(region);
    }
    if let Some(country) = data.country {
        contact_update = contact_update.country(country);
    }
    if let Some(postal_code) = data.postal_code {
        contact_update = contact_update.postal_code(postal_code);
    }
    if let Some(org_id) = data.organization_id {
        contact_update = contact_update.organization_id(org_id);
    }

    contact_repo
        .update(id, contact_update)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect(&format!("/contacts/{}/edit", id)))
}

pub async fn destroy(State(state): State<App>, Path(id): Path<i32>) -> AppResult<Response> {
    let contact_repo = ContactRepository::new(state.db_pool.clone());

    let _existing = contact_repo
        .find_by_id(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("Contact {} not found", id)))?;

    contact_repo
        .delete(id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    Ok(AppResponse::redirect("/contacts"))
}

pub async fn restore(State(_state): State<App>, Path(id): Path<i32>) -> AppResult<Response> {
    Ok(AppResponse::redirect(&format!("/contacts/{}/edit", id)))
}
