use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct RateLimiterState {
    requests: Mutex<HashMap<String, (u32, Instant)>>,
}

impl RateLimiterState {
    pub fn new() -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for RateLimiterState {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn rate_limit_middleware(
    axum::extract::State(rate_limiter): axum::extract::State<Arc<RateLimiterState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = addr.ip().to_string();
    let now = Instant::now();
    let window_duration = Duration::from_secs(60);
    let max_requests = 100u32;

    let allowed = {
        let mut requests = rate_limiter.requests.lock().unwrap();

        let entry = requests.entry(client_ip.clone()).or_insert((0, now));

        if now.duration_since(entry.1) > window_duration {
            entry.0 = 0;
            entry.1 = now;
        }

        if entry.0 >= max_requests {
            false
        } else {
            entry.0 += 1;
            true
        }
    };

    if !allowed {
        tracing::warn!("Rate limit exceeded for IP: {}", client_ip);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(request).await)
}

pub async fn simple_rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    Ok(next.run(request).await)
}
