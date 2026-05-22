use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    UserLogin,
    UserLogout,
    UserRegistration,
    PasswordChange,
    TwoFactorEnabled,
    TwoFactorDisabled,
    TwoFactorVerified,
    AccountLocked,
    AccountUnlocked,
    PermissionChanged,
    DataAccessed,
    DataModified,
    DataDeleted,
    SecurityViolation,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::UserLogin => "user_login",
            Self::UserLogout => "user_logout",
            Self::UserRegistration => "user_registration",
            Self::PasswordChange => "password_change",
            Self::TwoFactorEnabled => "2fa_enabled",
            Self::TwoFactorDisabled => "2fa_disabled",
            Self::TwoFactorVerified => "2fa_verified",
            Self::AccountLocked => "account_locked",
            Self::AccountUnlocked => "account_unlocked",
            Self::PermissionChanged => "permission_changed",
            Self::DataAccessed => "data_accessed",
            Self::DataModified => "data_modified",
            Self::DataDeleted => "data_deleted",
            Self::SecurityViolation => "security_violation",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub event_type: AuditEventType,
    pub user_id: Option<Uuid>,
    pub user_email: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource: Option<String>,
    pub action: String,
    pub status: String,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct AuditLogger;

impl AuditLogger {
    pub fn new(_db_pool: ()) -> Self {
        Self
    }

    pub fn stdout_only() -> Self {
        Self
    }

    pub async fn log(
        &self,
        event_type: AuditEventType,
        user_id: Option<Uuid>,
        user_email: Option<String>,
        action: impl Into<String>,
        status: impl Into<String>,
        details: Option<String>,
    ) {
        let action = action.into();
        let status = status.into();

        tracing::info!(
            event_type = %event_type,
            user_id = ?user_id,
            user_email = ?user_email,
            action = %action,
            status = %status,
            details = ?details,
            "Audit log entry"
        );
    }

    pub async fn log_login_success(&self, user_id: Uuid, email: &str, ip: Option<String>) {
        self.log(
            AuditEventType::UserLogin,
            Some(user_id),
            Some(email.to_string()),
            "User logged in successfully",
            "success",
            ip.map(|ip| format!("IP: {}", ip)),
        )
        .await;
    }

    pub async fn log_login_failure(&self, email: &str, reason: &str, ip: Option<String>) {
        self.log(
            AuditEventType::UserLogin,
            None,
            Some(email.to_string()),
            format!("Login attempt failed: {}", reason),
            "failure",
            ip.map(|ip| format!("IP: {}", ip)),
        )
        .await;
    }

    pub async fn log_logout(&self, user_id: Uuid, email: &str) {
        self.log(
            AuditEventType::UserLogout,
            Some(user_id),
            Some(email.to_string()),
            "User logged out",
            "success",
            None,
        )
        .await;
    }

    pub async fn log_2fa_enabled(&self, user_id: Uuid, email: &str) {
        self.log(
            AuditEventType::TwoFactorEnabled,
            Some(user_id),
            Some(email.to_string()),
            "Two-factor authentication enabled",
            "success",
            None,
        )
        .await;
    }

    pub async fn log_2fa_disabled(&self, user_id: Uuid, email: &str) {
        self.log(
            AuditEventType::TwoFactorDisabled,
            Some(user_id),
            Some(email.to_string()),
            "Two-factor authentication disabled",
            "success",
            None,
        )
        .await;
    }

    pub async fn log_security_violation(&self, details: String, ip: Option<String>) {
        self.log(
            AuditEventType::SecurityViolation,
            None,
            None,
            "Security violation detected",
            "alert",
            Some(format!(
                "{}{}",
                details,
                ip.map(|ip| format!(" | IP: {}", ip)).unwrap_or_default()
            )),
        )
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stdout_logger() {
        let logger = AuditLogger::stdout_only();

        logger
            .log_login_success(
                Uuid::new_v4(),
                "test@example.com",
                Some("127.0.0.1".to_string()),
            )
            .await;
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(AuditEventType::UserLogin.to_string(), "user_login");
        assert_eq!(AuditEventType::TwoFactorEnabled.to_string(), "2fa_enabled");
        assert_eq!(
            AuditEventType::SecurityViolation.to_string(),
            "security_violation"
        );
    }
}
