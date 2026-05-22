pub mod flat_file_config;
pub mod jsonapi_config;

pub use flat_file_config::{configure_content_routes, ContentRoutesConfig};
pub use jsonapi_config::{configure_v1_resources, V1JsonApiConfig};
