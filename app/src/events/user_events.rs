use appkit_core::event::Event;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserCreatedEvent {
    pub user_id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub account_id: Option<i32>,
}

impl Event for UserCreatedEvent {
    fn name(&self) -> &'static str {
        "user.created"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserUpdatedEvent {
    pub user_id: i32,
    pub email: String,
    pub changes: HashMap<String, String>,
}

impl Event for UserUpdatedEvent {
    fn name(&self) -> &'static str {
        "user.updated"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDeletedEvent {
    pub user_id: i32,
    pub email: String,
    pub deleted_at: NaiveDateTime,
}

impl Event for UserDeletedEvent {
    fn name(&self) -> &'static str {
        "user.deleted"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_created_event_name() {
        let event = UserCreatedEvent {
            user_id: 1,
            email: "test@example.com".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            account_id: None,
        };
        assert_eq!(event.name(), "user.created");
    }

    #[test]
    fn test_user_updated_event_name() {
        let event = UserUpdatedEvent {
            user_id: 1,
            email: "test@example.com".to_string(),
            changes: HashMap::new(),
        };
        assert_eq!(event.name(), "user.updated");
    }

    #[test]
    fn test_user_deleted_event_name() {
        let event = UserDeletedEvent {
            user_id: 1,
            email: "test@example.com".to_string(),
            deleted_at: chrono::Utc::now().naive_utc(),
        };
        assert_eq!(event.name(), "user.deleted");
    }
}
