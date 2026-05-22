use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::audit_logs;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = audit_logs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AuditLog {
    pub id: i32,
    pub entity: String,
    pub entity_id: String,
    pub action: String,
    pub changes: Option<String>,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = audit_logs)]
pub struct NewAuditLog {
    pub entity: String,
    pub entity_id: String,
    pub action: String,
    pub changes: Option<String>,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl NewAuditLog {
    pub fn new(entity: String, entity_id: String, action: String) -> Self {
        let now = Utc::now().naive_utc();
        Self {
            entity,
            entity_id,
            action,
            changes: None,
            user_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_user(mut self, user_id: i32) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_changes(mut self, changes: serde_json::Value) -> Self {
        self.changes = Some(changes.to_string());
        self
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = audit_logs)]
pub struct AuditLogUpdate {
    pub updated_at: NaiveDateTime,
}

impl AuditLogUpdate {
    pub fn new() -> Self {
        Self {
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Default for AuditLogUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::database::JsonApiResource for AuditLog {
    const TYPE: &'static str = "audit_logs";
    type Repository = crate::database::AuditLogRepository;
    type NewModel = NewAuditLog;
    type UpdateModel = AuditLogUpdate;

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn table_name() -> &'static str {
        "audit_logs"
    }

    fn field_names() -> &'static [&'static str] {
        &[
            "id",
            "entity",
            "entity_id",
            "action",
            "changes",
            "user_id",
            "created_at",
            "updated_at",
        ]
    }

    fn relationships() -> Vec<crate::database::jsonapi_resource::RelationshipMeta> {
        vec![]
    }

    fn attributes(&self) -> Vec<(&'static str, serde_json::Value)> {
        use serde_json::json;

        vec![
            ("entity", json!(self.entity)),
            ("entity_id", json!(self.entity_id)),
            ("action", json!(self.action)),
            ("changes", json!(self.changes)),
            ("user_id", json!(self.user_id)),
            ("created_at", json!(self.created_at.and_utc().to_rfc3339())),
            ("updated_at", json!(self.updated_at.and_utc().to_rfc3339())),
        ]
    }

    fn repository(pool: crate::database::pool::DbPool) -> Self::Repository {
        crate::database::AuditLogRepository::new(pool)
    }
}
