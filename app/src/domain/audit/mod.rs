use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i32,
    pub entity: String,
    pub entity_id: String,
    pub action: String,
    pub changes: Option<String>,
    pub user_id: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AuditEventType {
    Login,
    Logout,
    LoginFailed,
    PasswordChange,
    TwoFactorEnabled,
    TwoFactorDisabled,
    AccountLocked,
    AccountUnlocked,
    UserCreated,
    UserUpdated,
    UserDeleted,
}

impl AuditEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Logout => "logout",
            Self::LoginFailed => "login_failed",
            Self::PasswordChange => "password_change",
            Self::TwoFactorEnabled => "2fa_enabled",
            Self::TwoFactorDisabled => "2fa_disabled",
            Self::AccountLocked => "account_locked",
            Self::AccountUnlocked => "account_unlocked",
            Self::UserCreated => "user_created",
            Self::UserUpdated => "user_updated",
            Self::UserDeleted => "user_deleted",
        }
    }
}
