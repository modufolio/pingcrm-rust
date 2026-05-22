use crate::presenter::Presenter;
use appkit_core::security::user::{User, UserRole, UserStatus};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct UserPresenter {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl Presenter<User, UserPresenter> for UserPresenter {
    fn present(&self, _entity: &User) -> UserPresenter {
        self.clone()
    }
}

impl From<&User> for UserPresenter {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            role: user.role.clone(),
            status: user.status.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login_at: user.last_login_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UserListPresenter {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
}

impl From<&User> for UserListPresenter {
    fn from(user: &User) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            role: user.role.clone(),
            status: user.status.clone(),
            created_at: user.created_at,
        }
    }
}

impl Presenter<User, UserListPresenter> for UserListPresenter {
    fn present(&self, _entity: &User) -> UserListPresenter {
        self.clone()
    }
}

pub fn present_users(users: &[User]) -> Vec<UserPresenter> {
    users.iter().map(UserPresenter::from).collect()
}

pub fn present_users_list(users: &[User]) -> Vec<UserListPresenter> {
    users.iter().map(UserListPresenter::from).collect()
}
