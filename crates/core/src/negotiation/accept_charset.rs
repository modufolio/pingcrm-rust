use super::{parse_parameters, parse_quality, AcceptHeader};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AcceptCharset {
    value: String,

    charset: String,

    pub(crate) quality: f32,

    parameters: HashMap<String, String>,
}

impl AcceptCharset {
    pub fn new(value: &str) -> Self {
        let value = value.trim();

        let (charset, params_str) = if let Some(idx) = value.find(';') {
            (&value[..idx], &value[idx + 1..])
        } else {
            (value, "")
        };

        let charset = charset.trim();

        let mut parameters = parse_parameters(params_str);

        let quality = parameters
            .remove("q")
            .map(|q| parse_quality(&q))
            .unwrap_or(1.0);

        Self {
            value: value.to_string(),
            charset: charset.to_lowercase(),
            quality,
            parameters,
        }
    }

    pub fn is_wildcard(&self) -> bool {
        self.charset == "*"
    }
}

impl AcceptHeader for AcceptCharset {
    fn value(&self) -> &str {
        &self.value
    }

    fn get_type(&self) -> &str {
        &self.charset
    }

    fn quality(&self) -> f32 {
        self.quality
    }

    fn parameter(&self, name: &str) -> Option<&str> {
        self.parameters.get(name).map(|s| s.as_str())
    }

    fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_charset() {
        let charset = AcceptCharset::new("utf-8");
        assert_eq!(charset.get_type(), "utf-8");
        assert_eq!(charset.quality(), 1.0);
    }

    #[test]
    fn test_charset_with_quality() {
        let charset = AcceptCharset::new("utf-8; q=0.9");
        assert_eq!(charset.get_type(), "utf-8");
        assert_eq!(charset.quality(), 0.9);
    }

    #[test]
    fn test_wildcard_charset() {
        let charset = AcceptCharset::new("*");
        assert!(charset.is_wildcard());
    }

    #[test]
    fn test_charset_case_insensitive() {
        let charset = AcceptCharset::new("UTF-8");
        assert_eq!(charset.get_type(), "utf-8");
    }
}
