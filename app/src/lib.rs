pub use appkit_core;

use diesel_migrations::{embed_migrations, EmbeddedMigrations};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub mod app;
pub mod config;
pub mod console;
pub mod data;
pub mod database;
pub mod events;
pub mod handlers;
pub mod inertia;
pub mod listeners;
pub mod middleware;
pub mod registry;
pub mod requests;
pub mod router;
pub mod state;

pub mod clockwork;
pub mod extractors;
pub mod filter;
pub mod fixtures;
pub mod presenter;
pub mod seeder;

pub use app::{App, AppConfig, AppFactory};
pub use config as routes_config;
