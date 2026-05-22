use axum::http;

use axum::http::{Method, StatusCode};
use serde_json::{json, Value};

mod helpers;
use helpers::{DatabaseTest, TestFixtures};

#[tokio::test]
async fn test_health_endpoint() {
    let db = DatabaseTest::new().await;

    let response = db.server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_register_new_user() {
    let db = DatabaseTest::new().await;

    let response = db
        .server
        .post("/register")
        .json(&json!({
            "email": "newuser@example.com",
            "password": "SecurePass123!",
            "first_name": "New",
            "last_name": "User"
        }))
        .await;

    if response.status_code() != StatusCode::CREATED {
        let body_text = response.text();
        eprintln!(
            "Registration failed with status {}: {}",
            response.status_code(),
            body_text
        );
    }
    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: Value = response.json();
    assert_eq!(body["user"]["email"], "newuser@example.com");
    assert_eq!(body["user"]["first_name"], "New");
    assert_eq!(body["user"]["last_name"], "User");
    assert!(body["token"].is_string());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;

    let response = db
        .server
        .post("/register")
        .json(&json!({
            "email": "test@example.com",
            "password": "AnotherPass123!",
            "first_name": "Another",
            "last_name": "User"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_register_invalid_email() {
    let db = DatabaseTest::new().await;

    let response = db
        .server
        .post("/register")
        .json(&json!({
            "email": "not-an-email",
            "password": "SecurePass123!",
            "first_name": "Test",
            "last_name": "User"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_register_weak_password() {
    let db = DatabaseTest::new().await;

    let response = db
        .server
        .post("/register")
        .json(&json!({
            "email": "test@example.com",
            "password": "weak",
            "first_name": "Test",
            "last_name": "User"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    let body: Value = response.json();
    assert!(body["errors"].is_array());
}

#[tokio::test]
async fn test_login_valid_credentials() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;

    let response = db
        .server
        .post("/api/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "Test123!@#"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert!(body["data"]["token"].is_string());
    assert_eq!(body["data"]["user"]["email"], "test@example.com");
}

#[tokio::test]
async fn test_login_invalid_password() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;

    let response = db
        .server
        .post("/api/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "WrongPassword123!"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let db = DatabaseTest::new().await;

    let response = db
        .server
        .post("/api/login")
        .json(&json!({
            "email": "nonexistent@example.com",
            "password": "SomePassword123!"
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_without_token() {
    let db = DatabaseTest::new().await;

    let response = db.server.get("/api/profile").await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_with_invalid_token() {
    let db = DatabaseTest::new().await;

    let response = db
        .server
        .get("/api/profile")
        .add_header(
            http::header::AUTHORIZATION,
            "Bearer invalid-token".parse::<http::HeaderValue>().unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_with_valid_token() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .get("/api/profile")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["data"]["attributes"]["email"], "test@example.com");
}

#[tokio::test]
async fn test_admin_route_requires_admin_role() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .get("/admin/users")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_users_authenticated() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .get("/api/user")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert!(body["data"].is_array());
    assert!(body["meta"]["total"].is_number());
}

#[tokio::test]
async fn test_list_users_pagination() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .get("/api/user")
        .add_query_param("page[number]", "1")
        .add_query_param("page[size]", "10")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["meta"]["page"]["number"], 1);
    assert_eq!(body["meta"]["page"]["size"], 10);
}

#[tokio::test]
async fn test_get_user_by_id() {
    let db = DatabaseTest::new().await;

    let user = TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;
    let user_id = user["user"]["id"].as_str().unwrap();

    let response = db
        .server
        .get(&format!("/api/user/{}", user_id))
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    assert_eq!(body["data"]["id"], user_id);
    assert_eq!(body["data"]["type"], "users");
}

#[tokio::test]
async fn test_get_nonexistent_user() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .get("/api/user/99999999")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_jsonapi_content_type() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .get("/api/user")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok());

    assert!(content_type.is_some());
    assert!(content_type.unwrap().contains("application/vnd.api+json"));
}

#[tokio::test]
async fn test_jsonapi_structure() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .get("/api/user")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    let body: Value = response.json();

    assert!(body["data"].is_array());
    assert!(body["meta"].is_object());

    if let Some(first_user) = body["data"].as_array().and_then(|arr| arr.first()) {
        assert!(first_user["id"].is_string());
        assert_eq!(first_user["type"], "users");
        assert!(first_user["attributes"].is_object());
    }
}

#[tokio::test]
async fn test_jsonapi_error_format() {
    let db = DatabaseTest::new().await;

    let response = db.server.get("/api/profile").await;

    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    let body: Value = response.json();

    assert!(body["errors"].is_array());
    if let Some(first_error) = body["errors"].as_array().and_then(|arr| arr.first()) {
        assert!(first_error["status"].is_string());
        assert!(first_error["title"].is_string());
    }
}

#[tokio::test]
async fn test_security_headers_present() {
    let db = DatabaseTest::new().await;

    let response = db.server.get("/health").await;

    let headers = response.headers();

    assert!(headers.contains_key("x-content-type-options"));
    assert!(headers.contains_key("x-frame-options"));
    assert!(headers.contains_key("x-xss-protection"));
}

#[tokio::test]
async fn test_password_not_in_response() {
    let db = DatabaseTest::new().await;

    let response = db
        .server
        .post("/register")
        .json(&json!({
            "email": "test@example.com",
            "password": "Test123!@#",
            "first_name": "Test",
            "last_name": "User"
        }))
        .await;

    let body_text = response.text();

    assert!(!body_text.contains("Test123!@#"));
    assert!(!body_text.contains("password_hash"));
}

#[tokio::test]
async fn test_sql_injection_prevention() {
    let db = DatabaseTest::new().await;

    let response = db
        .server
        .post("/api/login")
        .json(&json!({
            "email": "admin@example.com' OR '1'='1",
            "password": "' OR '1'='1"
        }))
        .await;

    let status = response.status_code();
    assert!(
        status == StatusCode::UNAUTHORIZED
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 401, 400, or 422 for SQL injection attempt, got {}",
        status
    );
}

#[tokio::test]
async fn test_cors_headers() {
    let db = DatabaseTest::new().await;

    let response = db
        .server
        .method(Method::OPTIONS, "/api/user")
        .add_header(
            http::header::ORIGIN,
            "https://example.com".parse::<http::HeaderValue>().unwrap(),
        )
        .await;

    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn test_jsonapi_include_organization_in_contact() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let org = TestFixtures::create_organization(&db.server, &token, "Test Company").await;
    let org_id = org["data"]["id"].as_str().unwrap().parse::<i32>().unwrap();

    let contact =
        TestFixtures::create_contact(&db.server, &token, "John", "Doe", Some(org_id)).await;
    let contact_id = contact["data"]["id"].as_str().unwrap();

    let response_without_include = db
        .server
        .get(&format!("/api/v1/contacts/{}", contact_id))
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response_without_include.status_code(), StatusCode::OK);
    let body_without: Value = response_without_include.json();

    assert!(
        body_without["included"].is_null()
            || (body_without["included"].is_array()
                && body_without["included"].as_array().unwrap().is_empty()),
        "Should not have included section when not requested"
    );

    let response_with_include = db
        .server
        .get(&format!(
            "/api/v1/contacts/{}?include=organization",
            contact_id
        ))
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response_with_include.status_code(), StatusCode::OK);
    let body_with: Value = response_with_include.json();

    assert_eq!(body_with["data"]["type"], "contacts");
    assert_eq!(body_with["data"]["id"], contact_id);

    assert!(
        body_with["included"].is_array(),
        "Should have included array"
    );
    let included = body_with["included"].as_array().unwrap();
    assert!(!included.is_empty(), "Included array should not be empty");

    let org_resource = included
        .iter()
        .find(|r| r["type"] == "organizations" && r["id"] == org_id.to_string());

    assert!(
        org_resource.is_some(),
        "Organization should be in included resources"
    );
    let org_resource = org_resource.unwrap();

    assert_eq!(org_resource["attributes"]["name"], "Test Company");
    assert!(org_resource["attributes"]["email"].is_string());
}

#[tokio::test]
async fn test_jsonapi_include_multiple_relationships() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let org = TestFixtures::create_organization(&db.server, &token, "Multi Test Company").await;
    let org_id = org["data"]["id"].as_str().unwrap().parse::<i32>().unwrap();

    let contact =
        TestFixtures::create_contact(&db.server, &token, "Jane", "Smith", Some(org_id)).await;
    let contact_id = contact["data"]["id"].as_str().unwrap();

    let response = db
        .server
        .get(&format!(
            "/api/v1/contacts/{}?include=organization,account",
            contact_id
        ))
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();

    assert!(body["included"].is_array(), "Should have included array");
    let included = body["included"].as_array().unwrap();
    assert!(
        !included.is_empty(),
        "Included array should not be empty when relationships exist"
    );

    let has_org = included.iter().any(|r| r["type"] == "organizations");
    assert!(has_org, "Organization should be in included resources");
}

#[tokio::test]
async fn test_jsonapi_include_in_collection() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let org1 = TestFixtures::create_organization(&db.server, &token, "Company One").await;
    let org1_id = org1["data"]["id"].as_str().unwrap().parse::<i32>().unwrap();

    let org2 = TestFixtures::create_organization(&db.server, &token, "Company Two").await;
    let org2_id = org2["data"]["id"].as_str().unwrap().parse::<i32>().unwrap();

    TestFixtures::create_contact(&db.server, &token, "Alice", "Johnson", Some(org1_id)).await;
    TestFixtures::create_contact(&db.server, &token, "Bob", "Williams", Some(org2_id)).await;

    let response = db
        .server
        .get("/api/v1/contacts?include=organization")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();

    assert!(body["data"].is_array(), "Data should be an array");

    assert!(body["included"].is_array(), "Should have included array");
    let included = body["included"].as_array().unwrap();
    assert!(!included.is_empty(), "Included array should not be empty");

    let org_types = included
        .iter()
        .filter(|r| r["type"] == "organizations")
        .count();
    assert!(
        org_types >= 2,
        "Should have at least 2 organizations in included resources"
    );

    let has_org1 = included
        .iter()
        .any(|r| r["type"] == "organizations" && r["id"] == org1_id.to_string());
    let has_org2 = included
        .iter()
        .any(|r| r["type"] == "organizations" && r["id"] == org2_id.to_string());

    assert!(has_org1, "Company One should be in included resources");
    assert!(has_org2, "Company Two should be in included resources");
}

#[tokio::test]
async fn test_404_not_found() {
    let db = DatabaseTest::new().await;

    let response = db.server.get("/nonexistent-route").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_405_method_not_allowed() {
    let db = DatabaseTest::new().await;

    let response = db.server.post("/health").await;

    assert_eq!(response.status_code(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_malformed_json() {
    let db = DatabaseTest::new().await;

    let response = db.server.post("/api/login").text("{invalid json").await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_jsonapi_create_product() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .post("/api/v1/products")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .add_header(
            http::header::CONTENT_TYPE,
            "application/vnd.api+json"
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .json(&json!({
            "data": {
                "type": "products",
                "attributes": {
                    "name": "Test Product",
                    "slug": "test-product",
                    "description": "A test product for integration testing",
                    "price": 99.99,
                    "status": "active"
                }
            }
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: Value = response.json();
    assert_eq!(body["data"]["type"], "products");
    assert_eq!(body["data"]["attributes"]["name"], "Test Product");
    assert_eq!(body["data"]["attributes"]["price"], 99.99);
    assert!(body["data"]["id"].is_string());
}

#[tokio::test]
async fn test_jsonapi_update_product() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let create_response = db
        .server
        .post("/api/v1/products")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .add_header(
            http::header::CONTENT_TYPE,
            "application/vnd.api+json"
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .json(&json!({
            "data": {
                "type": "products",
                "attributes": {
                    "name": "Original Product",
                    "slug": "original-product",
                    "price": 50.00,
                    "status": "active"
                }
            }
        }))
        .await;

    let product_id = create_response.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let update_response = db
        .server
        .patch(&format!("/api/v1/products/{}", product_id))
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .add_header(
            http::header::CONTENT_TYPE,
            "application/vnd.api+json"
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .json(&json!({
            "data": {
                "type": "products",
                "id": product_id,
                "attributes": {
                    "name": "Updated Product",
                    "price": 75.00
                }
            }
        }))
        .await;

    assert_eq!(update_response.status_code(), StatusCode::OK);

    let body: Value = update_response.json();
    assert_eq!(body["data"]["attributes"]["name"], "Updated Product");
    assert_eq!(body["data"]["attributes"]["price"], 75.00);
}

#[tokio::test]
async fn test_jsonapi_delete_product() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let create_response = db
        .server
        .post("/api/v1/products")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .add_header(
            http::header::CONTENT_TYPE,
            "application/vnd.api+json"
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .json(&json!({
            "data": {
                "type": "products",
                "attributes": {
                    "name": "Product to Delete",
                    "slug": "product-to-delete",
                    "price": 25.00,
                    "status": "active"
                }
            }
        }))
        .await;

    let product_id = create_response.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let delete_response = db
        .server
        .delete(&format!("/api/v1/products/{}", product_id))
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(delete_response.status_code(), StatusCode::NO_CONTENT);

    let get_response = db
        .server
        .get(&format!("/api/v1/products/{}", product_id))
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(get_response.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_jsonapi_filter_products() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    for i in 0..3 {
        db.server
            .post("/api/v1/products")
            .add_header(
                http::header::AUTHORIZATION,
                format!("Bearer {}", token)
                    .parse::<http::HeaderValue>()
                    .unwrap(),
            )
            .add_header(
                http::header::CONTENT_TYPE,
                "application/vnd.api+json"
                    .parse::<http::HeaderValue>()
                    .unwrap(),
            )
            .json(&json!({
                "data": {
                    "type": "products",
                    "attributes": {
                        "name": format!("Active Product {}", i),
                        "slug": format!("active-product-{}", i),
                        "price": 10.00 + (i as f64),
                        "stock_status": "in_stock"
                    }
                }
            }))
            .await;
    }

    db.server
        .post("/api/v1/products")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .add_header(
            http::header::CONTENT_TYPE,
            "application/vnd.api+json"
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .json(&json!({
            "data": {
                "type": "products",
                "attributes": {
                    "name": "Inactive Product",
                    "slug": "inactive-product",
                    "price": 20.00,
                    "stock_status": "out_of_stock"
                }
            }
        }))
        .await;

    let response = db
        .server
        .get("/api/v1/products?filter[stock_status][eq]=in_stock")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    let products = body["data"].as_array().unwrap();

    assert!(products.len() >= 3);
    for product in products {
        if product["attributes"]["name"]
            .as_str()
            .unwrap()
            .contains("Product")
        {
            assert!(product["attributes"]["name"]
                .as_str()
                .unwrap()
                .contains("Active"));
        }
    }
}

#[tokio::test]
async fn test_jsonapi_sort_products() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let prices = vec![50.00, 10.00, 30.00, 20.00];
    for (i, price) in prices.iter().enumerate() {
        db.server
            .post("/api/v1/products")
            .add_header(
                http::header::AUTHORIZATION,
                format!("Bearer {}", token)
                    .parse::<http::HeaderValue>()
                    .unwrap(),
            )
            .add_header(
                http::header::CONTENT_TYPE,
                "application/vnd.api+json"
                    .parse::<http::HeaderValue>()
                    .unwrap(),
            )
            .json(&json!({
                "data": {
                    "type": "products",
                    "attributes": {
                        "name": format!("Product {}", i),
                        "slug": format!("product-{}", i),
                        "price": price,
                        "status": "active"
                    }
                }
            }))
            .await;
    }

    let response = db
        .server
        .get("/api/v1/products?sort=price")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    let products = body["data"].as_array().unwrap();

    if products.len() >= 2 {
        let first_price = products[0]["attributes"]["price"].as_f64().unwrap();
        let second_price = products[1]["attributes"]["price"].as_f64().unwrap();
        assert!(
            first_price <= second_price,
            "Products should be sorted by price ascending"
        );
    }
}

#[tokio::test]
async fn test_jsonapi_pagination_products() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    for i in 0..15 {
        db.server
            .post("/api/v1/products")
            .add_header(
                http::header::AUTHORIZATION,
                format!("Bearer {}", token)
                    .parse::<http::HeaderValue>()
                    .unwrap(),
            )
            .add_header(
                http::header::CONTENT_TYPE,
                "application/vnd.api+json"
                    .parse::<http::HeaderValue>()
                    .unwrap(),
            )
            .json(&json!({
                "data": {
                    "type": "products",
                    "attributes": {
                        "name": format!("Paginated Product {}", i),
                        "slug": format!("paginated-product-{}", i),
                        "price": 10.00,
                        "status": "active"
                    }
                }
            }))
            .await;
    }

    let response = db
        .server
        .get("/api/v1/products?page[number]=1&page[size]=5")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();

    assert!(body["meta"].is_object());
    assert!(body["meta"]["total"].is_number());
    assert_eq!(body["meta"]["page"]["number"], 1);
    assert_eq!(body["meta"]["page"]["size"], 5);

    assert!(body["links"].is_object());
    assert!(body["links"]["first"].is_string());
    assert!(body["links"]["last"].is_string());
    assert!(body["links"]["self"].is_string());
}

#[tokio::test]
async fn test_jsonapi_combined_filters_sorts_pagination() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    for i in 0..20 {
        let status = if i % 2 == 0 { "active" } else { "inactive" };
        db.server
            .post("/api/v1/products")
            .add_header(
                http::header::AUTHORIZATION,
                format!("Bearer {}", token)
                    .parse::<http::HeaderValue>()
                    .unwrap(),
            )
            .add_header(
                http::header::CONTENT_TYPE,
                "application/vnd.api+json"
                    .parse::<http::HeaderValue>()
                    .unwrap(),
            )
            .json(&json!({
                "data": {
                    "type": "products",
                    "attributes": {
                        "name": format!("Combined Test Product {}", i),
                        "slug": format!("combined-test-product-{}", i),
                        "price": 10.00 + (i as f64),
                        "status": status
                    }
                }
            }))
            .await;
    }

    let response = db
        .server
        .get("/api/v1/products?filter[status][eq]=active&sort=-price&page[number]=1&page[size]=5")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    let products = body["data"].as_array().unwrap();

    assert!(products.len() <= 5);
    assert_eq!(body["meta"]["page"]["size"], 5);
}

#[tokio::test]
async fn test_jsonapi_sparse_fieldsets() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let create_response = db
        .server
        .post("/api/v1/products")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .add_header(
            http::header::CONTENT_TYPE,
            "application/vnd.api+json"
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .json(&json!({
            "data": {
                "type": "products",
                "attributes": {
                    "name": "Sparse Fields Product",
                    "slug": "sparse-fields-product",
                    "description": "This description should not appear",
                    "price": 99.99,
                    "status": "active"
                }
            }
        }))
        .await;

    let product_id = create_response.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let response = db
        .server
        .get(&format!(
            "/api/v1/products/{}?fields[products]=name,price",
            product_id
        ))
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let body: Value = response.json();
    let attributes = &body["data"]["attributes"];

    assert!(attributes["name"].is_string());
    assert!(attributes["price"].is_number());

    assert!(
        attributes["description"].is_null()
            || !attributes.as_object().unwrap().contains_key("description")
    );
}

#[tokio::test]
async fn test_jsonapi_invalid_filter_field() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .get("/api/v1/products?filter[nonexistent_field][eq]=value")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    let body: Value = response.json();
    assert!(body["errors"].is_array());

    let error_text = body.to_string().to_lowercase();
    assert!(
        error_text.contains("filter")
            || error_text.contains("invalid")
            || error_text.contains("field")
    );
}

#[tokio::test]
async fn test_jsonapi_invalid_sort_field() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .get("/api/v1/products?sort=nonexistent_field")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    let body: Value = response.json();
    assert!(body["errors"].is_array());
}

#[tokio::test]
async fn test_jsonapi_invalid_include_relationship() {
    let db = DatabaseTest::new().await;

    TestFixtures::create_default_user(&db.server).await;
    let token = TestFixtures::login(&db.server, "test@example.com", "Test123!@#").await;

    let response = db
        .server
        .get("/api/v1/products?include=nonexistent_relationship")
        .add_header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", token)
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);

    let body: Value = response.json();
    assert!(body["errors"].is_array());
}
