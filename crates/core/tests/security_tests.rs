use appkit_core::security::authenticator::JwtClaims;

use appkit_core::security::token::{Token, TwoFactorToken, UsernamePasswordToken};
use appkit_core::security::user::{User, UserRole, UserStatus};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

fn create_test_user(id: i32, email: &str, role: UserRole) -> User {
    User {
        id,
        email: email.to_string(),
        password_hash: "$2b$12$abcdefghijklmnopqrstuv".to_string(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        role,
        status: UserStatus::Active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: None,
        failed_login_attempts: 0,
        totp_secret: None,
        two_factor_enabled: false,
        account_id: None,
    }
}

#[test]
fn test_jwt_claims_creation() {
    let claims = JwtClaims {
        sub: 123,
        email: "test@example.com".to_string(),
        role: UserRole::User,
        exp: (Utc::now() + Duration::hours(24)).timestamp(),
        iat: Utc::now().timestamp(),
    };

    assert_eq!(claims.sub, 123);
    assert_eq!(claims.email, "test@example.com");
    assert_eq!(claims.role, UserRole::User);
}

#[test]
fn test_jwt_encode_decode() {
    let secret = "test_secret_key_at_least_32_chars_long_for_security";

    let claims = JwtClaims {
        sub: 456,
        email: "user@example.com".to_string(),
        role: UserRole::Admin,
        exp: (Utc::now() + Duration::hours(24)).timestamp(),
        iat: Utc::now().timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();

    let decoded = decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .unwrap();

    assert_eq!(decoded.claims.sub, 456);
    assert_eq!(decoded.claims.email, "user@example.com");
    assert_eq!(decoded.claims.role, UserRole::Admin);
}

#[test]
fn test_jwt_expired_token() {
    let secret = "test_secret_key_at_least_32_chars_long";

    let claims = JwtClaims {
        sub: 789,
        email: "expired@example.com".to_string(),
        role: UserRole::User,
        exp: (Utc::now() - Duration::hours(1)).timestamp(),
        iat: (Utc::now() - Duration::hours(25)).timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();

    let result = decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    );

    assert!(result.is_err());
}

#[test]
fn test_jwt_wrong_secret() {
    let secret1 = "secret_key_one_at_least_32_chars_long_abc";
    let secret2 = "secret_key_two_at_least_32_chars_long_xyz";

    let claims = JwtClaims {
        sub: 111,
        email: "test@example.com".to_string(),
        role: UserRole::User,
        exp: (Utc::now() + Duration::hours(24)).timestamp(),
        iat: Utc::now().timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret1.as_bytes()),
    )
    .unwrap();

    let result = decode::<JwtClaims>(
        &token,
        &DecodingKey::from_secret(secret2.as_bytes()),
        &Validation::default(),
    );

    assert!(result.is_err());
}

#[test]
fn test_jwt_claims_with_different_roles() {
    let roles = vec![
        UserRole::User,
        UserRole::Admin,
        UserRole::Guest,
        UserRole::SuperAdmin,
    ];

    for role in roles {
        let claims = JwtClaims {
            sub: 999,
            email: "role_test@example.com".to_string(),
            role: role.clone(),
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            iat: Utc::now().timestamp(),
        };

        assert_eq!(claims.role, role);
    }
}

#[test]
fn test_username_password_token_creation() {
    let user = create_test_user(1, "test@example.com", UserRole::User);

    let token = UsernamePasswordToken::new(user.clone(), "main".to_string());

    assert_eq!(token.get_user().id, 1);
    assert_eq!(token.get_firewall_name(), "main");
    assert!(token.is_authenticated());
    assert_eq!(token.roles.len(), 1);
}

#[test]
fn test_two_factor_token_creation() {
    let mut user = create_test_user(2, "2fa@example.com", UserRole::Admin);
    user.totp_secret = Some("secret".to_string());
    user.two_factor_enabled = true;

    let token = TwoFactorToken::new(user.clone(), "secure".to_string());

    assert_eq!(token.get_user().id, 2);
    assert!(!token.is_authenticated());
}

#[test]
fn test_token_enum_username_password() {
    let user = create_test_user(3, "enum@example.com", UserRole::User);

    let token =
        Token::UsernamePassword(UsernamePasswordToken::new(user.clone(), "main".to_string()));

    assert_eq!(token.get_user().id, 3);
    assert!(token.is_authenticated());
}

#[test]
fn test_token_enum_two_factor() {
    let mut user = create_test_user(4, "2fa_enum@example.com", UserRole::Admin);
    user.totp_secret = Some("secret".to_string());
    user.two_factor_enabled = true;

    let token = Token::TwoFactor(TwoFactorToken::new(user.clone(), "secure".to_string()));

    assert_eq!(token.get_user().id, 4);
    assert!(!token.is_authenticated());
}

#[test]
fn test_user_creation() {
    let user = create_test_user(100, "new@example.com", UserRole::User);

    assert_eq!(user.id, 100);
    assert_eq!(user.email, "new@example.com");
    assert_eq!(user.status, UserStatus::Active);
    assert!(user.totp_secret.is_none());
}

#[test]
fn test_user_with_2fa() {
    let mut user = create_test_user(101, "2fa_user@example.com", UserRole::Admin);
    user.totp_secret = Some("JBSWY3DPEHPK3PXP".to_string());
    user.two_factor_enabled = true;

    assert!(user.totp_secret.is_some());
    assert!(user.two_factor_enabled);
}

#[test]
fn test_user_can_authenticate() {
    let active_user = create_test_user(102, "active@example.com", UserRole::User);
    assert!(active_user.can_authenticate());

    let mut disabled_user = create_test_user(103, "disabled@example.com", UserRole::User);
    disabled_user.status = UserStatus::Disabled;
    assert!(!disabled_user.can_authenticate());
}

#[test]
fn test_user_statuses() {
    let mut user = create_test_user(104, "test@example.com", UserRole::User);

    user.status = UserStatus::Active;
    assert_eq!(user.status, UserStatus::Active);
    assert!(user.can_authenticate());

    user.status = UserStatus::Disabled;
    assert_eq!(user.status, UserStatus::Disabled);
    assert!(!user.can_authenticate());

    user.status = UserStatus::Locked;
    assert_eq!(user.status, UserStatus::Locked);
    assert!(!user.can_authenticate());

    user.status = UserStatus::Expired;
    assert_eq!(user.status, UserStatus::Expired);
    assert!(!user.can_authenticate());
}

#[test]
fn test_user_roles() {
    let admin = create_test_user(105, "admin@example.com", UserRole::Admin);
    let guest = create_test_user(106, "guest@example.com", UserRole::Guest);

    assert_eq!(admin.role, UserRole::Admin);
    assert_eq!(guest.role, UserRole::Guest);
}

#[test]
fn test_user_is_admin() {
    let admin = create_test_user(107, "admin@example.com", UserRole::Admin);
    let super_admin = create_test_user(108, "super@example.com", UserRole::SuperAdmin);
    let user = create_test_user(109, "user@example.com", UserRole::User);

    assert!(admin.is_admin());
    assert!(super_admin.is_admin());
    assert!(!user.is_admin());
}

#[test]
fn test_user_has_role() {
    let super_admin = create_test_user(110, "super@example.com", UserRole::SuperAdmin);
    let admin = create_test_user(111, "admin@example.com", UserRole::Admin);
    let user = create_test_user(112, "user@example.com", UserRole::User);

    assert!(super_admin.has_role(&UserRole::SuperAdmin));
    assert!(super_admin.has_role(&UserRole::Admin));
    assert!(super_admin.has_role(&UserRole::User));
    assert!(super_admin.has_role(&UserRole::Guest));

    assert!(!admin.has_role(&UserRole::SuperAdmin));
    assert!(admin.has_role(&UserRole::Admin));
    assert!(admin.has_role(&UserRole::User));
    assert!(admin.has_role(&UserRole::Guest));

    assert!(!user.has_role(&UserRole::SuperAdmin));
    assert!(!user.has_role(&UserRole::Admin));
    assert!(user.has_role(&UserRole::User));
    assert!(user.has_role(&UserRole::Guest));
}

#[test]
fn test_role_hierarchy_permissions() {
    assert!(UserRole::SuperAdmin.has_permission(&UserRole::SuperAdmin));
    assert!(UserRole::SuperAdmin.has_permission(&UserRole::Admin));
    assert!(UserRole::SuperAdmin.has_permission(&UserRole::User));
    assert!(UserRole::SuperAdmin.has_permission(&UserRole::Guest));

    assert!(!UserRole::Admin.has_permission(&UserRole::SuperAdmin));
    assert!(UserRole::Admin.has_permission(&UserRole::Admin));
    assert!(UserRole::Admin.has_permission(&UserRole::User));
    assert!(UserRole::Admin.has_permission(&UserRole::Guest));

    assert!(!UserRole::User.has_permission(&UserRole::SuperAdmin));
    assert!(!UserRole::User.has_permission(&UserRole::Admin));
    assert!(UserRole::User.has_permission(&UserRole::User));
    assert!(UserRole::User.has_permission(&UserRole::Guest));

    assert!(!UserRole::Guest.has_permission(&UserRole::SuperAdmin));
    assert!(!UserRole::Guest.has_permission(&UserRole::Admin));
    assert!(!UserRole::Guest.has_permission(&UserRole::User));
    assert!(UserRole::Guest.has_permission(&UserRole::Guest));
}

#[test]
fn test_role_inherited_roles() {
    let super_admin_roles = UserRole::SuperAdmin.inherited_roles();
    assert_eq!(super_admin_roles.len(), 4);
    assert!(super_admin_roles.contains(&UserRole::SuperAdmin));
    assert!(super_admin_roles.contains(&UserRole::Admin));
    assert!(super_admin_roles.contains(&UserRole::User));
    assert!(super_admin_roles.contains(&UserRole::Guest));

    let admin_roles = UserRole::Admin.inherited_roles();
    assert_eq!(admin_roles.len(), 3);
    assert!(!admin_roles.contains(&UserRole::SuperAdmin));
    assert!(admin_roles.contains(&UserRole::Admin));
    assert!(admin_roles.contains(&UserRole::User));
    assert!(admin_roles.contains(&UserRole::Guest));

    let user_roles = UserRole::User.inherited_roles();
    assert_eq!(user_roles.len(), 2);
    assert!(user_roles.contains(&UserRole::User));
    assert!(user_roles.contains(&UserRole::Guest));

    let guest_roles = UserRole::Guest.inherited_roles();
    assert_eq!(guest_roles.len(), 1);
    assert!(guest_roles.contains(&UserRole::Guest));
}

#[test]
fn test_user_failed_login_attempts() {
    let mut user = create_test_user(113, "lockout@example.com", UserRole::User);

    assert_eq!(user.failed_login_attempts, 0);

    user.failed_login_attempts = 3;
    assert_eq!(user.failed_login_attempts, 3);

    user.failed_login_attempts = 0;
    assert_eq!(user.failed_login_attempts, 0);
}

#[test]
fn test_user_last_login_tracking() {
    let mut user = create_test_user(114, "login_tracking@example.com", UserRole::User);

    assert!(user.last_login_at.is_none());

    user.last_login_at = Some(Utc::now());
    assert!(user.last_login_at.is_some());
}
