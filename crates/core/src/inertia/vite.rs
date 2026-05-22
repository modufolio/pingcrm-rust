use serde::Deserialize;
use std::{collections::HashMap, fs, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct ManifestEntry {
    pub file: String,
    #[serde(default)]
    pub css: Vec<String>,
    #[serde(default)]
    pub imports: Vec<String>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Vite manifest not found at {0}. Run `npm run build` or start `npm run dev`.")]
    ManifestMissing(PathBuf),
    #[error("failed to parse manifest: {0}")]
    ParseManifest(#[from] serde_json::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub enum Vite {
    Dev {
        server_url: String,
    },
    Prod {
        manifest: HashMap<String, ManifestEntry>,
    },
}

impl Vite {
    pub fn detect(public_dir: impl Into<PathBuf>) -> Result<Self, Error> {
        let build = public_dir.into().join("build");

        let dev_marker = build.join(".vite-dev");
        if dev_marker.is_file() {
            let url = fs::read_to_string(&dev_marker)?.trim().to_string();
            if !url.is_empty() {
                return Ok(Vite::Dev { server_url: url });
            }
        }

        let manifest_path = build.join(".vite").join("manifest.json");
        if !manifest_path.is_file() {
            return Err(Error::ManifestMissing(manifest_path));
        }
        let raw = fs::read_to_string(&manifest_path)?;
        let manifest: HashMap<String, ManifestEntry> = serde_json::from_str(&raw)?;
        Ok(Vite::Prod { manifest })
    }

    pub fn tags(&self, entry: &str) -> String {
        match self {
            Vite::Dev { server_url } => format!(
                concat!(
                    r#"<script type="module" src="{0}/@vite/client"></script>"#,
                    "\n",
                    r#"<script type="module" src="{0}/{1}"></script>"#,
                ),
                server_url, entry,
            ),
            Vite::Prod { manifest } => {
                let Some(item) = manifest.get(entry) else {
                    return format!("<!-- vite: entry '{entry}' missing from manifest -->");
                };
                let mut out = String::new();
                let mut emitted_css: std::collections::HashSet<&String> =
                    std::collections::HashSet::new();
                collect_css(manifest, item, &mut out, &mut emitted_css);
                out.push_str(&format!(
                    r#"<script type="module" src="/build/{}"></script>"#,
                    item.file,
                ));
                out
            }
        }
    }
}

fn collect_css<'a>(
    manifest: &'a HashMap<String, ManifestEntry>,
    entry: &'a ManifestEntry,
    out: &mut String,
    seen: &mut std::collections::HashSet<&'a String>,
) {
    for css in &entry.css {
        if seen.insert(css) {
            out.push_str(&format!(r#"<link rel="stylesheet" href="/build/{css}">"#,));
            out.push('\n');
        }
    }
    for import in &entry.imports {
        if let Some(child) = manifest.get(import) {
            collect_css(manifest, child, out, seen);
        }
    }
}
