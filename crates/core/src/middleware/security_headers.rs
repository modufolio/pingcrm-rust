use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeadersConfig {
    pub x_frame_options: XFrameOptionsConfig,

    pub x_content_type_options: XContentTypeOptionsConfig,

    pub x_xss_protection: XXssProtectionConfig,

    pub referrer_policy: ReferrerPolicyConfig,

    pub hsts: HstsConfig,

    pub csp: CspConfig,

    pub permissions_policy: PermissionsPolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XFrameOptionsConfig {
    pub enabled: bool,

    pub value: XFrameOptionsValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum XFrameOptionsValue {
    Deny,
    SameOrigin,
}

impl std::fmt::Display for XFrameOptionsValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deny => write!(f, "DENY"),
            Self::SameOrigin => write!(f, "SAMEORIGIN"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XContentTypeOptionsConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XXssProtectionConfig {
    pub enabled: bool,
    pub mode_block: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferrerPolicyConfig {
    pub enabled: bool,
    pub policy: ReferrerPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReferrerPolicy {
    NoReferrer,
    NoReferrerWhenDowngrade,
    Origin,
    OriginWhenCrossOrigin,
    SameOrigin,
    StrictOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

impl std::fmt::Display for ReferrerPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::NoReferrer => "no-referrer",
            Self::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
            Self::Origin => "origin",
            Self::OriginWhenCrossOrigin => "origin-when-cross-origin",
            Self::SameOrigin => "same-origin",
            Self::StrictOrigin => "strict-origin",
            Self::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
            Self::UnsafeUrl => "unsafe-url",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HstsConfig {
    pub enabled: bool,

    pub max_age: u32,
    pub include_sub_domains: bool,

    pub preload: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CspConfig {
    pub enabled: bool,

    pub report_only: bool,

    pub report_uri: Option<String>,

    pub directives: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsPolicyConfig {
    pub enabled: bool,

    pub directives: HashMap<String, Vec<String>>,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            x_frame_options: XFrameOptionsConfig {
                enabled: true,
                value: XFrameOptionsValue::Deny,
            },
            x_content_type_options: XContentTypeOptionsConfig { enabled: true },
            x_xss_protection: XXssProtectionConfig {
                enabled: true,
                mode_block: true,
            },
            referrer_policy: ReferrerPolicyConfig {
                enabled: true,
                policy: ReferrerPolicy::NoReferrerWhenDowngrade,
            },
            hsts: HstsConfig {
                enabled: true,
                max_age: 31536000,
                include_sub_domains: true,
                preload: false,
            },
            csp: CspConfig {
                enabled: true,
                report_only: false,
                report_uri: None,
                directives: Self::default_csp_directives(),
            },
            permissions_policy: PermissionsPolicyConfig {
                enabled: true,
                directives: Self::default_permissions_directives(),
            },
        }
    }
}

impl SecurityHeadersConfig {
    fn default_csp_directives() -> HashMap<String, Vec<String>> {
        let mut directives = HashMap::new();
        directives.insert("default-src".to_string(), vec!["self".to_string()]);
        directives.insert(
            "script-src".to_string(),
            vec!["self".to_string(), "unsafe-inline".to_string()],
        );
        directives.insert(
            "style-src".to_string(),
            vec!["self".to_string(), "unsafe-inline".to_string()],
        );
        directives.insert(
            "img-src".to_string(),
            vec![
                "self".to_string(),
                "data:".to_string(),
                "https:".to_string(),
            ],
        );
        directives.insert(
            "font-src".to_string(),
            vec!["self".to_string(), "data:".to_string()],
        );
        directives.insert("connect-src".to_string(), vec!["self".to_string()]);
        directives.insert("frame-ancestors".to_string(), vec!["none".to_string()]);
        directives.insert("base-uri".to_string(), vec!["self".to_string()]);
        directives.insert("form-action".to_string(), vec!["self".to_string()]);
        directives
    }

    fn default_permissions_directives() -> HashMap<String, Vec<String>> {
        let mut directives = HashMap::new();

        directives.insert("geolocation".to_string(), vec![]);
        directives.insert("microphone".to_string(), vec![]);
        directives.insert("camera".to_string(), vec![]);
        directives.insert("payment".to_string(), vec![]);
        directives.insert("usb".to_string(), vec![]);
        directives.insert("magnetometer".to_string(), vec![]);
        directives.insert("gyroscope".to_string(), vec![]);
        directives.insert("accelerometer".to_string(), vec![]);
        directives
    }

    fn build_csp_header(&self) -> String {
        let mut directives = Vec::new();

        for (directive, sources) in &self.csp.directives {
            if sources.is_empty() {
                continue;
            }

            let formatted_sources: Vec<String> = sources
                .iter()
                .map(|source| {
                    if matches!(
                        source.as_str(),
                        "self"
                            | "none"
                            | "unsafe-inline"
                            | "unsafe-eval"
                            | "strict-dynamic"
                            | "unsafe-hashes"
                    ) {
                        format!("'{}'", source)
                    } else {
                        source.clone()
                    }
                })
                .collect();

            directives.push(format!("{} {}", directive, formatted_sources.join(" ")));
        }

        let mut csp = directives.join("; ");

        if let Some(report_uri) = &self.csp.report_uri {
            csp.push_str(&format!("; report-uri {}", report_uri));
        }

        csp
    }

    fn build_permissions_policy_header(&self) -> String {
        let mut policies = Vec::new();

        for (feature, allowlist) in &self.permissions_policy.directives {
            if allowlist.is_empty() {
                policies.push(format!("{}=()", feature));
            } else {
                let formatted: Vec<String> = allowlist
                    .iter()
                    .map(|origin| {
                        if origin == "self" {
                            "self".to_string()
                        } else {
                            format!("\"{}\"", origin)
                        }
                    })
                    .collect();
                policies.push(format!("{}=({})", feature, formatted.join(" ")));
            }
        }

        policies.join(", ")
    }

    fn is_https(request: &Request) -> bool {
        request.uri().scheme_str() == Some("https")
    }

    pub fn build_headers(&self, request: &Request) -> Vec<(&'static str, String)> {
        let mut headers = Vec::new();

        if self.x_frame_options.enabled {
            headers.push(("X-Frame-Options", self.x_frame_options.value.to_string()));
        }

        if self.x_content_type_options.enabled {
            headers.push(("X-Content-Type-Options", "nosniff".to_string()));
        }

        if self.x_xss_protection.enabled {
            let value = if self.x_xss_protection.mode_block {
                "1; mode=block"
            } else {
                "1"
            };
            headers.push(("X-XSS-Protection", value.to_string()));
        }

        if self.referrer_policy.enabled {
            headers.push(("Referrer-Policy", self.referrer_policy.policy.to_string()));
        }

        if self.hsts.enabled && Self::is_https(request) {
            let mut hsts = format!("max-age={}", self.hsts.max_age);
            if self.hsts.include_sub_domains {
                hsts.push_str("; includeSubDomains");
            }
            if self.hsts.preload {
                hsts.push_str("; preload");
            }
            headers.push(("Strict-Transport-Security", hsts));
        }

        if self.csp.enabled {
            let csp = self.build_csp_header();
            let header_name = if self.csp.report_only {
                "Content-Security-Policy-Report-Only"
            } else {
                "Content-Security-Policy"
            };
            headers.push((header_name, csp));
        }

        if self.permissions_policy.enabled {
            headers.push(("Permissions-Policy", self.build_permissions_policy_header()));
        }

        headers
    }
}

pub async fn enhanced_security_headers_middleware(
    config: SecurityHeadersConfig,
    request: Request,
    next: Next,
) -> Response {
    let headers_to_add = config.build_headers(&request);

    let mut response = next.run(request).await;

    for (name, value) in headers_to_add {
        if let Ok(header_value) = HeaderValue::from_str(&value) {
            response.headers_mut().insert(name, header_value);
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;

    #[test]
    fn test_default_config() {
        let config = SecurityHeadersConfig::default();
        assert!(config.x_frame_options.enabled);
        assert!(config.hsts.enabled);
        assert_eq!(config.hsts.max_age, 31536000);
    }

    #[test]
    fn test_csp_header_building() {
        let config = SecurityHeadersConfig::default();
        let csp = config.build_csp_header();

        assert!(csp.contains("default-src 'self'"));
        assert!(csp.contains("script-src 'self' 'unsafe-inline'"));
    }

    #[test]
    fn test_permissions_policy_building() {
        let config = SecurityHeadersConfig::default();
        let policy = config.build_permissions_policy_header();

        assert!(policy.contains("geolocation=()"));
        assert!(policy.contains("camera=()"));
    }

    #[test]
    fn test_hsts_header_building() {
        let mut config = SecurityHeadersConfig::default();
        config.hsts.include_sub_domains = true;
        config.hsts.preload = true;

        let request = Request::builder()
            .uri("https://example.com")
            .body(Body::empty())
            .unwrap();

        let headers = config.build_headers(&request);

        let hsts = headers
            .iter()
            .find(|(name, _)| *name == "Strict-Transport-Security")
            .map(|(_, value)| value);

        assert!(hsts.is_some());
        let hsts_value = hsts.unwrap();
        assert!(hsts_value.contains("max-age=31536000"));
        assert!(hsts_value.contains("includeSubDomains"));
        assert!(hsts_value.contains("preload"));
    }
}
