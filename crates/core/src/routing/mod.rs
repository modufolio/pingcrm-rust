use axum::Router;

#[derive(Clone, Debug)]
pub struct RouteInfo {
    pub name: String,
    pub path: String,
    pub method: String,
    pub firewall: String,
}

impl RouteInfo {
    pub fn new(
        name: impl Into<String>,
        path: impl Into<String>,
        method: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            method: method.into(),
            firewall: "unknown".to_string(),
        }
    }

    pub fn with_firewall(mut self, firewall: impl Into<String>) -> Self {
        self.firewall = firewall.into();
        self
    }
}

pub trait RouteLoader<S> {
    fn load(&self) -> Router<S>;

    fn get_routes(&self) -> Vec<RouteInfo>;
}

pub struct AppRouter<S = ()> {
    inner: Router<S>,
}

impl<S> AppRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: Router::new(),
        }
    }

    pub fn from_router(router: Router<S>) -> Self {
        Self { inner: router }
    }

    pub fn load<L>(mut self, loader: L) -> Self
    where
        L: RouteLoader<S>,
    {
        self.inner = self.inner.merge(loader.load());
        self
    }

    pub fn build(self) -> Router<S> {
        self.inner
    }

    pub fn merge(mut self, router: Router<S>) -> Self {
        self.inner = self.inner.merge(router);
        self
    }

    pub fn nest(mut self, path: &str, router: Router<S>) -> Self {
        self.inner = self.inner.nest(path, router);
        self
    }

    pub fn fallback<H, T>(mut self, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.fallback(handler);
        self
    }
}

impl<S> Default for AppRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}
