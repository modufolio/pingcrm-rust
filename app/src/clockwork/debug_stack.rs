use serde::{Deserialize, Serialize};

use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub sql: String,
    pub params: Vec<serde_json::Value>,
    pub types: Vec<String>,
    pub execution_ms: f64,
}

impl Query {
    pub fn new(sql: String, execution_ms: f64) -> Self {
        Self {
            sql,
            params: Vec::new(),
            types: Vec::new(),
            execution_ms,
        }
    }

    pub fn with_params(mut self, params: Vec<serde_json::Value>) -> Self {
        self.params = params;
        self
    }

    pub fn with_types(mut self, types: Vec<String>) -> Self {
        self.types = types;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDebugInfo {
    pub method: String,
    pub url: String,
    pub uri: String,
    pub headers: serde_json::Value,
    pub get_data: serde_json::Value,
    pub post_data: serde_json::Value,
    pub cookies: serde_json::Value,
    pub start_time: f64,
    pub memory_start: i64,
}

impl Default for RequestDebugInfo {
    fn default() -> Self {
        Self {
            method: String::new(),
            url: String::new(),
            uri: String::new(),
            headers: serde_json::json!({}),
            get_data: serde_json::json!({}),
            post_data: serde_json::json!({}),
            cookies: serde_json::json!({}),
            start_time: 0.0,
            memory_start: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DebugStack {
    inner: Arc<Mutex<DebugStackInner>>,
}

#[derive(Debug)]
struct DebugStackInner {
    queries: Vec<Query>,
    max_queries: usize,
    request_info: Option<RequestDebugInfo>,
    request_start: Option<Instant>,
}

impl Default for DebugStack {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugStack {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DebugStackInner {
                queries: Vec::new(),
                max_queries: 100,
                request_info: None,
                request_start: None,
            })),
        }
    }

    pub fn with_max_queries(max: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DebugStackInner {
                queries: Vec::new(),
                max_queries: max,
                request_info: None,
                request_start: None,
            })),
        }
    }

    pub fn set_max_queries(&self, max: usize) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.max_queries = max;
        }
    }

    pub fn start_request(&self, info: RequestDebugInfo) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.request_info = Some(info);
            inner.request_start = Some(Instant::now());
            inner.queries.clear();
        }
    }

    pub fn append(&self, query: Query) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.queries.push(query);

            if inner.queries.len() > inner.max_queries {
                let start = inner.queries.len() - inner.max_queries;
                inner.queries = inner.queries[start..].to_vec();
            }
        }
    }

    pub fn log_query(&self, sql: &str, execution_ms: f64) {
        let query = Query::new(sql.to_string(), execution_ms);
        self.append(query);
    }

    pub fn log_query_with_params(
        &self,
        sql: &str,
        params: Vec<serde_json::Value>,
        execution_ms: f64,
    ) {
        let query = Query::new(sql.to_string(), execution_ms).with_params(params);
        self.append(query);
    }

    pub fn get_queries(&self) -> Vec<Query> {
        self.inner
            .lock()
            .map(|inner| inner.queries.clone())
            .unwrap_or_default()
    }

    pub fn query_count(&self) -> usize {
        self.inner
            .lock()
            .map(|inner| inner.queries.len())
            .unwrap_or(0)
    }

    pub fn total_query_time(&self) -> f64 {
        self.inner
            .lock()
            .map(|inner| inner.queries.iter().map(|q| q.execution_ms).sum())
            .unwrap_or(0.0)
    }

    pub fn slow_query_count(&self) -> i32 {
        self.slow_query_count_threshold(100.0)
    }

    pub fn slow_query_count_threshold(&self, threshold_ms: f64) -> i32 {
        self.inner
            .lock()
            .map(|inner| {
                inner
                    .queries
                    .iter()
                    .filter(|q| q.execution_ms > threshold_ms)
                    .count() as i32
            })
            .unwrap_or(0)
    }

    pub fn query_counts_by_type(&self) -> QueryCounts {
        self.inner
            .lock()
            .map(|inner| {
                let mut counts = QueryCounts::default();
                for query in &inner.queries {
                    let sql_upper = query.sql.trim().to_uppercase();
                    if sql_upper.starts_with("SELECT") {
                        counts.selects += 1;
                    } else if sql_upper.starts_with("INSERT") {
                        counts.inserts += 1;
                    } else if sql_upper.starts_with("UPDATE") {
                        counts.updates += 1;
                    } else if sql_upper.starts_with("DELETE") {
                        counts.deletes += 1;
                    } else {
                        counts.others += 1;
                    }
                }
                counts
            })
            .unwrap_or_default()
    }

    pub fn request_start(&self) -> Option<Instant> {
        self.inner
            .lock()
            .map(|inner| inner.request_start)
            .unwrap_or(None)
    }

    pub fn request_info(&self) -> Option<RequestDebugInfo> {
        self.inner
            .lock()
            .map(|inner| inner.request_info.clone())
            .unwrap_or(None)
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.inner
            .lock()
            .map(|inner| {
                inner
                    .request_start
                    .map(|start| start.elapsed().as_secs_f64() * 1000.0)
                    .unwrap_or(0.0)
            })
            .unwrap_or(0.0)
    }

    pub fn reset(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.queries.clear();
            inner.request_info = None;
            inner.request_start = None;
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QueryCounts {
    pub selects: i32,
    pub inserts: i32,
    pub updates: i32,
    pub deletes: i32,
    pub others: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_stack_basic() {
        let stack = DebugStack::new();

        stack.log_query("SELECT * FROM users", 5.5);
        stack.log_query("INSERT INTO users VALUES (1)", 2.3);

        assert_eq!(stack.query_count(), 2);
        assert_eq!(stack.total_query_time(), 7.8);
    }

    #[test]
    fn test_debug_stack_circular_buffer() {
        let stack = DebugStack::with_max_queries(3);

        for i in 0..5 {
            stack.log_query(&format!("SELECT {}", i), 1.0);
        }

        assert_eq!(stack.query_count(), 3);

        let queries = stack.get_queries();
        assert_eq!(queries[0].sql, "SELECT 2");
        assert_eq!(queries[1].sql, "SELECT 3");
        assert_eq!(queries[2].sql, "SELECT 4");
    }

    #[test]
    fn test_query_counts_by_type() {
        let stack = DebugStack::new();

        stack.log_query("SELECT * FROM users", 1.0);
        stack.log_query("SELECT * FROM orders", 1.0);
        stack.log_query("INSERT INTO users VALUES (1)", 1.0);
        stack.log_query("UPDATE users SET name = 'test'", 1.0);
        stack.log_query("DELETE FROM users WHERE id = 1", 1.0);

        let counts = stack.query_counts_by_type();
        assert_eq!(counts.selects, 2);
        assert_eq!(counts.inserts, 1);
        assert_eq!(counts.updates, 1);
        assert_eq!(counts.deletes, 1);
    }

    #[test]
    fn test_slow_query_count() {
        let stack = DebugStack::new();

        stack.log_query("SELECT 1", 50.0);
        stack.log_query("SELECT 2", 150.0);
        stack.log_query("SELECT 3", 200.0);

        assert_eq!(stack.slow_query_count(), 2);
        assert_eq!(stack.slow_query_count_threshold(175.0), 1);
    }
}
