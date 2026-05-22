use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = contacts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Contact {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub account_id: Option<i32>,
    pub organization_id: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = contacts)]
pub struct NewContact {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub account_id: Option<i32>,
    pub organization_id: Option<i32>,
}

impl NewContact {
    pub fn new(first_name: String, last_name: String) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            first_name,
            last_name,
            email: None,
            phone: None,
            address: None,
            city: None,
            region: None,
            country: None,
            postal_code: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            account_id: None,
            organization_id: None,
        }
    }

    pub fn with_account(mut self, account_id: i32) -> Self {
        self.account_id = Some(account_id);
        self
    }

    pub fn with_organization(mut self, organization_id: i32) -> Self {
        self.organization_id = Some(organization_id);
        self
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = contacts)]
pub struct ContactUpdate {
    pub organization_id: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl ContactUpdate {
    pub fn new() -> Self {
        Self {
            organization_id: None,
            first_name: None,
            last_name: None,
            email: None,
            phone: None,
            address: None,
            city: None,
            region: None,
            country: None,
            postal_code: None,
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
        }
    }

    pub fn organization_id(mut self, organization_id: i32) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub fn first_name(mut self, first_name: String) -> Self {
        self.first_name = Some(first_name);
        self
    }

    pub fn last_name(mut self, last_name: String) -> Self {
        self.last_name = Some(last_name);
        self
    }

    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn phone(mut self, phone: String) -> Self {
        self.phone = Some(phone);
        self
    }

    pub fn address(mut self, address: String) -> Self {
        self.address = Some(address);
        self
    }

    pub fn city(mut self, city: String) -> Self {
        self.city = Some(city);
        self
    }

    pub fn region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub fn country(mut self, country: String) -> Self {
        self.country = Some(country);
        self
    }

    pub fn postal_code(mut self, postal_code: String) -> Self {
        self.postal_code = Some(postal_code);
        self
    }
}

impl Default for ContactUpdate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl crate::database::JsonApiResource for Contact {
    const TYPE: &'static str = "contacts";
    type Repository = crate::database::ContactRepository;
    type NewModel = NewContact;
    type UpdateModel = ContactUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "contacts"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "first_name",
            "last_name",
            "email",
            "phone",
            "address",
            "city",
            "region",
            "country",
            "postal_code",
            "created_at",
            "updated_at",
            "deleted_at",
            "account_id",
            "organization_id",
        ]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        vec![
            ("first_name", json!(self.first_name)),
            ("last_name", json!(self.last_name)),
            ("email", json!(self.email)),
            ("phone", json!(self.phone)),
            ("address", json!(self.address)),
            ("city", json!(self.city)),
            ("region", json!(self.region)),
            ("country", json!(self.country)),
            ("postal_code", json!(self.postal_code)),
            ("account_id", json!(self.account_id)),
            ("organization_id", json!(self.organization_id)),
            ("created_at", json!(self.created_at.and_utc().to_rfc3339())),
            ("updated_at", json!(self.updated_at.and_utc().to_rfc3339())),
            (
                "deleted_at",
                json!(self.deleted_at.map(|dt| dt.and_utc().to_rfc3339())),
            ),
        ]
    }

    fn relationships() -> Vec<crate::database::jsonapi_resource::RelationshipMeta> {
        use crate::database::jsonapi_resource::RelationshipMeta;
        vec![
            RelationshipMeta::belongs_to("organization", "organizations", "organization_id"),
            RelationshipMeta::belongs_to("account", "accounts", "account_id"),
        ]
    }

    fn repository(pool: crate::database::pool::DbPool) -> Self::Repository {
        crate::database::ContactRepository::new(pool)
    }

    async fn load_related_to_one(
        pool: &crate::database::pool::DbPool,
        resource: &Self,
        _relationship_meta: &crate::database::jsonapi_resource::RelationshipMeta,
        _id: i32,
        relationship: &str,
    ) -> Result<serde_json::Value, appkit_core::error::AppError> {
        use appkit_core::jsonapi::ResourceObject;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use serde_json::json;

        match relationship {
            "organization" => {
                if let Some(org_id) = resource.organization_id {
                    let mut conn = pool
                        .get()
                        .await
                        .map_err(|e| appkit_core::error::AppError::database_error(e))?;

                    let org: Option<crate::database::models::Organization> = RunQueryDsl::first(
                        organizations::table
                            .filter(organizations::id.eq(org_id))
                            .select(crate::database::models::Organization::as_select()),
                        &mut conn,
                    )
                    .await
                    .optional()
                    .map_err(|e| appkit_core::error::AppError::database_error(e))?;

                    if let Some(org) = org {
                        let mut resource_obj =
                            ResourceObject::new("organizations", org.id.to_string());
                        resource_obj = resource_obj.set_attribute("name", json!(org.name));
                        resource_obj = resource_obj.set_attribute("email", json!(org.email));
                        resource_obj = resource_obj.set_attribute("phone", json!(org.phone));
                        resource_obj = resource_obj.set_attribute("address", json!(org.address));
                        resource_obj = resource_obj.set_attribute("city", json!(org.city));
                        resource_obj = resource_obj.set_attribute("region", json!(org.region));
                        resource_obj = resource_obj.set_attribute("country", json!(org.country));
                        resource_obj =
                            resource_obj.set_attribute("postal_code", json!(org.postal_code));
                        resource_obj =
                            resource_obj.set_attribute("account_id", json!(org.account_id));
                        resource_obj = resource_obj.set_attribute(
                            "created_at",
                            json!(org.created_at.and_utc().to_rfc3339()),
                        );
                        resource_obj = resource_obj.set_attribute(
                            "updated_at",
                            json!(org.updated_at.and_utc().to_rfc3339()),
                        );
                        resource_obj = resource_obj.set_attribute(
                            "deleted_at",
                            json!(org
                                .deleted_at
                                .map(|dt: NaiveDateTime| dt.and_utc().to_rfc3339())),
                        );

                        return Ok(json!({ "data": resource_obj }));
                    }
                }
                Ok(json!({ "data": null }))
            }
            "account" => {
                if let Some(acc_id) = resource.account_id {
                    let mut conn = pool
                        .get()
                        .await
                        .map_err(|e| appkit_core::error::AppError::database_error(e))?;

                    let acc: Option<crate::database::models::Account> = RunQueryDsl::first(
                        accounts::table
                            .filter(accounts::id.eq(acc_id))
                            .select(crate::database::models::Account::as_select()),
                        &mut conn,
                    )
                    .await
                    .optional()
                    .map_err(|e| appkit_core::error::AppError::database_error(e))?;

                    if let Some(acc) = acc {
                        let mut resource_obj = ResourceObject::new("accounts", acc.id.to_string());
                        resource_obj = resource_obj.set_attribute("name", json!(acc.name));
                        resource_obj = resource_obj.set_attribute(
                            "created_at",
                            json!(acc.created_at.and_utc().to_rfc3339()),
                        );
                        resource_obj = resource_obj.set_attribute(
                            "updated_at",
                            json!(acc.updated_at.and_utc().to_rfc3339()),
                        );

                        return Ok(json!({ "data": resource_obj }));
                    }
                }
                Ok(json!({ "data": null }))
            }
            _ => Err(appkit_core::error::AppError::NotImplemented),
        }
    }

    async fn load_related_to_one_batch(
        pool: &crate::database::pool::DbPool,
        resources: &[&Self],
        _relationship_meta: &crate::database::jsonapi_resource::RelationshipMeta,
        relationship: &str,
    ) -> Result<
        std::collections::HashMap<i32, appkit_core::jsonapi::ResourceObject>,
        appkit_core::error::AppError,
    > {
        use appkit_core::jsonapi::ResourceObject;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        use serde_json::json;
        use std::collections::{HashMap, HashSet};

        match relationship {
            "organization" => {
                let org_ids: Vec<i32> = resources
                    .iter()
                    .filter_map(|c| c.organization_id)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();

                if org_ids.is_empty() {
                    return Ok(HashMap::new());
                }

                let mut conn = pool
                    .get()
                    .await
                    .map_err(|e| appkit_core::error::AppError::database_error(e))?;

                let orgs: Vec<crate::database::models::Organization> = RunQueryDsl::load(
                    organizations::table
                        .filter(organizations::id.eq_any(&org_ids))
                        .select(crate::database::models::Organization::as_select()),
                    &mut conn,
                )
                .await
                .map_err(|e| appkit_core::error::AppError::database_error(e))?;

                let mut org_map: HashMap<i32, crate::database::models::Organization> =
                    HashMap::new();
                for org in orgs {
                    org_map.insert(org.id, org);
                }

                let mut result = HashMap::new();
                for contact in resources {
                    if let Some(org_id) = contact.organization_id {
                        if let Some(org) = org_map.get(&org_id) {
                            let contact_id = contact.id;
                            let mut resource_obj =
                                ResourceObject::new("organizations", org.id.to_string());
                            resource_obj = resource_obj.set_attribute("name", json!(org.name));
                            resource_obj = resource_obj.set_attribute("email", json!(org.email));
                            resource_obj = resource_obj.set_attribute("phone", json!(org.phone));
                            resource_obj =
                                resource_obj.set_attribute("address", json!(org.address));
                            resource_obj = resource_obj.set_attribute("city", json!(org.city));
                            resource_obj = resource_obj.set_attribute("region", json!(org.region));
                            resource_obj =
                                resource_obj.set_attribute("country", json!(org.country));
                            resource_obj =
                                resource_obj.set_attribute("postal_code", json!(org.postal_code));
                            resource_obj =
                                resource_obj.set_attribute("account_id", json!(org.account_id));
                            resource_obj = resource_obj.set_attribute(
                                "created_at",
                                json!(org.created_at.and_utc().to_rfc3339()),
                            );
                            resource_obj = resource_obj.set_attribute(
                                "updated_at",
                                json!(org.updated_at.and_utc().to_rfc3339()),
                            );
                            resource_obj = resource_obj.set_attribute(
                                "deleted_at",
                                json!(org
                                    .deleted_at
                                    .map(|dt: NaiveDateTime| dt.and_utc().to_rfc3339())),
                            );
                            result.insert(contact_id, resource_obj);
                        }
                    }
                }

                Ok(result)
            }
            "account" => {
                let acc_ids: Vec<i32> = resources
                    .iter()
                    .filter_map(|c| c.account_id)
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();

                if acc_ids.is_empty() {
                    return Ok(HashMap::new());
                }

                let mut conn = pool
                    .get()
                    .await
                    .map_err(|e| appkit_core::error::AppError::database_error(e))?;

                let accs: Vec<crate::database::models::Account> = RunQueryDsl::load(
                    accounts::table
                        .filter(accounts::id.eq_any(&acc_ids))
                        .select(crate::database::models::Account::as_select()),
                    &mut conn,
                )
                .await
                .map_err(|e| appkit_core::error::AppError::database_error(e))?;

                let mut acc_map: HashMap<i32, crate::database::models::Account> = HashMap::new();
                for acc in accs {
                    acc_map.insert(acc.id, acc);
                }

                let mut result = HashMap::new();
                for contact in resources {
                    if let Some(acc_id) = contact.account_id {
                        if let Some(acc) = acc_map.get(&acc_id) {
                            let contact_id = contact.id;
                            let mut resource_obj =
                                ResourceObject::new("accounts", acc.id.to_string());
                            resource_obj = resource_obj.set_attribute("name", json!(acc.name));
                            resource_obj = resource_obj.set_attribute(
                                "created_at",
                                json!(acc.created_at.and_utc().to_rfc3339()),
                            );
                            resource_obj = resource_obj.set_attribute(
                                "updated_at",
                                json!(acc.updated_at.and_utc().to_rfc3339()),
                            );
                            result.insert(contact_id, resource_obj);
                        }
                    }
                }

                Ok(result)
            }
            _ => Err(appkit_core::error::AppError::NotImplemented),
        }
    }
}
