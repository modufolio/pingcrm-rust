use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::Response,
};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tower_sessions::Session;

use crate::app::App;
use crate::data::Txt;
use crate::inertia::DefaultProps;
use appkit_core::security::user::User;
use appkit_core::{
    error::{AppError, AppResult},
    response::AppResponse,
};

pub type MicrocontrollerFn = Arc<
    dyn Fn(
            &Request<Body>,
            &HashMap<String, String>,
            &HashMap<String, String>,
        ) -> HashMap<String, Value>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct MicrocontrollerRegistry {
    handlers: Arc<std::sync::RwLock<HashMap<String, MicrocontrollerFn>>>,
}

impl MicrocontrollerRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    pub fn register<F>(&self, template_name: impl Into<String>, handler: F)
    where
        F: Fn(
                &Request<Body>,
                &HashMap<String, String>,
                &HashMap<String, String>,
            ) -> HashMap<String, Value>
            + Send
            + Sync
            + 'static,
    {
        let handlers = &mut *self.handlers.write().unwrap();
        handlers.insert(template_name.into(), Arc::new(handler));
    }

    pub fn get(&self, template_name: &str) -> Option<MicrocontrollerFn> {
        let handlers = self.handlers.read().unwrap();
        handlers.get(template_name).cloned()
    }
}

impl Default for MicrocontrollerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn render_with_metadata(
    State(state): State<App>,
    session: Session,
    request: Request<Body>,
    content_file: String,
    template_name: String,
    parent: Option<String>,
) -> AppResult<Response> {
    let data = parse_content_file(&content_file)?;

    let microcontroller_context = if let Some(registry) = state.microcontroller_registry.as_ref() {
        if let Some(handler) = registry.get(&template_name) {
            let meta = create_meta(&template_name, parent.as_deref(), &content_file);
            handler(&request, &data, &meta)
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    let mut view_data = Map::new();

    for (key, value) in &data {
        view_data.insert(key.clone(), Value::String(value.clone()));
    }

    for (key, value) in microcontroller_context {
        view_data.insert(key, value);
    }

    view_data.insert(
        "_template".to_string(),
        Value::String(template_name.clone()),
    );
    if let Some(ref parent_val) = parent {
        view_data.insert("_parent".to_string(), Value::String(parent_val.clone()));
    }
    view_data.insert(
        "_content_file".to_string(),
        Value::String(content_file.clone()),
    );

    let props = DefaultProps::merge(request.extensions().get::<User>(), &session, &state, Value::Object(view_data)).await;

    let component_name = to_pascal_case(&template_name);
    let response = AppResponse::inertia(&format!("Content/{}", component_name))
        .with_props(props)
        .render(&request);

    Ok(response)
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ' ')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn parse_content_file(filepath: &str) -> AppResult<HashMap<String, String>> {
    let path = PathBuf::from(filepath);

    if !path.exists() {
        return Err(AppError::NotFound(format!(
            "Content file not found: {}",
            filepath
        )));
    }

    let content = std::fs::read_to_string(&path).map_err(|e| {
        AppError::InternalError(format!("Failed to read content file {}: {}", filepath, e))
    })?;

    Ok(Txt::decode(&content))
}

fn create_meta(
    template: &str,
    parent: Option<&str>,
    content_file: &str,
) -> HashMap<String, String> {
    let mut meta = HashMap::new();
    meta.insert("template".to_string(), template.to_string());
    meta.insert("content_file".to_string(), content_file.to_string());

    if let Some(p) = parent {
        meta.insert("parent".to_string(), p.to_string());
    } else {
        meta.insert("parent".to_string(), "null".to_string());
    }

    meta
}

pub async fn render_html(
    State(_state): State<App>,
    Path(params): Path<HashMap<String, String>>,
    _request: Request<Body>,
) -> AppResult<Response> {
    let content_file = params
        .get("content_file")
        .ok_or_else(|| AppError::NotFound("Content file not specified".to_string()))?;

    let template_name = params
        .get("template_name")
        .unwrap_or(&"default".to_string())
        .clone();

    let data = parse_content_file(content_file)?;

    let mut html = format!(
        "<html><head><title>{}</title></head><body>",
        data.get("title").unwrap_or(&"Untitled".to_string())
    );

    html.push_str(&format!("<h1>Template: {}</h1>", template_name));

    for (key, value) in &data {
        html.push_str(&format!("<div><strong>{}:</strong> {}</div>", key, value));
    }

    html.push_str("</body></html>");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap())
}
