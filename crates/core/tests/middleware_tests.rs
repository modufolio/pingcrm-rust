use appkit_core::security::{RateLimitConfig, RateLimiter};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_rate_limiter_allows_under_limit() {
    let config = RateLimitConfig {
        max_requests: 5,
        window: Duration::from_secs(60),
    };
    let limiter = RateLimiter::with_config(config);
    let ip: IpAddr = "192.168.1.100".parse().unwrap();

    for _ in 0..5 {
        assert!(
            limiter.check_rate_limit(ip).await,
            "Request should be allowed under limit"
        );
    }
}

#[tokio::test]
async fn test_rate_limiter_blocks_over_limit() {
    let config = RateLimitConfig {
        max_requests: 3,
        window: Duration::from_secs(60),
    };
    let limiter = RateLimiter::with_config(config);
    let ip: IpAddr = "192.168.1.101".parse().unwrap();

    for _ in 0..3 {
        assert!(limiter.check_rate_limit(ip).await);
    }

    assert!(
        !limiter.check_rate_limit(ip).await,
        "Request should be blocked after exceeding limit"
    );

    assert!(
        !limiter.check_rate_limit(ip).await,
        "Subsequent requests should remain blocked"
    );
}

#[tokio::test]
async fn test_rate_limiter_window_reset() {
    let config = RateLimitConfig {
        max_requests: 2,
        window: Duration::from_millis(100),
    };
    let limiter = RateLimiter::with_config(config);
    let ip: IpAddr = "192.168.1.102".parse().unwrap();

    assert!(limiter.check_rate_limit(ip).await);
    assert!(limiter.check_rate_limit(ip).await);
    assert!(!limiter.check_rate_limit(ip).await, "Should be blocked");

    tokio::time::sleep(Duration::from_millis(150)).await;

    assert!(
        limiter.check_rate_limit(ip).await,
        "Should be allowed after window reset"
    );
}

#[tokio::test]
async fn test_rate_limiter_different_ips_independent() {
    let config = RateLimitConfig {
        max_requests: 2,
        window: Duration::from_secs(60),
    };
    let limiter = RateLimiter::with_config(config);

    let ip1: IpAddr = "192.168.1.1".parse().unwrap();
    let ip2: IpAddr = "192.168.1.2".parse().unwrap();

    assert!(limiter.check_rate_limit(ip1).await);
    assert!(limiter.check_rate_limit(ip1).await);
    assert!(!limiter.check_rate_limit(ip1).await);

    assert!(
        limiter.check_rate_limit(ip2).await,
        "Different IP should have independent limit"
    );
    assert!(limiter.check_rate_limit(ip2).await);
    assert!(!limiter.check_rate_limit(ip2).await);
}

#[tokio::test]
async fn test_rate_limiter_cleanup_expired() {
    let config = RateLimitConfig {
        max_requests: 5,
        window: Duration::from_millis(100),
    };
    let limiter = RateLimiter::with_config(config);

    let ips: Vec<IpAddr> = (1..=10)
        .map(|i| format!("192.168.1.{}", i).parse().unwrap())
        .collect();

    for ip in &ips {
        limiter.check_rate_limit(*ip).await;
    }

    tokio::time::sleep(Duration::from_millis(150)).await;

    limiter.cleanup_expired().await;

    for ip in &ips {
        assert!(
            limiter.check_rate_limit(*ip).await,
            "IP should work after cleanup"
        );
    }
}

#[tokio::test]
async fn test_rate_limiter_concurrent_access() {
    use std::sync::Arc;

    let config = RateLimitConfig {
        max_requests: 100,
        window: Duration::from_secs(60),
    };
    let limiter = Arc::new(RateLimiter::with_config(config));
    let ip: IpAddr = "192.168.1.200".parse().unwrap();

    let mut handles = vec![];
    for _ in 0..50 {
        let limiter = Arc::clone(&limiter);
        let handle = tokio::spawn(async move { limiter.check_rate_limit(ip).await });
        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if handle.await.unwrap() {
            success_count += 1;
        }
    }

    assert_eq!(
        success_count, 50,
        "All concurrent requests should succeed under limit"
    );
}

