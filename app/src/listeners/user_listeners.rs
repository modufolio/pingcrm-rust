use crate::database::models::NewAuditLog;
use crate::database::repositories::audit_log_repository::AuditLogRepository;
use crate::events::{UserCreatedEvent, UserDeletedEvent, UserUpdatedEvent};
use appkit_core::event::EventListener;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;

pub struct SendWelcomeEmailListener;

impl SendWelcomeEmailListener {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventListener<UserCreatedEvent> for SendWelcomeEmailListener {
    async fn handle(&self, event: &UserCreatedEvent) -> Result<(), Box<dyn Error>> {
        tracing::info!(
            "Sending welcome email to {} ({} {})",
            event.email,
            event.first_name,
            event.last_name
        );

        Ok(())
    }

    fn priority(&self) -> i32 {
        100
    }

    fn name(&self) -> &'static str {
        "SendWelcomeEmailListener"
    }
}

pub struct CreateUserAuditLogListener {
    audit_log_repo: Arc<AuditLogRepository>,
}

impl CreateUserAuditLogListener {
    pub fn new(audit_log_repo: Arc<AuditLogRepository>) -> Self {
        Self { audit_log_repo }
    }
}

#[async_trait]
impl EventListener<UserCreatedEvent> for CreateUserAuditLogListener {
    async fn handle(&self, event: &UserCreatedEvent) -> Result<(), Box<dyn Error>> {
        let audit_log = NewAuditLog::new(
            "User".to_string(),
            event.user_id.to_string(),
            "created".to_string(),
        );

        match self.audit_log_repo.create(audit_log).await {
            Ok(_) => {
                tracing::debug!("Created audit log for user.created: {}", event.user_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "Failed to create audit log for user {}: {}",
                    event.user_id,
                    e
                );

                Ok(())
            }
        }
    }

    fn priority(&self) -> i32 {
        200
    }

    fn name(&self) -> &'static str {
        "CreateUserAuditLogListener (UserCreated)"
    }
}

pub struct UpdateUserAuditLogListener {
    audit_log_repo: Arc<AuditLogRepository>,
}

impl UpdateUserAuditLogListener {
    pub fn new(audit_log_repo: Arc<AuditLogRepository>) -> Self {
        Self { audit_log_repo }
    }
}

#[async_trait]
impl EventListener<UserUpdatedEvent> for UpdateUserAuditLogListener {
    async fn handle(&self, event: &UserUpdatedEvent) -> Result<(), Box<dyn Error>> {
        let changes_json = serde_json::to_value(&event.changes)?;

        let audit_log = NewAuditLog::new(
            "User".to_string(),
            event.user_id.to_string(),
            "updated".to_string(),
        )
        .with_changes(changes_json);

        match self.audit_log_repo.create(audit_log).await {
            Ok(_) => {
                tracing::debug!("Created audit log for user.updated: {}", event.user_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "Failed to create audit log for user {}: {}",
                    event.user_id,
                    e
                );
                Ok(())
            }
        }
    }

    fn priority(&self) -> i32 {
        200
    }

    fn name(&self) -> &'static str {
        "CreateUserAuditLogListener (UserUpdated)"
    }
}

pub struct DeleteUserAuditLogListener {
    audit_log_repo: Arc<AuditLogRepository>,
}

impl DeleteUserAuditLogListener {
    pub fn new(audit_log_repo: Arc<AuditLogRepository>) -> Self {
        Self { audit_log_repo }
    }
}

#[async_trait]
impl EventListener<UserDeletedEvent> for DeleteUserAuditLogListener {
    async fn handle(&self, event: &UserDeletedEvent) -> Result<(), Box<dyn Error>> {
        let changes_json = serde_json::json!({
            "deleted_at": event.deleted_at.to_string()
        });

        let audit_log = NewAuditLog::new(
            "User".to_string(),
            event.user_id.to_string(),
            "deleted".to_string(),
        )
        .with_changes(changes_json);

        match self.audit_log_repo.create(audit_log).await {
            Ok(_) => {
                tracing::debug!("Created audit log for user.deleted: {}", event.user_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "Failed to create audit log for user {}: {}",
                    event.user_id,
                    e
                );
                Ok(())
            }
        }
    }

    fn priority(&self) -> i32 {
        200
    }

    fn name(&self) -> &'static str {
        "CreateUserAuditLogListener (UserDeleted)"
    }
}

pub struct ClearUserCacheListener;

impl ClearUserCacheListener {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventListener<UserCreatedEvent> for ClearUserCacheListener {
    async fn handle(&self, event: &UserCreatedEvent) -> Result<(), Box<dyn Error>> {
        tracing::debug!("Clearing cache for user {}", event.user_id);

        Ok(())
    }

    fn priority(&self) -> i32 {
        50
    }

    fn name(&self) -> &'static str {
        "ClearUserCacheListener (UserCreated)"
    }
}

pub struct ClearUserCacheOnUpdateListener;

impl ClearUserCacheOnUpdateListener {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventListener<UserUpdatedEvent> for ClearUserCacheOnUpdateListener {
    async fn handle(&self, event: &UserUpdatedEvent) -> Result<(), Box<dyn Error>> {
        tracing::debug!("Clearing cache for updated user {}", event.user_id);
        Ok(())
    }

    fn priority(&self) -> i32 {
        50
    }

    fn name(&self) -> &'static str {
        "ClearUserCacheListener (UserUpdated)"
    }
}

pub struct ClearUserCacheOnDeleteListener;

impl ClearUserCacheOnDeleteListener {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EventListener<UserDeletedEvent> for ClearUserCacheOnDeleteListener {
    async fn handle(&self, event: &UserDeletedEvent) -> Result<(), Box<dyn Error>> {
        tracing::debug!("Clearing cache for deleted user {}", event.user_id);
        Ok(())
    }

    fn priority(&self) -> i32 {
        50
    }

    fn name(&self) -> &'static str {
        "ClearUserCacheListener (UserDeleted)"
    }
}

pub struct NotifyAdminUserCreatedListener {
    environment: String,
}

impl NotifyAdminUserCreatedListener {
    pub fn new(environment: String) -> Self {
        Self { environment }
    }
}

#[async_trait]
impl EventListener<UserCreatedEvent> for NotifyAdminUserCreatedListener {
    async fn handle(&self, event: &UserCreatedEvent) -> Result<(), Box<dyn Error>> {
        tracing::info!(
            "Admin notification: New user {} {} ({}) created",
            event.first_name,
            event.last_name,
            event.email
        );

        Ok(())
    }

    fn priority(&self) -> i32 {
        10
    }

    fn should_handle(&self, _event: &UserCreatedEvent) -> bool {
        self.environment == "production"
    }

    fn name(&self) -> &'static str {
        "NotifyAdminUserCreatedListener"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener_priorities() {
        assert_eq!(SendWelcomeEmailListener::new().priority(), 100);
        assert_eq!(ClearUserCacheListener::new().priority(), 50);
        assert_eq!(
            NotifyAdminUserCreatedListener::new("test".to_string()).priority(),
            10
        );
    }

    #[test]
    fn test_notify_admin_should_handle() {
        let prod_listener = NotifyAdminUserCreatedListener::new("production".to_string());
        let dev_listener = NotifyAdminUserCreatedListener::new("development".to_string());

        let event = UserCreatedEvent {
            user_id: 1,
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            account_id: None,
        };

        assert!(prod_listener.should_handle(&event));
        assert!(!dev_listener.should_handle(&event));
    }
}
