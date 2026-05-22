use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{json, Value};

pub struct TestFixtures;

impl TestFixtures {
    pub async fn create_user(
        server: &TestServer,
        email: &str,
        password: &str,
        first_name: &str,
        last_name: &str,
    ) -> Value {
        let response = server
            .post("/register")
            .json(&json!({
                "email": email,
                "password": password,
                "first_name": first_name,
                "last_name": last_name
            }))
            .await;

        assert_eq!(
            response.status_code(),
            StatusCode::CREATED,
            "Failed to create test user: {}",
            response.text()
        );

        response.json()
    }

    pub async fn create_default_user(server: &TestServer) -> Value {
        Self::create_user(server, "test@example.com", "Test123!@#", "Test", "User").await
    }

    pub async fn login(server: &TestServer, email: &str, password: &str) -> String {
        let response = server
            .post("/api/login")
            .json(&json!({
                "email": email,
                "password": password
            }))
            .await;

        assert_eq!(
            response.status_code(),
            StatusCode::OK,
            "Login failed for {}: {}",
            email,
            response.text()
        );

        let body: Value = response.json();
        body["data"]["token"].as_str().unwrap().to_string()
    }

    pub async fn create_users(server: &TestServer, count: usize) -> Vec<Value> {
        let mut users = Vec::new();

        for i in 0..count {
            let user = Self::create_user(
                server,
                &format!("user{}@example.com", i),
                "Test123!@#",
                &format!("User{}", i),
                "Test",
            )
            .await;
            users.push(user);
        }

        users
    }

    #[allow(dead_code)]
    pub async fn create_organization(server: &TestServer, token: &str, name: &str) -> Value {
        let response = server
            .post("/api/v1/organizations")
            .add_header(
                "Authorization".parse::<axum::http::HeaderName>().unwrap(),
                format!("Bearer {}", token)
                    .parse::<axum::http::HeaderValue>()
                    .unwrap(),
            )
            .json(&json!({
                "data": {
                    "type": "organizations",
                    "attributes": {
                        "name": name,
                        "email": format!("{}@example.com", name.to_lowercase().replace(" ", "")),
                        "phone": "+1234567890"
                    }
                }
            }))
            .await;

        response.json()
    }

    #[allow(dead_code)]
    pub async fn create_contact(
        server: &TestServer,
        token: &str,
        first_name: &str,
        last_name: &str,
        organization_id: Option<i32>,
    ) -> Value {
        let attributes = json!({
            "first_name": first_name,
            "last_name": last_name,
            "email": format!("{}.{}@example.com", first_name.to_lowercase(), last_name.to_lowercase()),
        });

        let mut payload = json!({
            "data": {
                "type": "contacts",
                "attributes": attributes
            }
        });

        if let Some(org_id) = organization_id {
            payload["data"]["relationships"] = json!({
                "organization": {
                    "data": {
                        "type": "organizations",
                        "id": org_id.to_string()
                    }
                }
            });
        }

        let response = server
            .post("/api/v1/contacts")
            .add_header(
                "Authorization".parse::<axum::http::HeaderName>().unwrap(),
                format!("Bearer {}", token)
                    .parse::<axum::http::HeaderValue>()
                    .unwrap(),
            )
            .json(&payload)
            .await;

        response.json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::DatabaseTest;

    #[tokio::test]
    async fn test_create_default_user() {
        let db = DatabaseTest::new().await;
        let user = TestFixtures::create_default_user(&db.server).await;

        assert_eq!(user["user"]["email"], "test@example.com");
        assert!(user["token"].is_string());
    }

    #[tokio::test]
    async fn test_create_multiple_users() {
        let db = DatabaseTest::new().await;
        let users = TestFixtures::create_users(&db.server, 3).await;

        assert_eq!(users.len(), 3);
        assert_eq!(users[0]["user"]["email"], "user0@example.com");
        assert_eq!(users[1]["user"]["email"], "user1@example.com");
        assert_eq!(users[2]["user"]["email"], "user2@example.com");
    }
}
