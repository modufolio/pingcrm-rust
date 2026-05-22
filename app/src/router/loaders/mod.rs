pub mod admin;
pub mod api;
pub mod app;
pub mod clockwork;
pub mod flat_file;
pub mod jsonapi;
pub mod static_files;
pub mod web;

pub use admin::AdminRoutes;
pub use api::ApiRoutes;
pub use app::AppRoutes;
pub use clockwork::ClockworkRoutes;
pub use flat_file::{FlatFileConfig, FlatFileRouteLoader};
pub use jsonapi::JsonApiRouteLoader;
pub use static_files::StaticFileRoutes;
pub use web::WebRoutes;
