use appkit_core::security::access_control::{
    access_control_middleware, AccessControlConfig, AccessControlRule,
};
use appkit_core::security::user::{User, UserRole, UserStatus};
use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    middleware,
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Utc;
use std::sync::Arc;
use tower::ServiceExt;

fn create_test_user(id: i32, role: UserRole) -> User {
    User {
        id,
        email: format!("user{}@example.com", id),
        password_hash: "hash".to_string(),
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

async fn mock_handler() -> impl IntoResponse {
    (StatusCode::OK, "Success")
}

fn create_request(uri: &str, user: Option<User>) -> Request {
    let mut request = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    if let Some(u) = user {
        request.extensions_mut().insert(u);
    }

    request
}

#[tokio::test]
async fn test_no_rule_allows_access() {
    let config = Arc::new(AccessControlConfig::new());

    let app = Router::new()
        .route("/public", get(mock_handler))
        .layer(middleware::from_fn(move |req, next| {
            let cfg = config.clone();
            access_control_middleware(cfg, req, next)
        }));

    let request = create_request("/public", None);
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_rule_with_no_roles_allows_unauthenticated() {
    let config =
        Arc::new(AccessControlConfig::new().add_rule(AccessControlRule::new("/api/public")));

    let app = Router::new()
        .route("/api/public/data", get(mock_handler))
        .layer(middleware::from_fn(move |req, next| {
            let cfg = config.clone();
            access_control_middleware(cfg, req, next)
        }));

    let request = create_request("/api/public/data", None);
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_authenticated_user_with_correct_role() {
    let config = Arc::new(
        AccessControlConfig::new()
            .add_rule(AccessControlRule::new("/admin").with_roles(vec![UserRole::Admin])),
    );

    let app = Router::new()
        .route("/admin/users", get(mock_handler))
        .layer(middleware::from_fn(move |req, next| {
            let cfg = config.clone();
            access_control_middleware(cfg, req, next)
        }));

    let admin = create_test_user(1, UserRole::Admin);
    let request = create_request("/admin/users", Some(admin));
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_authenticated_user_with_insufficient_role() {
    let config = Arc::new(
        AccessControlConfig::new()
            .add_rule(AccessControlRule::new("/admin").with_roles(vec![UserRole::Admin])),
    );

    let app = Router::new()
        .route("/admin/users", get(mock_handler))
        .layer(middleware::from_fn(move |req, next| {
            let cfg = config.clone();
            access_control_middleware(cfg, req, next)
        }));

    let user = create_test_user(2, UserRole::User);
    let request = create_request("/admin/users", Some(user));
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_no_user_authenticated_but_roles_required() {
    let config = Arc::new(
        AccessControlConfig::new()
            .add_rule(AccessControlRule::new("/api").with_roles(vec![UserRole::User])),
    );

    let app = Router::new()
        .route("/api/data", get(mock_handler))
        .layer(middleware::from_fn(move |req, next| {
            let cfg = config.clone();
            access_control_middleware(cfg, req, next)
        }));

    let request = create_request("/api/data", None);
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_role_hierarchy_super_admin_can_access_admin() {
    let config = Arc::new(
        AccessControlConfig::new()
            .add_rule(AccessControlRule::new("/admin").with_roles(vec![UserRole::Admin])),
    );

    let app = Router::new()
        .route("/admin/settings", get(mock_handler))
        .layer(middleware::from_fn(move |req, next| {
            let cfg = config.clone();
            access_control_middleware(cfg, req, next)
        }));

    let super_admin = create_test_user(3, UserRole::SuperAdmin);
    let request = create_request("/admin/settings", Some(super_admin));
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_first_rule_match_wins() {
    let config = Arc::new(
        AccessControlConfig::new()
            .add_rule(AccessControlRule::new("/api/public"))
            .add_rule(AccessControlRule::new("/api").with_roles(vec![UserRole::User])),
    );

    let app = Router::new()
        .route("/api/public/data", get(mock_handler))
        .layer(middleware::from_fn(move |req, next| {
            let cfg = config.clone();
            access_control_middleware(cfg, req, next)
        }));

    let request = create_request("/api/public/data", None);
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_multiple_required_roles_any_match_succeeds() {
    let config = Arc::new(AccessControlConfig::new().add_rule(
        AccessControlRule::new("/api").with_roles(vec![UserRole::User, UserRole::Admin]),
    ));

    let app1 = Router::new()
        .route("/api/data", get(mock_handler))
        .layer(middleware::from_fn(move |req, next| {
            let cfg = config.clone();
            access_control_middleware(cfg, req, next)
        }));

    let user = create_test_user(4, UserRole::User);
    let request = create_request("/api/data", Some(user));
    let response = app1.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let admin = create_test_user(5, UserRole::Admin);
    let request = create_request("/api/data", Some(admin));
    let response = app1.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_default_pingcrm_rules() {
    let config = Arc::new(AccessControlConfig::default_rules());

    let app = Router::new()
        .route("/admin/dashboard", get(mock_handler))
        .route("/api/users", get(mock_handler))
        .layer(middleware::from_fn(move |req, next| {
            let cfg = config.clone();
            access_control_middleware(cfg, req, next)
        }));

    let user = create_test_user(6, UserRole::User);
    let request = create_request("/admin/dashboard", Some(user));
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let user = create_test_user(7, UserRole::User);
    let request = create_request("/api/users", Some(user));
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
