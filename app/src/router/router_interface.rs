use axum::Router;
use std::sync::Arc;

pub trait RouterInterface: Send + Sync {
    fn router(&self) -> Arc<Router>;

    fn generate_url(&self, name: &str, params: &[(&str, &str)]) -> String {
        let _ = params;
        name.to_string()
    }
}
