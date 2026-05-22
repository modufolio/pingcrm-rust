use crate::database::models::{AuditLog, NewAuditLog};
use chrono::Utc;

use super::factory_trait::Factory;

#[derive(Clone, Default)]
pub struct AuditLogFactory {
    entity: Option<String>,
    entity_id: Option<String>,
    action: Option<String>,
    changes: Option<serde_json::Value>,
    user_id: Option<i32>,
    user_email: Option<String>,
    status: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    details: Option<String>,
    resource: Option<String>,
}

impl AuditLogFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_entity(mut self, entity: impl Into<String>) -> Self {
        self.entity = Some(entity.into());
        self
    }

    pub fn with_entity_id(mut self, entity_id: impl Into<String>) -> Self {
        self.entity_id = Some(entity_id.into());
        self
    }

    pub fn with_user(mut self, user_id: i32, user_email: String) -> Self {
        self.user_id = Some(user_id);
        self.user_email = Some(user_email);
        self
    }

    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn with_changes(mut self, changes: serde_json::Value) -> Self {
        self.changes = Some(changes);
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }
}

impl Factory for AuditLogFactory {
    type Model = AuditLog;
    type NewModel = NewAuditLog;

    fn build(&self) -> NewAuditLog {
        let entity = self.entity.clone().unwrap_or_else(|| "User".to_string());

        let entity_id = self.entity_id.clone().unwrap_or_else(|| "1".to_string());

        let action = self.action.clone().unwrap_or_else(|| "create".to_string());

        let now = Utc::now().naive_utc();

        let mut changes_map = serde_json::Map::new();
        if let Some(status) = &self.status {
            changes_map.insert("status".to_string(), serde_json::json!(status));
        }
        if let Some(ip) = &self.ip_address {
            changes_map.insert("ip_address".to_string(), serde_json::json!(ip));
        }
        if let Some(ua) = &self.user_agent {
            changes_map.insert("user_agent".to_string(), serde_json::json!(ua));
        }
        if let Some(details) = &self.details {
            changes_map.insert("details".to_string(), serde_json::json!(details));
        }
        if let Some(resource) = &self.resource {
            changes_map.insert("resource".to_string(), serde_json::json!(resource));
        }
        if let Some(email) = &self.user_email {
            changes_map.insert("user_email".to_string(), serde_json::json!(email));
        }

        let changes = if !changes_map.is_empty() {
            Some(serde_json::Value::Object(changes_map).to_string())
        } else {
            self.changes.as_ref().map(|c| c.to_string())
        };

        NewAuditLog {
            entity,
            entity_id,
            action,
            changes,
            user_id: self.user_id,
            created_at: now,
            updated_at: now,
        }
    }

    fn table_name() -> &'static str {
        "audit_logs"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_factory_build() {
        let factory = AuditLogFactory::default();
        let log = factory.build();

        assert_eq!(log.entity, "User");
        assert_eq!(log.action, "create");
    }

    #[test]
    fn test_audit_log_factory_with_user() {
        let factory = AuditLogFactory::default()
            .with_user(1, "user@example.com".to_string())
            .with_action("update")
            .with_entity("Product");

        let log = factory.build();

        assert_eq!(log.user_id, Some(1));
        assert_eq!(log.entity, "Product");
        assert_eq!(log.action, "update");
    }
}