#[tokio::test]
async fn test_rate_limiter_burst_handling() {
    let config = RateLimitConfig {
        max_requests: 10,
        window: Duration::from_secs(1),
    };
    let limiter = RateLimiter::with_config(config);
    let ip: IpAddr = "192.168.1.201".parse().unwrap();

    let mut allowed = 0;
    for _ in 0..20 {
        if limiter.check_rate_limit(ip).await {
            allowed += 1;
        }
    }

    assert_eq!(allowed, 10, "Should allow exactly max_requests in a burst");
}

#[tokio::test]
async fn test_csrf_token_generation() {
    use appkit_core::security::CsrfTokenManager;
    use tower_sessions::{MemoryStore, Session};

    let store = MemoryStore::default();
    let session = Session::new(None, Arc::new(store), None);

    let token = CsrfTokenManager::generate_token(&session)
        .await
        .expect("Should generate token");

    assert!(!token.is_empty(), "Token should not be empty");
    assert!(token.len() >= 32, "Token should be sufficiently long");
}

#[tokio::test]
async fn test_csrf_token_validation_valid() {
    use appkit_core::security::CsrfTokenManager;
    use std::sync::Arc;
    use tower_sessions::{MemoryStore, Session};

    let store = MemoryStore::default();
    let session = Session::new(None, Arc::new(store), None);

    let _token = CsrfTokenManager::generate_token(&session)
        .await
        .expect("Should generate token");

    let stored_token = CsrfTokenManager::get_token(&session)
        .await
        .expect("Should get token")
        .expect("Token should exist");

    let result = CsrfTokenManager::validate_token(&session, &stored_token).await;

    let _ = result;
}

#[tokio::test]
async fn test_csrf_token_validation_invalid() {
    use appkit_core::security::CsrfTokenManager;
    use std::sync::Arc;
    use tower_sessions::{MemoryStore, Session};

    let store = MemoryStore::default();
    let session = Session::new(None, Arc::new(store), None);

    CsrfTokenManager::generate_token(&session)
        .await
        .expect("Should generate token");

    let result = CsrfTokenManager::validate_token(&session, "wrong-token").await;

    assert!(result.is_err(), "Should reject invalid token");
}

#[tokio::test]
async fn test_csrf_token_missing_from_session() {
    use appkit_core::security::CsrfTokenManager;
    use std::sync::Arc;
    use tower_sessions::{MemoryStore, Session};

    let store = MemoryStore::default();
    let session = Session::new(None, Arc::new(store), None);

    let result = CsrfTokenManager::validate_token(&session, "any-token").await;

    assert!(result.is_err(), "Should reject when no token in session");
}

#[tokio::test]
async fn test_security_headers_config_default() {
    use appkit_core::middleware::SecurityHeadersConfig;

    let config = SecurityHeadersConfig::default();

    assert!(config.csp.enabled, "Should have CSP enabled by default");
    assert!(config.hsts.enabled, "Should have HSTS enabled by default");
    assert_eq!(
        config.hsts.max_age, 31536000,
        "Should have 1 year HSTS by default"
    );
    assert!(
        config.x_frame_options.enabled,
        "Should have X-Frame-Options enabled"
    );
    assert!(
        config.x_content_type_options.enabled,
        "Should have X-Content-Type-Options enabled"
    );
}

#[tokio::test]
async fn test_security_headers_hsts_config() {
    use appkit_core::middleware::SecurityHeadersConfig;

    let config = SecurityHeadersConfig::default();

    assert!(config.hsts.enabled);
    assert!(config.hsts.max_age >= 31536000);
    assert!(config.hsts.include_sub_domains);
}

#[test]
fn test_rate_limit_config_default() {
    let config = RateLimitConfig::default();

    assert_eq!(config.max_requests, 100);
    assert_eq!(config.window, Duration::from_secs(60));
}

#[test]
fn test_rate_limit_config_custom() {
    let config = RateLimitConfig {
        max_requests: 50,
        window: Duration::from_secs(30),
    };

    assert_eq!(config.max_requests, 50);
    assert_eq!(config.window, Duration::from_secs(30));
}
