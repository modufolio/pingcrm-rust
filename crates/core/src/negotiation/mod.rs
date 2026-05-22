mod accept;
mod accept_charset;
mod accept_encoding;
mod accept_language;
mod charset_negotiator;
mod encoding_negotiator;
mod language_negotiator;
mod negotiator;

pub use accept::Accept;
pub use accept_charset::AcceptCharset;
pub use accept_encoding::AcceptEncoding;
pub use accept_language::AcceptLanguage;
pub use charset_negotiator::CharsetNegotiator;
pub use encoding_negotiator::EncodingNegotiator;
pub use language_negotiator::LanguageNegotiator;
pub use negotiator::Negotiator;

pub trait AcceptHeader: Clone + std::fmt::Debug {
    fn value(&self) -> &str;

    fn get_type(&self) -> &str;

    fn quality(&self) -> f32;

    fn parameter(&self, name: &str) -> Option<&str>;

    fn parameters(&self) -> &std::collections::HashMap<String, String>;
}

fn parse_quality(value: &str) -> f32 {
    value.parse::<f32>().unwrap_or(1.0).clamp(0.0, 1.0)
}

fn parse_parameters(params_str: &str) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();

    for param in params_str.split(';') {
        let param = param.trim();
        if let Some((key, value)) = param.split_once('=') {
            params.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_quality() {
        assert_eq!(parse_quality("1.0"), 1.0);
        assert_eq!(parse_quality("0.8"), 0.8);
        assert_eq!(parse_quality("0.5"), 0.5);
        assert_eq!(parse_quality("0"), 0.0);
        assert_eq!(parse_quality("invalid"), 1.0);
        assert_eq!(parse_quality("1.5"), 1.0);
        assert_eq!(parse_quality("-0.5"), 0.0);
    }

    #[test]
    fn test_parse_parameters() {
        let params = parse_parameters("q=0.8; level=1; charset=utf-8");
        assert_eq!(params.get("q"), Some(&"0.8".to_string()));
        assert_eq!(params.get("level"), Some(&"1".to_string()));
        assert_eq!(params.get("charset"), Some(&"utf-8".to_string()));
    }
}
