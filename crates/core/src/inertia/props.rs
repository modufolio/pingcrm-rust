use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,

    pub account: AccountData,
    pub role: Option<String>,
    pub two_factor_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountData {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpersonationData {
    pub is_impersonating: bool,
    pub original_user: OriginalUserData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginalUserData {
    pub id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthData {
    pub user: Option<UserData>,
    pub impersonation: Option<ImpersonationData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashData {
    pub success: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedProps {
    pub auth: AuthData,
    pub flash: FlashData,
    pub errors: HashMap<String, String>,
}

impl SharedProps {
    pub fn new() -> Self {
        Self {
            auth: AuthData {
                user: None,
                impersonation: None,
            },
            flash: FlashData {
                success: None,
                error: None,
            },
            errors: HashMap::new(),
        }
    }

    pub fn with_user(mut self, user: UserData) -> Self {
        self.auth.user = Some(user);
        self
    }

    pub fn with_impersonation(mut self, impersonation: ImpersonationData) -> Self {
        self.auth.impersonation = Some(impersonation);
        self
    }

    pub fn with_success(mut self, message: String) -> Self {
        self.flash.success = Some(message);
        self
    }

    pub fn with_error(mut self, message: String) -> Self {
        self.flash.error = Some(message);
        self
    }

    pub fn with_field_error(mut self, field: String, message: String) -> Self {
        self.errors.insert(field, message);
        self
    }

    pub fn with_errors(mut self, errors: HashMap<String, String>) -> Self {
        self.errors = errors;
        self
    }

    pub fn to_value(&self) -> Value {
        json!(self)
    }

    pub fn merge_with(&self, mut props: Value) -> Value {
        if let Value::Object(ref mut map) = props {
            map.insert("auth".to_string(), json!(self.auth));
            map.insert("flash".to_string(), json!(self.flash));
            map.insert("errors".to_string(), json!(self.errors));
        }
        props
    }
}

impl Default for SharedProps {
    fn default() -> Self {
        Self::new()
    }
}
