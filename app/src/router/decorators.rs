use crate::app::App;

use crate::router::loader::RouteLoader;
use appkit_core::security::RateLimiter;
use axum::{middleware, Router};
use std::sync::Arc;

pub struct WithRateLimit<L> {
    inner: L,
    rate_limiter: Arc<RateLimiter>,
}

impl<L> WithRateLimit<L> {
    pub fn new(inner: L, rate_limiter: Arc<RateLimiter>) -> Self {
        Self {
            inner,
            rate_limiter,
        }
    }
}

impl<L> RouteLoader<App> for WithRateLimit<L>
where
    L: RouteLoader<App>,
{
    fn load(&self) -> Router<App> {
        let limiter = self.rate_limiter.clone();

        self.inner.load().layer(middleware::from_fn(move |request, next| {
            let limiter = limiter.clone();
            async move {
                appkit_core::security::rate_limit_middleware(limiter, request, next).await
            }
        }))
    }

    fn get_routes(&self) -> Vec<appkit_core::routing::RouteInfo> {
        self.inner.get_routes()
    }
}

pub struct WithBodyLimit<L> {
    inner: L,
    limit: usize,
}

impl<L> WithBodyLimit<L> {
    pub fn new(inner: L, limit: usize) -> Self {
        Self { inner, limit }
    }
}

impl<L> RouteLoader<App> for WithBodyLimit<L>
where
    L: RouteLoader<App>,
{
    fn load(&self) -> Router<App> {
        use axum::extract::DefaultBodyLimit;

        self.inner.load().layer(DefaultBodyLimit::max(self.limit))
    }

    fn get_routes(&self) -> Vec<appkit_core::routing::RouteInfo> {
        self.inner.get_routes()
    }
}

pub struct WithPrefix<L> {
    inner: L,
    prefix: String,
}

impl<L> WithPrefix<L> {
    pub fn new(inner: L, prefix: impl Into<String>) -> Self {
        Self {
            inner,
            prefix: prefix.into(),
        }
    }
}

impl<L> RouteLoader<App> for WithPrefix<L>
where
    L: RouteLoader<App>,
{
    fn load(&self) -> Router<App> {
        Router::new().nest(&self.prefix, self.inner.load())
    }

    fn get_routes(&self) -> Vec<appkit_core::routing::RouteInfo> {
        self.inner
            .get_routes()
            .into_iter()
            .map(|mut r| {
                r.path = format!("{}{}", self.prefix, r.path);
                r
            })
            .collect()
    }
}
