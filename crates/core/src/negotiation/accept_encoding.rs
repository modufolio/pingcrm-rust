use super::{parse_parameters, parse_quality, AcceptHeader};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AcceptEncoding {
    value: String,

    encoding: String,

    pub(crate) quality: f32,

    parameters: HashMap<String, String>,
}

impl AcceptEncoding {
    pub fn new(value: &str) -> Self {
        let value = value.trim();

        let (encoding, params_str) = if let Some(idx) = value.find(';') {
            (&value[..idx], &value[idx + 1..])
        } else {
            (value, "")
        };

        let encoding = encoding.trim();

        let mut parameters = parse_parameters(params_str);

        let quality = parameters
            .remove("q")
            .map(|q| parse_quality(&q))
            .unwrap_or(1.0);

        Self {
            value: value.to_string(),
            encoding: encoding.to_lowercase(),
            quality,
            parameters,
        }
    }

    pub fn is_wildcard(&self) -> bool {
        self.encoding == "*"
    }
}

impl AcceptHeader for AcceptEncoding {
    fn value(&self) -> &str {
        &self.value
    }

    fn get_type(&self) -> &str {
        &self.encoding
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
    fn test_simple_encoding() {
        let encoding = AcceptEncoding::new("gzip");
        assert_eq!(encoding.get_type(), "gzip");
        assert_eq!(encoding.quality(), 1.0);
    }

    #[test]
    fn test_encoding_with_quality() {
        let encoding = AcceptEncoding::new("gzip; q=0.8");
        assert_eq!(encoding.get_type(), "gzip");
        assert_eq!(encoding.quality(), 0.8);
    }

    #[test]
    fn test_wildcard_encoding() {
        let encoding = AcceptEncoding::new("*");
        assert!(encoding.is_wildcard());
    }

    #[test]
    fn test_encoding_case_insensitive() {
        let encoding = AcceptEncoding::new("GZIP");
        assert_eq!(encoding.get_type(), "gzip");
    }
}
