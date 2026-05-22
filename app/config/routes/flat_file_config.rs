use crate::router::loaders::{FlatFileConfig, FlatFileRouteLoader};
use std::path::PathBuf;

pub struct ContentRoutesConfig {
    pub loader: FlatFileRouteLoader,
}

pub fn configure_content_routes() -> ContentRoutesConfig {
    let config = FlatFileConfig::default().with_home_folder("");

    let loader = FlatFileRouteLoader::with_config(config);

    ContentRoutesConfig { loader }
}

#[allow(dead_code)]
pub fn get_content_structure() -> Vec<String> {
    let config = FlatFileConfig::default().with_home_folder("");
    let base_dir = config.base_dir.clone();

    if !base_dir.exists() {
        return vec![];
    }

    let mut routes = Vec::new();
    collect_routes(&base_dir, "", &config, &mut routes);
    routes
}

#[allow(dead_code)]
fn collect_routes(
    dir: &PathBuf,
    parent_path: &str,
    config: &FlatFileConfig,
    routes: &mut Vec<String>,
) {
    if !dir.is_dir() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut folders: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                !config.ignore.contains(&name) && e.path().is_dir()
            })
            .collect();

        folders.sort_by(|a, b| {
            let a_name = a.file_name().to_string_lossy().to_string();
            let b_name = b.file_name().to_string_lossy().to_string();
            natord::compare(&a_name, &b_name)
        });

        for entry in folders {
            let folder_name = entry.file_name().to_string_lossy().to_string();
            let slug = strip_numeric_prefix(&folder_name);

            let url_path = if parent_path.is_empty() {
                slug.to_string()
            } else {
                format!("{}/{}", parent_path, slug)
            };

            let final_path = if url_path == config.home_folder {
                "/".to_string()
            } else {
                format!("/{}", url_path)
            };

            routes.push(final_path);

            collect_routes(&entry.path(), &url_path, config, routes);
        }
    }
}

#[allow(dead_code)]
fn strip_numeric_prefix(name: &str) -> &str {
    lazy_static::lazy_static! {
        static ref NUMERIC_PREFIX: regex::Regex = regex::Regex::new(r"^\d+_").unwrap();
    }

    NUMERIC_PREFIX
        .find(name)
        .and_then(|m| name.get(m.end()..))
        .unwrap_or(name)
}
