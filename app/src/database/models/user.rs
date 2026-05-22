use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::users;
use appkit_core::security::user::{User as SecurityUser, UserRole, UserStatus};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub password_version: i32,
    pub owner: bool,
    pub photo_filename: Option<String>,
    pub roles: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub account_expires_at: Option<NaiveDateTime>,
    pub credentials_expire_at: Option<NaiveDateTime>,
    pub account_status: String,
    pub enabled: bool,
    pub locked: bool,
    pub locked_at: Option<NaiveDateTime>,
    pub locked_reason: Option<String>,
    pub account_id: Option<i32>,
}

impl User {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub fn is_active(&self) -> bool {
        self.enabled && !self.locked && self.deleted_at.is_none()
    }

    pub fn get_roles(&self) -> Vec<String> {
        self.roles
            .as_ref()
            .and_then(|r| serde_json::from_str(r).ok())
            .unwrap_or_default()
    }

    pub fn to_security_user(&self) -> SecurityUser {
        SecurityUser {
            id: self.id,
            email: self.email.clone(),
            password_hash: self.password.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            role: self.parse_role(),
            status: self.parse_status(),
            created_at: DateTime::<Utc>::from_naive_utc_and_offset(self.created_at, Utc),
            updated_at: DateTime::<Utc>::from_naive_utc_and_offset(self.updated_at, Utc),
            last_login_at: None,
            failed_login_attempts: 0,
            totp_secret: None,
            two_factor_enabled: false,
            account_id: self.account_id,
        }
    }

    fn parse_role(&self) -> UserRole {
        let roles = self.get_roles();
        if roles.contains(&"ROLE_SUPER_ADMIN".to_string()) {
            UserRole::SuperAdmin
        } else if roles.contains(&"ROLE_ADMIN".to_string()) {
            UserRole::Admin
        } else if roles.contains(&"ROLE_USER".to_string()) {
            UserRole::User
        } else {
            UserRole::Guest
        }
    }

    fn parse_status(&self) -> UserStatus {
        if !self.enabled || self.deleted_at.is_some() {
            UserStatus::Disabled
        } else if self.locked {
            UserStatus::Locked
        } else if self
            .account_expires_at
            .map(|exp| exp < Utc::now().naive_utc())
            .unwrap_or(false)
        {
            UserStatus::Expired
        } else if self.account_status == "active" {
            UserStatus::Active
        } else {
            UserStatus::Disabled
        }
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub password_version: i32,
    pub owner: bool,
    pub photo_filename: Option<String>,
    pub roles: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub account_expires_at: Option<NaiveDateTime>,
    pub credentials_expire_at: Option<NaiveDateTime>,
    pub account_status: String,
    pub enabled: bool,
    pub locked: bool,
    pub locked_at: Option<NaiveDateTime>,
    pub locked_reason: Option<String>,
    pub account_id: Option<i32>,
}

impl NewUser {
    pub fn new(email: String, password: String, first_name: String, last_name: String) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            email,
            first_name,
            last_name,
            password,
            password_version: 1,
            owner: false,
            photo_filename: None,
            roles: Some(r#"["ROLE_USER"]"#.to_string()),
            created_at: now,
            updated_at: now,
            deleted_at: None,
            account_expires_at: None,
            credentials_expire_at: None,
            account_status: "active".to_string(),
            enabled: true,
            locked: false,
            locked_at: None,
            locked_reason: None,
            account_id: None,
        }
    }

    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = Some(serde_json::to_string(&roles).unwrap_or_default());
        self
    }

    pub fn with_account(mut self, account_id: i32, owner: bool) -> Self {
        self.account_id = Some(account_id);
        self.owner = owner;
        self
    }

    pub fn with_status(mut self, status: String) -> Self {
        self.account_status = status;
        self
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = users)]
pub struct UserUpdate {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub password: Option<String>,
    pub password_version: Option<i32>,
    pub owner: Option<bool>,
    pub photo_filename: Option<String>,
    pub roles: Option<String>,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub account_expires_at: Option<NaiveDateTime>,
    pub credentials_expire_at: Option<NaiveDateTime>,
    pub account_status: Option<String>,
    pub enabled: Option<bool>,
    pub locked: Option<bool>,
    pub locked_at: Option<NaiveDateTime>,
    pub locked_reason: Option<String>,
}

impl UserUpdate {
    pub fn new() -> Self {
        Self {
            email: None,
            first_name: None,
            last_name: None,
            password: None,
            password_version: None,
            owner: None,
            photo_filename: None,
            roles: None,
            updated_at: chrono::Utc::now().naive_utc(),
            deleted_at: None,
            account_expires_at: None,
            credentials_expire_at: None,
            account_status: None,
            enabled: None,
            locked: None,
            locked_at: None,
            locked_reason: None,
        }
    }

    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
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

    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self.password_version = Some(self.password_version.unwrap_or(1) + 1);
        self
    }

    pub fn owner(mut self, owner: bool) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn photo_filename(mut self, photo_filename: String) -> Self {
        self.photo_filename = Some(photo_filename);
        self
    }

    pub fn roles(mut self, roles: Vec<String>) -> Self {
        self.roles = Some(serde_json::to_string(&roles).unwrap_or_default());
        self
    }

    pub fn status(mut self, status: String) -> Self {
        self.account_status = Some(status);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    pub fn locked(mut self, locked: bool, reason: Option<String>) -> Self {
        self.locked = Some(locked);
        if locked {
            self.locked_at = Some(chrono::Utc::now().naive_utc());
            self.locked_reason = reason;
        } else {
            self.locked_at = None;
            self.locked_reason = None;
        }
        self
    }
}

impl Default for UserUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::database::JsonApiResource for User {
    const TYPE: &'static str = "users";
    type Repository = crate::database::UserRepository;
    type NewModel = NewUser;
    type UpdateModel = UserUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "users"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "email",
            "first_name",
            "last_name",
            "password",
            "password_version",
            "owner",
            "photo_filename",
            "roles",
            "created_at",
            "updated_at",
            "deleted_at",
            "account_expires_at",
            "credentials_expire_at",
            "account_status",
            "enabled",
            "locked",
            "locked_at",
            "locked_reason",
            "account_id",
        ]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        vec![
            ("email", json!(self.email)),
            ("first_name", json!(self.first_name)),
            ("last_name", json!(self.last_name)),
            ("owner", json!(self.owner)),
            ("photo", json!(self.photo_filename)),
            ("created_at", json!(self.created_at.and_utc().to_rfc3339())),
            ("updated_at", json!(self.updated_at.and_utc().to_rfc3339())),
            (
                "deleted_at",
                json!(self.deleted_at.map(|dt| dt.and_utc().to_rfc3339())),
            ),
            ("account_id", json!(self.account_id)),
        ]
    }

    fn relationships() -> Vec<crate::database::jsonapi_resource::RelationshipMeta> {
        use crate::database::jsonapi_resource::RelationshipMeta;
        vec![RelationshipMeta::belongs_to(
            "account",
            "accounts",
            "account_id",
        )]
    }

    fn repository(pool: crate::database::pool::DbPool) -> Self::Repository {
        crate::database::UserRepository::new(pool)
    }
}
