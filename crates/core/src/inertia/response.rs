use axum::{
    body::Body,
    http::{header, HeaderMap, Request, StatusCode},
    response::Response,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tera::Context;

use super::template::render_template;
use super::vite::Vite;

const INERTIA_ENTRY: &str = "resources/js/app.js";

fn vite_tags_for_entry(entry: &str) -> String {
    match Vite::detect("public") {
        Ok(v) => v.tags(entry),
        Err(e) => {
            tracing::warn!("Vite asset resolver unavailable: {e}");
            "<!-- vite resolver unavailable -->".to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct InertiaVersion(String);

impl InertiaVersion {
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for InertiaVersion {
    fn default() -> Self {
        Self("1".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InertiaPage {
    pub component: String,
    pub props: Value,
    pub url: String,
    pub version: String,
}

pub struct InertiaResponse {
    component: String,
    props: Value,
    version: InertiaVersion,
    url: Option<String>,
}

impl InertiaResponse {
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            props: json!({}),
            version: InertiaVersion::default(),
            url: None,
        }
    }

    pub fn with_props(mut self, props: Value) -> Self {
        self.props = props;
        self
    }

    pub fn with_version(mut self, version: InertiaVersion) -> Self {
        self.version = version;
        self
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn render(self, request: &Request<Body>) -> Response {
        let is_inertia_request = request
            .headers()
            .get("X-Inertia")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "true")
            .unwrap_or(false);

        let url = self.url.unwrap_or_else(|| {
            let uri = request.uri();
            let path = uri.path();
            let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();
            format!("{}{}", path, query)
        });

        let page = InertiaPage {
            component: self.component.clone(),
            props: self.props.clone(),
            url,
            version: self.version.as_str().to_string(),
        };

        if is_inertia_request {
            InertiaResponse::render_json_static(request.headers(), page)
        } else {
            InertiaResponse::render_html_static(page)
        }
    }

    fn render_json_static(headers: &HeaderMap, page: InertiaPage) -> Response {
        let partial_component = headers
            .get("X-Inertia-Partial-Component")
            .and_then(|v| v.to_str().ok());

        let partial_data = headers
            .get("X-Inertia-Partial-Data")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').map(|s| s.trim()).collect::<Vec<_>>());

        let mut response_page = page;

        if let (Some(component), Some(only)) = (partial_component, partial_data) {
            if component == response_page.component {
                if let Value::Object(ref mut props_map) = response_page.props {
                    props_map.retain(|key, _| only.contains(&key.as_str()));
                }
            }
        }

        let json_body = serde_json::to_string(&response_page).unwrap_or_default();

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-Inertia", "true")
            .body(Body::from(json_body))
            .unwrap()
    }

    fn render_html_static(page: InertiaPage) -> Response {
        let mut context = Context::new();

        let page_json = serde_json::to_string(&page).unwrap_or_default();

        let page_json_escaped = page_json
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#039;");

        context.insert("page", &page_json_escaped);

        context.insert("vite_tags", &vite_tags_for_entry(INERTIA_ENTRY));

        match render_template("inertia.html", &context) {
            Ok(html) => Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(html))
                .unwrap(),
            Err(e) => {
                tracing::error!("Failed to render Inertia template: {}", e);
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(format!("Template rendering error: {}", e)))
                    .unwrap()
            }
        }
    }
}
