use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Guest,
    User,
    Admin,
    SuperAdmin,
}

impl UserRole {
    pub fn has_permission(&self, required: &UserRole) -> bool {
        use UserRole::*;
        match (self, required) {
            (SuperAdmin, _) => true,
            (Admin, Guest | User | Admin) => true,
            (User, Guest | User) => true,
            (Guest, Guest) => true,
            _ => false,
        }
    }

    pub fn inherited_roles(&self) -> Vec<UserRole> {
        use UserRole::*;
        match self {
            SuperAdmin => vec![SuperAdmin, Admin, User, Guest],
            Admin => vec![Admin, User, Guest],
            User => vec![User, Guest],
            Guest => vec![Guest],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Disabled,
    Locked,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub failed_login_attempts: i32,
    pub totp_secret: Option<String>,
    pub two_factor_enabled: bool,
    pub account_id: Option<i32>,
}

impl User {
    pub fn can_authenticate(&self) -> bool {
        self.status == UserStatus::Active
    }

    pub fn has_role(&self, role: &UserRole) -> bool {
        self.role.has_permission(role)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::SuperAdmin)
    }

    pub fn verify_password(&self, password: &str) -> bool {
        crate::security::password_hasher::verify_password(password, &self.password_hash)
    }
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,

    #[validate(length(min = 1))]
    pub first_name: Option<String>,

    #[validate(length(min = 1))]
    pub last_name: Option<String>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct UpdateUserDto {
    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(min = 1))]
    pub first_name: Option<String>,

    #[validate(length(min = 1))]
    pub last_name: Option<String>,

    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
}

#[derive(Debug, Deserialize, validator::Validate)]
pub struct LoginCredentials {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1))]
    pub password: String,
}
