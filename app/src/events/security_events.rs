use appkit_core::event::Event;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginFailedEvent {
    pub email: String,
    pub ip: String,
    pub reason: String,
    pub timestamp: NaiveDateTime,
}

impl Event for LoginFailedEvent {
    fn name(&self) -> &'static str {
        "security.login.failed"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginSuccessEvent {
    pub user_id: i32,
    pub email: String,
    pub ip: String,
    pub used_2fa: bool,
    pub timestamp: NaiveDateTime,
}

impl Event for LoginSuccessEvent {
    fn name(&self) -> &'static str {
        "security.login.success"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PasswordChangedEvent {
    pub user_id: i32,
    pub email: String,
    pub ip: String,
    pub timestamp: NaiveDateTime,
}

impl Event for PasswordChangedEvent {
    fn name(&self) -> &'static str {
        "security.password.changed"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_failed_event_name() {
        let event = LoginFailedEvent {
            email: "test@example.com".to_string(),
            ip: "127.0.0.1".to_string(),
            reason: "Invalid password".to_string(),
            timestamp: chrono::Utc::now().naive_utc(),
        };
        assert_eq!(event.name(), "security.login.failed");
    }

    #[test]
    fn test_login_success_event_name() {
        let event = LoginSuccessEvent {
            user_id: 1,
            email: "test@example.com".to_string(),
            ip: "127.0.0.1".to_string(),
            used_2fa: true,
            timestamp: chrono::Utc::now().naive_utc(),
        };
        assert_eq!(event.name(), "security.login.success");
    }

    #[test]
    fn test_password_changed_event_name() {
        let event = PasswordChangedEvent {
            user_id: 1,
            email: "test@example.com".to_string(),
            ip: "127.0.0.1".to_string(),
            timestamp: chrono::Utc::now().naive_utc(),
        };
        assert_eq!(event.name(), "security.password.changed");
    }
}
