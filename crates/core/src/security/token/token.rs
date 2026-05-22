use crate::security::user::User;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernamePasswordToken {
    pub user: User,
    pub firewall_name: String,
    pub roles: Vec<String>,
    pub authenticated: bool,
}

impl UsernamePasswordToken {
    pub fn new(user: User, firewall_name: String) -> Self {
        let roles = vec![format!("{:?}", user.role)];
        Self {
            user,
            firewall_name,
            roles,
            authenticated: true,
        }
    }

    pub fn get_user(&self) -> &User {
        &self.user
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    pub fn get_firewall_name(&self) -> &str {
        &self.firewall_name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoFactorToken {
    pub user: User,
    pub firewall_name: String,
    pub authenticated: bool,
}

impl TwoFactorToken {
    pub fn new(user: User, firewall_name: String) -> Self {
        Self {
            user,
            firewall_name,
            authenticated: false,
        }
    }

    pub fn get_user(&self) -> &User {
        &self.user
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Token {
    UsernamePassword(UsernamePasswordToken),
    TwoFactor(TwoFactorToken),
}

impl Token {
    pub fn get_user(&self) -> &User {
        match self {
            Token::UsernamePassword(t) => t.get_user(),
            Token::TwoFactor(t) => t.get_user(),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        match self {
            Token::UsernamePassword(t) => t.is_authenticated(),
            Token::TwoFactor(t) => t.is_authenticated(),
        }
    }

    pub fn firewall_name(&self) -> &str {
        match self {
            Token::UsernamePassword(t) => t.get_firewall_name(),
            Token::TwoFactor(t) => &t.firewall_name,
        }
    }
}
