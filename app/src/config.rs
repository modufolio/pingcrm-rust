pub use crate::app::AppConfig;
pub use appkit_core::tus::TusConfig;

#[path = "../config/mod.rs"]
pub mod routes;

pub use routes::configure_routes;

pub use routes::{configure_v1_resources, V1JsonApiConfig};

pub use appkit_core::jsonapi::Operations;
