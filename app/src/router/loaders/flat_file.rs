use crate::app::App;

use crate::router::controllers::flat_file;
use crate::router::loader::{RouteInfo, RouteLoader};
use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

const DEFAULT_BASE_DIR: &str = "content";

const DEFAULT_FILE_EXTENSION: &str = "txt";

const DEFAULT_HOME_FOLDER: &str = "home";

const IGNORED_CURRENT_DIR: &str = ".";
const IGNORED_PARENT_DIR: &str = "..";
const IGNORED_DS_STORE: &str = ".DS_Store";
const IGNORED_GITIGNORE: &str = ".gitignore";
const IGNORED_GIT_DIR: &str = ".git";
const IGNORED_SVN_DIR: &str = ".svn";
const IGNORED_HTACCESS: &str = ".htaccess";
const IGNORED_THUMBS_DB: &str = "Thumb.db";
const IGNORED_SYNOLOGY_DIR: &str = "@eaDir";

#[derive(Clone, Debug)]
pub struct FlatFileConfig {
    pub base_dir: PathBuf,

    pub file_extension: String,

    pub home_folder: String,

    pub ignore: HashSet<String>,
}

impl Default for FlatFileConfig {
    fn default() -> Self {
        let mut ignore = HashSet::new();

        ignore.insert(IGNORED_CURRENT_DIR.to_string());
        ignore.insert(IGNORED_PARENT_DIR.to_string());
        ignore.insert(IGNORED_DS_STORE.to_string());
        ignore.insert(IGNORED_GITIGNORE.to_string());
        ignore.insert(IGNORED_GIT_DIR.to_string());
        ignore.insert(IGNORED_SVN_DIR.to_string());
        ignore.insert(IGNORED_HTACCESS.to_string());
        ignore.insert(IGNORED_THUMBS_DB.to_string());
        ignore.insert(IGNORED_SYNOLOGY_DIR.to_string());

        Self {
            base_dir: PathBuf::from(DEFAULT_BASE_DIR),
            file_extension: DEFAULT_FILE_EXTENSION.to_string(),
            home_folder: DEFAULT_HOME_FOLDER.to_string(),
            ignore,
        }
    }
}

impl FlatFileConfig {
    pub fn new<P: Into<PathBuf>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.into(),
            ..Default::default()
        }
    }

    pub fn with_extension(mut self, extension: impl Into<String>) -> Self {
        self.file_extension = extension.into();
        self
    }

    pub fn with_home_folder(mut self, home_folder: impl Into<String>) -> Self {
        self.home_folder = home_folder.into();
        self
    }

    pub fn ignore(mut self, patterns: Vec<String>) -> Self {
        self.ignore.extend(patterns);
        self
    }
}

pub struct FlatFileRouteLoader {
    config: FlatFileConfig,
    routes: Vec<RouteInfo>,
}

impl FlatFileRouteLoader {
    pub fn new() -> Self {
        Self::with_config(FlatFileConfig::default())
    }

    pub fn with_config(config: FlatFileConfig) -> Self {
        Self {
            config,
            routes: Vec::new(),
        }
    }

    fn strip_numeric_prefix<'a>(&self, name: &'a str) -> &'a str {
        lazy_static::lazy_static! {
            static ref NUMERIC_PREFIX: Regex = Regex::new(r"^\d+_")
                .expect("Invalid regex pattern for numeric prefix");
        }

        NUMERIC_PREFIX
            .find(name)
            .and_then(|m| name.get(m.end()..))
            .unwrap_or(name)
    }

    fn find_content_files(&self, dir: &Path) -> Vec<(PathBuf, String)> {
        let pattern = format!("*.{}", self.config.file_extension);
        let glob_pattern = dir.join(&pattern);

        match glob::glob(&glob_pattern.to_string_lossy()) {
            Ok(paths) => paths
                .filter_map(|p| p.ok())
                .filter(|p| p.is_file())
                .map(|p| {
                    let template_name = p
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("default")
                        .to_string();
                    (p, template_name)
                })
                .collect(),
            Err(e) => {
                tracing::error!("Failed to glob pattern {:?}: {}", glob_pattern, e);
                Vec::new()
            }
        }
    }

    fn add_routes(&mut self, router: Router<App>, dir: &Path, parent_path: &str) -> Router<App> {
        if !dir.is_dir() {
            return router;
        }

        let mut entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    !self.config.ignore.contains(&name)
                })
                .filter(|e| e.path().is_dir())
                .collect::<Vec<_>>(),
            Err(e) => {
                tracing::error!("Failed to read directory {:?}: {}", dir, e);
                return router;
            }
        };

        entries.sort_by(|a, b| {
            let a_name = a.file_name().to_string_lossy().to_string();
            let b_name = b.file_name().to_string_lossy().to_string();
            natord::compare(&a_name, &b_name)
        });

        let mut current_router = router;

        for entry in entries {
            let folder_path = entry.path();
            let folder_name = entry.file_name().to_string_lossy().to_string();

            let slug = self.strip_numeric_prefix(&folder_name);

            let url_path = if parent_path.is_empty() {
                slug.to_string()
            } else {
                format!("{}/{}", parent_path, slug)
            };

            let final_url_path = if url_path == self.config.home_folder {
                "".to_string()
            } else {
                url_path.clone()
            };

            let content_files = self.find_content_files(&folder_path);

            if let Some((content_file, template_name)) = content_files.first() {
                let route_path = format!("/{}", final_url_path);
                let route_name = if final_url_path.is_empty() {
                    "content_home".to_string()
                } else {
                    format!("content_{}", final_url_path.replace('/', "_"))
                };

                let content_file_str = content_file.to_string_lossy().to_string();
                let template_str = template_name.clone();
                let parent_str = if parent_path.is_empty() {
                    None
                } else {
                    Some(parent_path.to_string())
                };

                current_router =
                    current_router.route(
                        &route_path,
                        get(move |state: State<App>, session: tower_sessions::Session, request: Request<Body>| {
                            let cf = content_file_str.clone();
                            let ts = template_str.clone();
                            let ps = parent_str.clone();
                            async move {
                                flat_file::render_with_metadata(state, session, request, cf, ts, ps).await
                            }
                        }),
                    );

                self.routes
                    .push(RouteInfo::new(&route_name, &route_path, "GET").with_firewall("content"));

                tracing::debug!(
                    "Registered flat file route: {} -> {} (template: {})",
                    route_name,
                    route_path,
                    template_name
                );
            }

            current_router = self.add_routes(current_router, &folder_path, &url_path);
        }

        current_router
    }

    fn build(&mut self) -> Router<App> {
        let router = Router::new();

        let base_dir = self.config.base_dir.clone();

        if !base_dir.exists() {
            tracing::warn!("Flat file base directory does not exist: {:?}", base_dir);
            return router;
        }

        self.add_routes(router, &base_dir, "")
    }
}

impl Default for FlatFileRouteLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl RouteLoader<App> for FlatFileRouteLoader {
    fn load(&self) -> Router<App> {
        let mut loader = Self::with_config(self.config.clone());
        loader.build()
    }

    fn get_routes(&self) -> Vec<RouteInfo> {
        let mut loader = Self::with_config(self.config.clone());
        let _ = loader.build();
        loader.routes
    }
}
