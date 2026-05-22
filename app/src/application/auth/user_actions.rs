use crate::database::{
    DbPool as DieselPool, DieselUserRepository, NewUser, UserRepository, UserUpdate,
};
use appkit_core::error::{AppError, AppResult};
use appkit_core::security::authenticator::JwtAuthenticator;
use appkit_core::security::user::{
    CreateUserDto, LoginCredentials, UpdateUserDto, User as SecurityUser, UserRole, UserStatus,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub errors: Vec<String>,
}

impl<T> ActionResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            errors: Vec::new(),
        }
    }

    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message.into()),
            errors: Vec::new(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.into()),
            errors: Vec::new(),
        }
    }

    pub fn errors(errors: Vec<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: None,
            errors,
        }
    }

    pub fn is_success(&self) -> bool {
        self.success
    }
}

fn role_to_string(role: UserRole) -> String {
    match role {
        UserRole::SuperAdmin => "SuperAdmin".to_string(),
        UserRole::Admin => "Admin".to_string(),
        UserRole::User => "User".to_string(),
        UserRole::Guest => "Guest".to_string(),
    }
}

fn status_to_string(status: UserStatus) -> String {
    match status {
        UserStatus::Active => "active".to_string(),
        UserStatus::Disabled => "disabled".to_string(),
        UserStatus::Locked => "locked".to_string(),
        UserStatus::Expired => "expired".to_string(),
    }
}

pub struct LoginAction {
    repository: UserRepository,
    jwt_auth: JwtAuthenticator<DieselUserRepository>,
}

#[derive(Debug, Serialize)]
pub struct LoginResult {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
}

impl From<&SecurityUser> for UserInfo {
    fn from(user: &SecurityUser) -> Self {
        Self {
            id: user.id,
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            role: user.role.clone(),
        }
    }
}

impl LoginAction {
    pub fn new(pool: DieselPool, jwt_secret: String) -> Self {
        let repository = UserRepository::new(pool.clone());
        let diesel_user_repo = DieselUserRepository::new(pool);
        Self {
            repository,
            jwt_auth: JwtAuthenticator::new(jwt_secret, diesel_user_repo),
        }
    }

    pub async fn execute(
        &self,
        credentials: LoginCredentials,
    ) -> AppResult<ActionResult<LoginResult>> {
        credentials
            .validate()
            .map_err(|e| AppError::ValidationFailed(e.to_string()))?;

        let db_user = self
            .repository
            .find_by_email(&credentials.email)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::AuthenticationFailed("Invalid credentials".to_string()))?;

        let user = db_user.to_security_user();

        if !user.can_authenticate() {
            return Err(AppError::AuthenticationFailed(
                "Account is not active".to_string(),
            ));
        }

        let password_valid = user.verify_password(&credentials.password);

        if !password_valid {
            return Err(AppError::AuthenticationFailed(
                "Invalid credentials".to_string(),
            ));
        }

        let token = self.jwt_auth.generate_token(&user)?;

        Ok(ActionResult::success(LoginResult {
            token,
            user: UserInfo::from(&user),
        }))
    }
}

pub struct CreateUserAction {
    repository: UserRepository,
}

impl CreateUserAction {
    pub fn new(pool: DieselPool) -> Self {
        Self {
            repository: UserRepository::new(pool),
        }
    }

    pub async fn execute(&self, dto: CreateUserDto) -> AppResult<ActionResult<SecurityUser>> {
        dto.validate()
            .map_err(|e| AppError::ValidationFailed(e.to_string()))?;

        if let Some(_) = self
            .repository
            .find_by_email(&dto.email)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }

        let password = dto.password.clone();
        let password_hash =
            tokio::task::spawn_blocking(move || appkit_core::security::hash_password(&password))
                .await
                .map_err(|e| AppError::InternalServerError(format!("Task join error: {}", e)))?
                .map_err(|e| {
                    AppError::InternalServerError(format!("Password hash failed: {}", e))
                })?;

        let new_user = NewUser::new(
            dto.email.clone(),
            password_hash,
            dto.first_name.unwrap_or_else(|| String::from("")),
            dto.last_name.unwrap_or_else(|| String::from("")),
        )
        .with_roles(vec!["ROLE_USER".to_string()]);

        let db_user = self
            .repository
            .create(new_user)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

        let user = db_user.to_security_user();

        Ok(ActionResult::success_with_message(
            user,
            "User created successfully",
        ))
    }
}

pub struct UpdateUserAction {
    repository: UserRepository,
}

impl UpdateUserAction {
    pub fn new(pool: DieselPool) -> Self {
        Self {
            repository: UserRepository::new(pool),
        }
    }

    pub async fn execute(
        &self,
        user_id: i32,
        dto: UpdateUserDto,
        current_user: &SecurityUser,
    ) -> AppResult<ActionResult<SecurityUser>> {
        dto.validate()
            .map_err(|e| AppError::ValidationFailed(e.to_string()))?;

        if current_user.id != user_id && !current_user.is_admin() {
            return Err(AppError::AuthorizationFailed(
                "Cannot update other users".to_string(),
            ));
        }

        let existing = self
            .repository
            .find(user_id)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if let Some(ref email) = dto.email {
            if email != &existing.email {
                if let Some(_) =
                    self.repository.find_by_email(email).await.map_err(|e| {
                        AppError::InternalServerError(format!("Database error: {}", e))
                    })?
                {
                    return Err(AppError::Conflict("Email already exists".to_string()));
                }
            }
        }

        let mut user_update = UserUpdate::new();
        user_update.email = dto.email;
        user_update.first_name = dto.first_name;
        user_update.last_name = dto.last_name;

        if current_user.is_admin() {
            user_update.roles = dto
                .role
                .map(|r| serde_json::to_string(&vec![role_to_string(r)]).unwrap_or_default());
            user_update.account_status = dto.status.map(status_to_string);
        }

        let db_user = self
            .repository
            .update(user_id, user_update)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

        let user = db_user.to_security_user();

        Ok(ActionResult::success_with_message(
            user,
            "User updated successfully",
        ))
    }
}
