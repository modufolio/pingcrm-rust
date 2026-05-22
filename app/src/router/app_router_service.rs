use super::router_interface::RouterInterface;
use axum::Router;
use std::sync::Arc;

pub struct AppRouterService {
    router: Arc<Router>,
}

impl AppRouterService {
    pub fn new(router: Router) -> Self {
        Self {
            router: Arc::new(router),
        }
    }
}

impl RouterInterface for AppRouterService {
    fn router(&self) -> Arc<Router> {
        self.router.clone()
    }

    fn generate_url(&self, name: &str, params: &[(&str, &str)]) -> String {
        let _ = params;
        name.to_string()
    }
}
