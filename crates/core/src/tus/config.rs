use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TusConfig {
    pub server: TusServerConfig,
    pub storage: TusStorageConfig,
    pub protocol: TusProtocolConfig,
    pub cors: TusCorsConfig,
    pub database: TusDatabaseConfig,
    pub security: TusSecurityConfig,
    pub logging: TusLoggingConfig,
    pub performance: TusPerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TusServerConfig {
    pub api_path: String,
    pub max_size: u64,
    pub chunk_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TusStorageConfig {
    pub upload_dir: String,
    pub container_dir: String,
    pub auto_cleanup: bool,
    pub expiration_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TusProtocolConfig {
    pub version: String,
    pub extensions: Vec<String>,
    pub checksum_algorithms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TusCorsConfig {
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TusDatabaseConfig {
    pub auto_save: bool,
    pub default_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TusSecurityConfig {
    pub sanitize_filenames: bool,
    pub allowed_extensions: Vec<String>,
    pub blocked_extensions: Vec<String>,
    pub validate_mime_types: bool,
    pub allowed_mime_types: Vec<String>,
    pub blocked_mime_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TusLoggingConfig {
    pub verbose: bool,
    pub log_progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TusPerformanceConfig {
    pub concurrent_uploads: bool,
    pub max_concurrent_uploads: u32,
    pub rate_limiting: bool,
    pub max_upload_rate: u64,
}

impl TusConfig {
    pub fn load() -> anyhow::Result<Self> {
        let mut config = Self::default();

        let config_path = PathBuf::from("config/tus.toml");
        if config_path.exists() {
            let toml_content = std::fs::read_to_string(&config_path)?;
            config = toml::from_str(&toml_content)?;
            tracing::info!("Loaded TUS configuration from config/tus.toml");
        } else {
            tracing::warn!("TUS config file not found at config/tus.toml, using defaults");
        }

        if let Ok(val) = std::env::var("TUS_API_PATH") {
            config.server.api_path = val;
        }
        if let Ok(val) = std::env::var("TUS_MAX_SIZE") {
            config.server.max_size = val.parse()?;
        }
        if let Ok(val) = std::env::var("TUS_CHUNK_SIZE") {
            config.server.chunk_size = val.parse()?;
        }
        if let Ok(val) = std::env::var("TUS_UPLOAD_DIR") {
            config.storage.upload_dir = val;
        }
        if let Ok(val) = std::env::var("TUS_CONTAINER_DIR") {
            config.storage.container_dir = val;
        }
        if let Ok(val) = std::env::var("TUS_AUTO_CLEANUP") {
            config.storage.auto_cleanup = val.parse()?;
        }
        if let Ok(val) = std::env::var("TUS_EXPIRATION_TIME") {
            config.storage.expiration_time = val.parse()?;
        }
        if let Ok(val) = std::env::var("TUS_CORS_ENABLED") {
            config.cors.enabled = val.parse()?;
        }
        if let Ok(val) = std::env::var("TUS_CORS_ORIGINS") {
            config.cors.allowed_origins = val.split(',').map(|s| s.trim().to_string()).collect();
        }
        if let Ok(val) = std::env::var("TUS_AUTO_SAVE") {
            config.database.auto_save = val.parse()?;
        }
        if let Ok(val) = std::env::var("TUS_VERBOSE") {
            config.logging.verbose = val.parse()?;
        }

        Ok(config)
    }

    pub fn upload_dir(&self) -> PathBuf {
        PathBuf::from(&self.storage.upload_dir)
    }

    pub fn container_dir(&self) -> PathBuf {
        PathBuf::from(&self.storage.container_dir)
    }
}

impl Default for TusConfig {
    fn default() -> Self {
        Self {
            server: TusServerConfig {
                api_path: "/tus".to_string(),
                max_size: 100 * 1024 * 1024,
                chunk_size: 5 * 1024 * 1024,
            },
            storage: TusStorageConfig {
                upload_dir: "public/uploads/tus".to_string(),
                container_dir: "public/uploads/tus/.containers".to_string(),
                auto_cleanup: true,
                expiration_time: 86400,
            },
            protocol: TusProtocolConfig {
                version: "1.0.0".to_string(),
                extensions: vec![
                    "creation".to_string(),
                    "checksum".to_string(),
                    "concatenation".to_string(),
                    "termination".to_string(),
                    "creation-with-upload".to_string(),
                ],
                checksum_algorithms: vec![
                    "md5".to_string(),
                    "sha1".to_string(),
                    "sha256".to_string(),
                    "sha512".to_string(),
                ],
            },
            cors: TusCorsConfig {
                enabled: true,
                allowed_origins: vec!["*".to_string()],
                allowed_methods: vec![
                    "OPTIONS".to_string(),
                    "POST".to_string(),
                    "HEAD".to_string(),
                    "PATCH".to_string(),
                    "DELETE".to_string(),
                ],
                allowed_headers: vec!["*".to_string()],
                max_age: 86400,
            },
            database: TusDatabaseConfig {
                auto_save: true,
                default_public: true,
            },
            security: TusSecurityConfig {
                sanitize_filenames: true,
                allowed_extensions: vec![],
                blocked_extensions: vec![
                    "exe".to_string(),
                    "bat".to_string(),
                    "cmd".to_string(),
                    "sh".to_string(),
                    "php".to_string(),
                    "phtml".to_string(),
                ],
                validate_mime_types: true,
                allowed_mime_types: vec![],
                blocked_mime_types: vec![
                    "application/x-executable".to_string(),
                    "application/x-msdownload".to_string(),
                    "application/x-dosexec".to_string(),
                ],
            },
            logging: TusLoggingConfig {
                verbose: false,
                log_progress: false,
            },
            performance: TusPerformanceConfig {
                concurrent_uploads: true,
                max_concurrent_uploads: 5,
                rate_limiting: false,
                max_upload_rate: 0,
            },
        }
    }
}
