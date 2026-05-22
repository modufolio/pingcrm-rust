use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::database::schema::{clockwork_queries, clockwork_requests};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = clockwork_requests)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ClockworkRequest {
    pub id: String,
    pub version: i32,
    pub request_type: String,
    pub time: f64,
    pub method: String,
    pub url: String,
    pub uri: String,
    pub headers: Option<String>,
    pub get_data: Option<String>,
    pub post_data: Option<String>,
    pub cookies: Option<String>,
    pub response_status: i32,
    pub response_duration: f64,
    pub memory_usage: i64,
    pub queries_count: i32,
    pub queries_duration: f64,
    pub slow_queries: i32,
    pub middleware: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = clockwork_requests)]
pub struct NewClockworkRequest {
    pub id: String,
    pub version: i32,
    pub request_type: String,
    pub time: f64,
    pub method: String,
    pub url: String,
    pub uri: String,
    pub headers: Option<String>,
    pub get_data: Option<String>,
    pub post_data: Option<String>,
    pub cookies: Option<String>,
    pub response_status: i32,
    pub response_duration: f64,
    pub memory_usage: i64,
    pub queries_count: i32,
    pub queries_duration: f64,
    pub slow_queries: i32,
    pub middleware: Option<String>,
    pub created_at: NaiveDateTime,
}

impl NewClockworkRequest {
    pub fn new(method: String, url: String, uri: String) -> Self {
        use chrono::Utc;
        let now = Utc::now();
        let timestamp =
            now.timestamp() as f64 + (now.timestamp_subsec_micros() as f64 / 1_000_000.0);

        Self {
            id: format!("clk_{}.{}", now.timestamp(), now.timestamp_subsec_micros()),
            version: 1,
            request_type: "request".to_string(),
            time: timestamp,
            method,
            url,
            uri,
            headers: None,
            get_data: None,
            post_data: None,
            cookies: None,
            response_status: 200,
            response_duration: 0.0,
            memory_usage: 0,
            queries_count: 0,
            queries_duration: 0.0,
            slow_queries: 0,
            middleware: None,
            created_at: now.naive_utc(),
        }
    }

    pub fn with_headers(mut self, headers: serde_json::Value) -> Self {
        self.headers = Some(headers.to_string());
        self
    }

    pub fn with_get_data(mut self, data: serde_json::Value) -> Self {
        self.get_data = Some(data.to_string());
        self
    }

    pub fn with_post_data(mut self, data: serde_json::Value) -> Self {
        self.post_data = Some(data.to_string());
        self
    }

    pub fn with_cookies(mut self, cookies: serde_json::Value) -> Self {
        self.cookies = Some(cookies.to_string());
        self
    }

    pub fn with_response(mut self, status: i32, duration_ms: f64) -> Self {
        self.response_status = status;
        self.response_duration = duration_ms;
        self
    }

    pub fn with_memory_usage(mut self, bytes: i64) -> Self {
        self.memory_usage = bytes;
        self
    }

    pub fn with_query_stats(mut self, count: i32, duration_ms: f64, slow: i32) -> Self {
        self.queries_count = count;
        self.queries_duration = duration_ms;
        self.slow_queries = slow;
        self
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations,
)]
#[diesel(table_name = clockwork_queries)]
#[diesel(belongs_to(ClockworkRequest, foreign_key = request_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ClockworkQuery {
    pub id: i32,
    pub request_id: String,
    pub sql: String,
    pub bindings: Option<String>,
    pub duration: f64,
    pub query_type: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = clockwork_queries)]
pub struct NewClockworkQuery {
    pub request_id: String,
    pub sql: String,
    pub bindings: Option<String>,
    pub duration: f64,
    pub query_type: String,
}

impl NewClockworkQuery {
    pub fn new(request_id: String, sql: String, duration_ms: f64) -> Self {
        let query_type = if sql.trim().to_uppercase().starts_with("SELECT") {
            "SELECT"
        } else if sql.trim().to_uppercase().starts_with("INSERT") {
            "INSERT"
        } else if sql.trim().to_uppercase().starts_with("UPDATE") {
            "UPDATE"
        } else if sql.trim().to_uppercase().starts_with("DELETE") {
            "DELETE"
        } else {
            "OTHER"
        }
        .to_string();

        Self {
            request_id,
            sql,
            bindings: None,
            duration: duration_ms,
            query_type,
        }
    }

    pub fn with_bindings(mut self, bindings: serde_json::Value) -> Self {
        self.bindings = Some(bindings.to_string());
        self
    }
}
