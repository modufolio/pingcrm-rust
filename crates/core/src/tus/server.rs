use axum::{
    body::Body,
    http::{HeaderMap, Method, StatusCode},
    response::Response,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{
    checksum::ChecksumInfo,
    metadata::UploadMetadata,
    storage::{TusStorage, UploadContainer},
    CHECKSUM_ALGORITHMS, TUS_EXTENSIONS, TUS_VERSION,
};

pub struct TusServer<S: TusStorage> {
    storage: Arc<RwLock<S>>,
    max_size: u64,
    #[allow(dead_code)]
    chunk_size: usize,
    api_path: String,
}

impl<S: TusStorage + 'static> TusServer<S> {
    pub fn new(storage: S, max_size: Option<u64>, chunk_size: Option<usize>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            max_size: max_size.unwrap_or(100 * 1024 * 1024),
            chunk_size: chunk_size.unwrap_or(5 * 1024 * 1024),
            api_path: "/tus".to_string(),
        }
    }

    pub async fn handle_request(
        &self,
        method: Method,
        path: Option<String>,
        headers: HeaderMap,
        body: Option<Vec<u8>>,
    ) -> Response {
        if let Some(tus_header) = headers.get("tus-resumable") {
            if tus_header.to_str().unwrap_or("") != TUS_VERSION {
                return self.error_response(
                    StatusCode::PRECONDITION_FAILED,
                    "Unsupported TUS protocol version",
                );
            }
        }

        match method {
            Method::OPTIONS => self.handle_options().await,
            Method::POST => self.handle_creation(headers, body).await,
            Method::HEAD => {
                if let Some(filename) = path {
                    self.handle_head(&filename, headers).await
                } else {
                    self.error_response(StatusCode::BAD_REQUEST, "Missing filename")
                }
            }
            Method::PATCH => {
                if let Some(filename) = path {
                    self.handle_patch(&filename, headers, body).await
                } else {
                    self.error_response(StatusCode::BAD_REQUEST, "Missing filename")
                }
            }
            Method::DELETE => {
                if let Some(filename) = path {
                    self.handle_delete(&filename).await
                } else {
                    self.error_response(StatusCode::BAD_REQUEST, "Missing filename")
                }
            }
            _ => self.error_response(StatusCode::METHOD_NOT_ALLOWED, "Method not allowed"),
        }
    }

    async fn handle_options(&self) -> Response {
        let storage = self.storage.read().await;
        let mut extensions = TUS_EXTENSIONS.to_vec();

        if storage.supports_cross_check() {
            extensions.push("crosscheck");
        }

        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("Tus-Resumable", TUS_VERSION)
            .header("Tus-Version", TUS_VERSION)
            .header("Tus-Extension", extensions.join(","))
            .header("Tus-Max-Size", self.max_size.to_string())
            .header("Tus-Checksum-Algorithm", CHECKSUM_ALGORITHMS.join(","))
            .header("Cache-Control", "no-store")
            .header("Access-Control-Allow-Origin", "*")
            .header(
                "Access-Control-Allow-Methods",
                "OPTIONS,POST,HEAD,PATCH,DELETE",
            )
            .header("Access-Control-Allow-Headers", "*")
            .header("Access-Control-Max-Age", "86400")
            .body(Body::empty())
            .unwrap()
    }

    async fn handle_creation(&self, headers: HeaderMap, body: Option<Vec<u8>>) -> Response {
        let storage = self.storage.read().await;

        let upload_length = headers
            .get("upload-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let defer_length = headers
            .get("upload-defer-length")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "1")
            .unwrap_or(false);

        let metadata = headers
            .get("upload-metadata")
            .and_then(|v| v.to_str().ok())
            .map(UploadMetadata::parse)
            .unwrap_or_default();

        let filename = match metadata.filename() {
            Some(f) => self.sanitize_filename(f),
            None => {
                return self.error_response(
                    StatusCode::BAD_REQUEST,
                    "Missing filename in Upload-Metadata",
                )
            }
        };

        if let Some(length) = upload_length {
            if length > self.max_size {
                return self.error_response(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "File size exceeds maximum limit",
                );
            }
        }

        if storage.exists(&filename).await.unwrap_or(false)
            || storage.container_exists(&filename).await.unwrap_or(false)
        {
            if let Ok(Some(container)) = storage.container_fetch(&filename).await {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("Tus-Resumable", TUS_VERSION)
                    .header("Location", &container.location)
                    .header("Cache-Control", "no-store")
                    .body(Body::empty())
                    .unwrap();
            }

            return self.error_response(
                StatusCode::CONFLICT,
                "File already exists without valid container",
            );
        }

        let now = chrono::Utc::now();
        let container = UploadContainer {
            length: upload_length,
            deferred: defer_length,
            metadata: metadata.metadata.clone(),
            is_partial: false,
            partials: vec![],
            created_at: now.to_rfc3339(),
            expires_at: (now + chrono::Duration::days(1)).to_rfc3339(),
            location: format!("{}/{}", self.api_path, filename),
            checksum: None,
        };

        if let Err(e) = storage.container_create(&filename, &container).await {
            return self.error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }

        if let Some(data) = body {
            if !data.is_empty() {
                if let Err(e) = storage.create(&filename).await {
                    return self.error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                }

                if let Err(e) = storage.append(&filename, &data).await {
                    let _ = storage.delete(&filename).await;
                    let _ = storage.container_delete(&filename).await;
                    return self.error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
                }

                if let Some(length) = upload_length {
                    if let Ok(size) = storage.get_size(&filename).await {
                        if size > length {
                            let _ = storage.delete(&filename).await;
                            let _ = storage.container_delete(&filename).await;
                            return self.error_response(
                                StatusCode::PAYLOAD_TOO_LARGE,
                                "Upload exceeds declared length",
                            );
                        }
                    }
                }
            }
        } else if !defer_length {
            if let Err(e) = storage.create(&filename).await {
                return self.error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
            }
        }

        Response::builder()
            .status(StatusCode::CREATED)
            .header("Tus-Resumable", TUS_VERSION)
            .header("Location", &container.location)
            .header("Cache-Control", "no-store")
            .body(Body::empty())
            .unwrap()
    }

    async fn handle_head(&self, filename: &str, _headers: HeaderMap) -> Response {
        let storage = self.storage.read().await;

        let exists = storage.exists(filename).await.unwrap_or(false);
        let container_exists = storage.container_exists(filename).await.unwrap_or(false);

        if !exists && !container_exists {
            return self.error_response(StatusCode::NOT_FOUND, "Not found");
        }

        let container = match storage.container_fetch(filename).await {
            Ok(Some(c)) => c,
            _ => return self.error_response(StatusCode::NOT_FOUND, "Container not found"),
        };

        let offset = storage.get_size(filename).await.unwrap_or(0);

        let mut response = Response::builder()
            .status(StatusCode::OK)
            .header("Tus-Resumable", TUS_VERSION)
            .header("Upload-Offset", offset.to_string())
            .header("Cache-Control", "no-store");

        if let Some(length) = container.length {
            response = response.header("Upload-Length", length.to_string());
        }

        response.body(Body::empty()).unwrap()
    }

    async fn handle_patch(
        &self,
        filename: &str,
        headers: HeaderMap,
        body: Option<Vec<u8>>,
    ) -> Response {
        let storage = self.storage.read().await;

        if !storage.container_exists(filename).await.unwrap_or(false) {
            return self.error_response(StatusCode::NOT_FOUND, "Not found");
        }

        let container = match storage.container_fetch(filename).await {
            Ok(Some(c)) => c,
            _ => return self.error_response(StatusCode::NOT_FOUND, "Container not found"),
        };

        let offset = headers
            .get("upload-offset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let current_size = storage.get_size(filename).await.unwrap_or(0);
        if offset != current_size {
            return Response::builder()
                .status(StatusCode::CONFLICT)
                .header("Tus-Resumable", TUS_VERSION)
                .header("Upload-Offset", current_size.to_string())
                .body(Body::from("Invalid offset"))
                .unwrap();
        }

        let data = match body {
            Some(d) if !d.is_empty() => d,
            _ => return self.error_response(StatusCode::BAD_REQUEST, "No upload data provided"),
        };

        if !storage.exists(filename).await.unwrap_or(false) {
            if let Err(e) = storage.create(filename).await {
                return self.error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
            }
        }

        if let Some(checksum_header) = headers.get("upload-checksum") {
            if let Some(checksum) = checksum_header.to_str().ok().and_then(ChecksumInfo::parse) {
                if !checksum.verify(&data) {
                    let _ = storage.delete(filename).await;
                    let _ = storage.container_delete(filename).await;
                    return self
                        .error_response(StatusCode::from_u16(460).unwrap(), "Checksum mismatch");
                }
            }
        }

        if let Err(e) = storage.append(filename, &data).await {
            let _ = storage.delete(filename).await;
            let _ = storage.container_delete(filename).await;
            return self.error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }

        let new_offset = storage.get_size(filename).await.unwrap_or(0);

        if let Some(length) = container.length {
            if new_offset > length {
                let _ = storage.delete(filename).await;
                let _ = storage.container_delete(filename).await;
                return self.error_response(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "Upload exceeds declared length",
                );
            }

            if new_offset >= length {
                let _ = storage.container_delete(filename).await;
            }
        }

        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("Tus-Resumable", TUS_VERSION)
            .header("Upload-Offset", new_offset.to_string())
            .header("Cache-Control", "no-store")
            .body(Body::empty())
            .unwrap()
    }

    async fn handle_delete(&self, filename: &str) -> Response {
        let storage = self.storage.read().await;

        let exists = storage.exists(filename).await.unwrap_or(false);
        let container_exists = storage.container_exists(filename).await.unwrap_or(false);

        if !exists && !container_exists {
            return self.error_response(StatusCode::NOT_FOUND, "Not found");
        }

        if exists {
            let _ = storage.delete(filename).await;
        }

        if container_exists {
            let _ = storage.container_delete(filename).await;
        }

        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("Tus-Resumable", TUS_VERSION)
            .header("Cache-Control", "no-store")
            .body(Body::empty())
            .unwrap()
    }

    fn sanitize_filename(&self, filename: &str) -> String {
        let sanitized = filename
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        let sanitized = sanitized.trim_start_matches('.');

        if sanitized.len() >= 3 {
            sanitized.to_string()
        } else {
            format!("file_{}", uuid::Uuid::new_v4())
        }
    }

    fn error_response(&self, status: StatusCode, message: &str) -> Response {
        let body = serde_json::json!({
            "status": "error",
            "message": message,
        });

        Response::builder()
            .status(status)
            .header("Tus-Resumable", TUS_VERSION)
            .header("Cache-Control", "no-store")
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }
}
