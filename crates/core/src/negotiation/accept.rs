use super::{parse_parameters, parse_quality, AcceptHeader};
use std::collections::HashMap;

const SCORE_WILDCARD: i32 = 1000;
const SCORE_WILDCARD_SUBTYPE: i32 = 2000;
const SCORE_EXACT: i32 = 3000;

#[derive(Debug, Clone, PartialEq)]
pub struct Accept {
    value: String,

    media_type: String,

    pub(crate) quality: f32,

    parameters: HashMap<String, String>,

    main_type: String,

    sub_type: String,
}

impl Accept {
    pub fn new(value: &str) -> Self {
        let value = value.trim();

        let (media_type, params_str) = if let Some(idx) = value.find(';') {
            (&value[..idx], &value[idx + 1..])
        } else {
            (value, "")
        };

        let media_type = media_type.trim();

        let mut parameters = parse_parameters(params_str);

        let quality = parameters
            .remove("q")
            .map(|q| parse_quality(&q))
            .unwrap_or(1.0);

        let (main_type, sub_type) = if let Some((main, sub)) = media_type.split_once('/') {
            (main.trim().to_string(), sub.trim().to_string())
        } else {
            (media_type.to_string(), String::new())
        };

        Self {
            value: value.to_string(),
            media_type: media_type.to_string(),
            quality,
            parameters,
            main_type,
            sub_type,
        }
    }

    pub fn main_type(&self) -> &str {
        &self.main_type
    }

    pub fn sub_type(&self) -> &str {
        &self.sub_type
    }

    pub fn is_wildcard(&self) -> bool {
        self.main_type == "*" && self.sub_type == "*"
    }

    pub fn is_wildcard_subtype(&self) -> bool {
        self.sub_type == "*" && self.main_type != "*"
    }

    pub fn match_score(&self) -> i32 {
        if self.is_wildcard() {
            SCORE_WILDCARD
        } else if self.is_wildcard_subtype() {
            SCORE_WILDCARD_SUBTYPE
        } else {
            SCORE_EXACT
        }
    }
}

impl AcceptHeader for Accept {
    fn value(&self) -> &str {
        &self.value
    }

    fn get_type(&self) -> &str {
        &self.media_type
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
    fn test_simple_accept() {
        let accept = Accept::new("application/json");
        assert_eq!(accept.get_type(), "application/json");
        assert_eq!(accept.main_type(), "application");
        assert_eq!(accept.sub_type(), "json");
        assert_eq!(accept.quality(), 1.0);
    }

    #[test]
    fn test_accept_with_quality() {
        let accept = Accept::new("application/json; q=0.8");
        assert_eq!(accept.get_type(), "application/json");
        assert_eq!(accept.quality(), 0.8);
    }

    #[test]
    fn test_accept_with_parameters() {
        let accept = Accept::new("text/html; q=0.9; level=1");
        assert_eq!(accept.get_type(), "text/html");
        assert_eq!(accept.quality(), 0.9);
        assert_eq!(accept.parameter("level"), Some("1"));
    }

    #[test]
    fn test_wildcard_accept() {
        let accept = Accept::new("*/*");
        assert!(accept.is_wildcard());
        assert!(!accept.is_wildcard_subtype());
    }

    #[test]
    fn test_wildcard_subtype_accept() {
        let accept = Accept::new("text/*");
        assert!(!accept.is_wildcard());
        assert!(accept.is_wildcard_subtype());
        assert_eq!(accept.main_type(), "text");
    }
}
