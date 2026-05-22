use crate::database::models::NewAuditLog;
use crate::database::repositories::audit_log_repository::AuditLogRepository;
use crate::events::{LoginFailedEvent, LoginSuccessEvent, PasswordChangedEvent};
use appkit_core::event::EventListener;
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TrackFailedLoginListener {
    audit_log_repo: Arc<AuditLogRepository>,

    failed_attempts: Arc<RwLock<HashMap<String, i32>>>,
}

impl TrackFailedLoginListener {
    pub fn new(audit_log_repo: Arc<AuditLogRepository>) -> Self {
        Self {
            audit_log_repo,
            failed_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    const MAX_FAILED_ATTEMPTS: i32 = 5;
}

#[async_trait]
impl EventListener<LoginFailedEvent> for TrackFailedLoginListener {
    async fn handle(&self, event: &LoginFailedEvent) -> Result<(), Box<dyn Error>> {
        let key = format!("{}:{}", event.email, event.ip);
        let mut attempts = self.failed_attempts.write().await;
        let count = attempts.entry(key.clone()).or_insert(0);
        *count += 1;

        tracing::warn!(
            "Failed login attempt for {} from {} (attempt {}/{}): {}",
            event.email,
            event.ip,
            count,
            Self::MAX_FAILED_ATTEMPTS,
            event.reason
        );

        let changes_json = serde_json::json!({
            "email": event.email,
            "ip": event.ip,
            "reason": event.reason,
            "attempts": count
        });

        let audit_log = NewAuditLog::new(
            "Security".to_string(),
            "login_failed".to_string(),
            "failed_login".to_string(),
        )
        .with_changes(changes_json);

        if let Err(e) = self.audit_log_repo.create(audit_log).await {
            tracing::error!("Failed to create security audit log: {}", e);
        }

        if *count >= Self::MAX_FAILED_ATTEMPTS {
            tracing::error!(
                "Account {} locked due to {} failed login attempts from {}",
                event.email,
                count,
                event.ip
            );

            attempts.remove(&key);
        }

        Ok(())
    }

    fn priority(&self) -> i32 {
        200
    }

    fn name(&self) -> &'static str {
        "TrackFailedLoginListener"
    }
}

pub struct NotifyPasswordChangeListener {
    audit_log_repo: Arc<AuditLogRepository>,
}

impl NotifyPasswordChangeListener {
    pub fn new(audit_log_repo: Arc<AuditLogRepository>) -> Self {
        Self { audit_log_repo }
    }
}

#[async_trait]
impl EventListener<PasswordChangedEvent> for NotifyPasswordChangeListener {
    async fn handle(&self, event: &PasswordChangedEvent) -> Result<(), Box<dyn Error>> {
        tracing::info!(
            "Password changed for user {} from IP {}",
            event.email,
            event.ip
        );

        let changes_json = serde_json::json!({
            "user_id": event.user_id,
            "ip": event.ip,
            "timestamp": event.timestamp.to_string()
        });

        let audit_log = NewAuditLog::new(
            "Security".to_string(),
            event.user_id.to_string(),
            "password_changed".to_string(),
        )
        .with_changes(changes_json)
        .with_user(event.user_id);

        if let Err(e) = self.audit_log_repo.create(audit_log).await {
            tracing::error!("Failed to create audit log for password change: {}", e);
        }

        Ok(())
    }

    fn priority(&self) -> i32 {
        100
    }

    fn name(&self) -> &'static str {
        "NotifyPasswordChangeListener"
    }
}

pub struct Detect2FABypassListener {
    audit_log_repo: Arc<AuditLogRepository>,
}

impl Detect2FABypassListener {
    pub fn new(audit_log_repo: Arc<AuditLogRepository>) -> Self {
        Self { audit_log_repo }
    }

    fn is_suspicious(&self, event: &LoginSuccessEvent) -> bool {
        !event.used_2fa
    }
}

#[async_trait]
impl EventListener<LoginSuccessEvent> for Detect2FABypassListener {
    async fn handle(&self, event: &LoginSuccessEvent) -> Result<(), Box<dyn Error>> {
        if self.is_suspicious(event) {
            tracing::warn!(
                "Suspicious login detected for user {} from {}: 2FA not used",
                event.email,
                event.ip
            );

            let changes_json = serde_json::json!({
                "user_id": event.user_id,
                "email": event.email,
                "ip": event.ip,
                "used_2fa": event.used_2fa,
                "alert": "2FA not used for login"
            });

            let audit_log = NewAuditLog::new(
                "Security".to_string(),
                event.user_id.to_string(),
                "suspicious_login".to_string(),
            )
            .with_changes(changes_json)
            .with_user(event.user_id);

            if let Err(e) = self.audit_log_repo.create(audit_log).await {
                tracing::error!("Failed to create security alert: {}", e);
            }
        }

        Ok(())
    }

    fn priority(&self) -> i32 {
        150
    }

    fn name(&self) -> &'static str {
        "Detect2FABypassListener"
    }
}

pub struct LogSuccessfulLoginListener {
    audit_log_repo: Arc<AuditLogRepository>,
}

impl LogSuccessfulLoginListener {
    pub fn new(audit_log_repo: Arc<AuditLogRepository>) -> Self {
        Self { audit_log_repo }
    }
}

#[async_trait]
impl EventListener<LoginSuccessEvent> for LogSuccessfulLoginListener {
    async fn handle(&self, event: &LoginSuccessEvent) -> Result<(), Box<dyn Error>> {
        tracing::info!(
            "Successful login for user {} from {} (2FA: {})",
            event.email,
            event.ip,
            event.used_2fa
        );

        let changes_json = serde_json::json!({
            "email": event.email,
            "ip": event.ip,
            "used_2fa": event.used_2fa
        });

        let audit_log = NewAuditLog::new(
            "Security".to_string(),
            event.user_id.to_string(),
            "login_success".to_string(),
        )
        .with_changes(changes_json)
        .with_user(event.user_id);

        if let Err(e) = self.audit_log_repo.create(audit_log).await {
            tracing::error!("Failed to create login audit log: {}", e);
        }

        Ok(())
    }

    fn priority(&self) -> i32 {
        50
    }

    fn name(&self) -> &'static str {
        "LogSuccessfulLoginListener"
    }
}
