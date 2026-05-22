use crate::response::AppResponse;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,

    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<IpAddr, RateLimitEntry>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::with_config(RateLimitConfig::default())
    }

    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let mut entries = self.entries.write().await;
        let now = Instant::now();

        match entries.get_mut(&ip) {
            Some(entry) => {
                if now.duration_since(entry.window_start) > self.config.window {
                    entry.count = 1;
                    entry.window_start = now;
                    true
                } else if entry.count >= self.config.max_requests {
                    false
                } else {
                    entry.count += 1;
                    true
                }
            }
            None => {
                entries.insert(
                    ip,
                    RateLimitEntry {
                        count: 1,
                        window_start: now,
                    },
                );
                true
            }
        }
    }

    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.write().await;
        let now = Instant::now();

        entries.retain(|_, entry| now.duration_since(entry.window_start) <= self.config.window);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_ip(request: &Request) -> Option<IpAddr> {
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip_str) = forwarded_str.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse() {
                    return Some(ip);
                }
            }
        }
    }

    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return Some(ip);
            }
        }
    }

    Some(IpAddr::from([127, 0, 0, 1]))
}

pub async fn rate_limit_middleware(
    limiter: Arc<RateLimiter>,
    request: Request,
    next: Next,
) -> Response {
    let ip = match extract_ip(&request) {
        Some(ip) => ip,
        None => {
            tracing::warn!("Failed to extract IP address for rate limiting");

            return next.run(request).await;
        }
    };

    if !limiter.check_rate_limit(ip).await {
        tracing::warn!("Rate limit exceeded for IP: {}", ip);
        return AppResponse::too_many_requests("Rate limit exceeded. Please try again later.");
    }

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            max_requests: 5,
            window: Duration::from_secs(60),
        };
        let limiter = RateLimiter::with_config(config);

        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        for _ in 0..5 {
            assert!(limiter.check_rate_limit(ip).await);
        }

        assert!(!limiter.check_rate_limit(ip).await);
    }

    #[tokio::test]
    async fn test_rate_limit_window_reset() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_millis(100),
        };
        let limiter = RateLimiter::with_config(config);

        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        assert!(limiter.check_rate_limit(ip).await);
        assert!(limiter.check_rate_limit(ip).await);

        assert!(!limiter.check_rate_limit(ip).await);

        tokio::time::sleep(Duration::from_millis(150)).await;

        assert!(limiter.check_rate_limit(ip).await);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let config = RateLimitConfig {
            max_requests: 5,
            window: Duration::from_millis(100),
        };
        let limiter = RateLimiter::with_config(config);

        let ip1: IpAddr = "127.0.0.1".parse().unwrap();
        let ip2: IpAddr = "192.168.1.1".parse().unwrap();

        limiter.check_rate_limit(ip1).await;
        limiter.check_rate_limit(ip2).await;

        {
            let entries = limiter.entries.read().await;
            assert_eq!(entries.len(), 2);
        }

        tokio::time::sleep(Duration::from_millis(150)).await;

        limiter.cleanup_expired().await;

        {
            let entries = limiter.entries.read().await;
            assert_eq!(entries.len(), 0);
        }
    }
}
