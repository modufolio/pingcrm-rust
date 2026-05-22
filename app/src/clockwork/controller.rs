use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::app::App;
use crate::clockwork::repository::ClockworkRepository;
use crate::clockwork::{DebugStack, RequestDebugInfo};
use crate::database::models::{NewClockworkQuery, NewClockworkRequest};
use crate::database::DbPool;

pub struct ClockworkController {
    repository: ClockworkRepository,
    debug_stack: DebugStack,
}

impl ClockworkController {
    pub fn new(pool: DbPool, debug_stack: DebugStack) -> Self {
        Self {
            repository: ClockworkRepository::new(pool),
            debug_stack,
        }
    }

    pub async fn store_request(
        &self,
        response_status: i32,
        response_duration_ms: f64,
    ) -> Option<String> {
        let info = self.debug_stack.request_info()?;
        let queries = self.debug_stack.get_queries();
        let query_count = queries.len() as i32;
        let total_duration = self.debug_stack.total_query_time();
        let slow_queries = self.debug_stack.slow_query_count();

        let new_request =
            NewClockworkRequest::new(info.method.clone(), info.url.clone(), info.uri.clone())
                .with_headers(info.headers.clone())
                .with_get_data(info.get_data.clone())
                .with_post_data(info.post_data.clone())
                .with_cookies(info.cookies.clone())
                .with_response(response_status, response_duration_ms)
                .with_memory_usage(info.memory_start)
                .with_query_stats(query_count, total_duration, slow_queries);

        let request_id = new_request.id.clone();

        match self.repository.create_request(new_request).await {
            Ok(_) => {
                for query in queries {
                    if query.sql == "CONNECT" {
                        continue;
                    }

                    let new_query = NewClockworkQuery::new(
                        request_id.clone(),
                        query.sql.clone(),
                        query.execution_ms,
                    );

                    let new_query = if !query.params.is_empty() {
                        new_query.with_bindings(json!(query.params))
                    } else {
                        new_query
                    };

                    if let Err(e) = self.repository.create_query(new_query).await {
                        tracing::error!("Failed to store clockwork query: {}", e);
                    }
                }

                if let Err(e) = self.repository.cleanup_old_requests(1000).await {
                    tracing::error!("Failed to cleanup old clockwork requests: {}", e);
                }

                Some(request_id)
            }
            Err(e) => {
                tracing::error!("Failed to store clockwork request: {}", e);
                None
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClockworkResponse {
    pub id: String,
    pub version: i32,
    #[serde(rename = "type")]
    pub request_type: String,
    pub time: f64,
    pub method: String,
    pub url: String,
    pub uri: String,
    pub headers: serde_json::Value,
    pub controller: Option<String>,
    pub get_data: serde_json::Value,
    pub post_data: serde_json::Value,
    pub session_data: serde_json::Value,
    pub cookies: serde_json::Value,
    pub response_time: f64,
    pub response_status: i32,
    pub response_duration: f64,
    pub memory_usage: i64,
    pub middleware: serde_json::Value,
    pub database_queries: Vec<DatabaseQueryResponse>,
    pub database_duration: f64,
    pub database_queries_count: i32,
    pub database_slow_queries: i32,
    pub database_selects: i32,
    pub database_inserts: i32,
    pub database_updates: i32,
    pub database_deletes: i32,
    pub database_others: i32,
    pub timeline_data: Vec<serde_json::Value>,
    pub log: Vec<serde_json::Value>,
    pub events: Vec<serde_json::Value>,
    pub routes: Vec<serde_json::Value>,
    pub emails_data: Vec<serde_json::Value>,
    pub views_data: Vec<serde_json::Value>,
    pub user_data: serde_json::Value,
    pub subrequests: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseQueryResponse {
    pub query: String,
    pub bindings: serde_json::Value,
    pub duration: f64,
    pub connection: String,
    pub time: f64,
    pub file: Option<String>,
    pub line: Option<i32>,
    pub trace: Option<Vec<String>>,
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

pub async fn list_requests(State(state): State<App>) -> Result<Response, StatusCode> {
    let repository = ClockworkRepository::new(state.db_pool.clone());

    match repository.get_last_request_ids(50).await {
        Ok(ids) => Ok(Json(ids).into_response()),
        Err(e) => {
            tracing::error!("Failed to list clockwork requests: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_latest(State(state): State<App>) -> Result<Response, StatusCode> {
    let repository = ClockworkRepository::new(state.db_pool.clone());

    match repository.get_latest_request().await {
        Ok(Some(request)) => {
            let response = format_request_response(&repository, request).await;
            Ok(Json(response).into_response())
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get latest clockwork request: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_request(
    State(state): State<App>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let repository = ClockworkRepository::new(state.db_pool.clone());

    match repository.find_request_by_id(&id).await {
        Ok(Some(request)) => {
            let response = format_request_response(&repository, request).await;
            Ok(Json(response).into_response())
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get clockwork request {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_next(
    State(state): State<App>,
    Path((id, limit)): Path<(String, i64)>,
) -> Result<Response, StatusCode> {
    let repository = ClockworkRepository::new(state.db_pool.clone());

    match repository.get_next_requests(&id, limit).await {
        Ok(requests) => {
            let mut responses = Vec::new();
            for request in requests {
                responses.push(format_request_response(&repository, request).await);
            }
            Ok(Json(responses).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to get next clockwork requests after {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_previous(
    State(state): State<App>,
    Path((id, limit)): Path<(String, i64)>,
) -> Result<Response, StatusCode> {
    let repository = ClockworkRepository::new(state.db_pool.clone());

    match repository.get_previous_requests(&id, limit).await {
        Ok(requests) => {
            let mut responses = Vec::new();
            for request in requests {
                responses.push(format_request_response(&repository, request).await);
            }
            Ok(Json(responses).into_response())
        }
        Err(e) => {
            tracing::error!(
                "Failed to get previous clockwork requests before {}: {}",
                id,
                e
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn format_request_response(
    repository: &ClockworkRepository,
    request: crate::database::models::ClockworkRequest,
) -> ClockworkResponse {
    let queries = repository
        .get_queries_for_request(&request.id)
        .await
        .unwrap_or_default();

    let mut selects = 0;
    let mut inserts = 0;
    let mut updates = 0;
    let mut deletes = 0;
    let mut others = 0;

    let database_queries: Vec<DatabaseQueryResponse> = queries
        .into_iter()
        .map(|q| {
            match q.query_type.as_str() {
                "SELECT" => selects += 1,
                "INSERT" => inserts += 1,
                "UPDATE" => updates += 1,
                "DELETE" => deletes += 1,
                _ => others += 1,
            }

            let tags = if q.duration > 100.0 {
                vec!["slow".to_string()]
            } else {
                Vec::new()
            };

            DatabaseQueryResponse {
                query: q.sql,
                bindings: q
                    .bindings
                    .and_then(|b| serde_json::from_str(&b).ok())
                    .unwrap_or(json!([])),
                duration: q.duration,
                connection: "default".to_string(),
                time: request.time,
                file: None,
                line: None,
                trace: None,
                model: None,
                tags,
            }
        })
        .collect();

    ClockworkResponse {
        id: request.id,
        version: request.version,
        request_type: request.request_type,
        time: request.time,
        method: request.method,
        url: request.url,
        uri: request.uri,
        headers: request
            .headers
            .and_then(|h| serde_json::from_str(&h).ok())
            .unwrap_or(json!({})),
        controller: None,
        get_data: request
            .get_data
            .and_then(|d| serde_json::from_str(&d).ok())
            .unwrap_or(json!({})),
        post_data: request
            .post_data
            .and_then(|d| serde_json::from_str(&d).ok())
            .unwrap_or(json!({})),
        session_data: json!({}),
        cookies: request
            .cookies
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or(json!({})),
        response_time: request.time + (request.response_duration / 1000.0),
        response_status: request.response_status,
        response_duration: request.response_duration,
        memory_usage: request.memory_usage,
        middleware: request
            .middleware
            .and_then(|m| serde_json::from_str(&m).ok())
            .unwrap_or(json!([])),
        database_queries,
        database_duration: request.queries_duration,
        database_queries_count: request.queries_count,
        database_slow_queries: request.slow_queries,
        database_selects: selects,
        database_inserts: inserts,
        database_updates: updates,
        database_deletes: deletes,
        database_others: others,
        timeline_data: Vec::new(),
        log: Vec::new(),
        events: Vec::new(),
        routes: Vec::new(),
        emails_data: Vec::new(),
        views_data: Vec::new(),
        user_data: json!({}),
        subrequests: Vec::new(),
    }
}

pub fn start_request_tracking(
    debug_stack: &DebugStack,
    method: &str,
    url: &str,
    uri: &str,
    headers: serde_json::Value,
    query_params: serde_json::Value,
    post_data: serde_json::Value,
) {
    let info = RequestDebugInfo {
        method: method.to_string(),
        url: url.to_string(),
        uri: uri.to_string(),
        headers,
        get_data: query_params,
        post_data,
        cookies: json!({}),
        start_time: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0),
        memory_start: 0,
    };

    debug_stack.start_request(info);
}
