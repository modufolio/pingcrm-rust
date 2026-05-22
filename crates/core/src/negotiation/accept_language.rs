use super::{parse_parameters, parse_quality, AcceptHeader};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AcceptLanguage {
    value: String,

    language: String,

    pub(crate) quality: f32,

    parameters: HashMap<String, String>,

    primary: String,

    sub: Option<String>,
}

impl AcceptLanguage {
    pub fn new(value: &str) -> Self {
        let value = value.trim();

        let (language, params_str) = if let Some(idx) = value.find(';') {
            (&value[..idx], &value[idx + 1..])
        } else {
            (value, "")
        };

        let language = language.trim();

        let mut parameters = parse_parameters(params_str);

        let quality = parameters
            .remove("q")
            .map(|q| parse_quality(&q))
            .unwrap_or(1.0);

        let (primary, sub) = if let Some((prim, sub_tag)) = language.split_once('-') {
            (prim.to_lowercase(), Some(sub_tag.to_string()))
        } else {
            (language.to_lowercase(), None)
        };

        Self {
            value: value.to_string(),
            language: language.to_string(),
            quality,
            parameters,
            primary,
            sub,
        }
    }

    pub fn primary(&self) -> &str {
        &self.primary
    }

    pub fn sub(&self) -> Option<&str> {
        self.sub.as_deref()
    }

    pub fn is_wildcard(&self) -> bool {
        self.primary == "*"
    }
}

impl AcceptHeader for AcceptLanguage {
    fn value(&self) -> &str {
        &self.value
    }

    fn get_type(&self) -> &str {
        &self.language
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
    fn test_simple_language() {
        let lang = AcceptLanguage::new("en");
        assert_eq!(lang.get_type(), "en");
        assert_eq!(lang.primary(), "en");
        assert_eq!(lang.sub(), None);
        assert_eq!(lang.quality(), 1.0);
    }

    #[test]
    fn test_language_with_region() {
        let lang = AcceptLanguage::new("en-US");
        assert_eq!(lang.get_type(), "en-US");
        assert_eq!(lang.primary(), "en");
        assert_eq!(lang.sub(), Some("US"));
    }

    #[test]
    fn test_language_with_quality() {
        let lang = AcceptLanguage::new("en-US; q=0.9");
        assert_eq!(lang.quality(), 0.9);
    }

    #[test]
    fn test_wildcard_language() {
        let lang = AcceptLanguage::new("*");
        assert!(lang.is_wildcard());
    }
}
