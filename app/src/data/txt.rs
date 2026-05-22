use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub struct Txt;

impl Txt {
    pub fn decode(string: &str) -> HashMap<String, String> {
        if string.is_empty() {
            return HashMap::new();
        }

        let mut string = string;

        if string.starts_with("\u{FEFF}") {
            string = &string[3..];
        }

        lazy_static::lazy_static! {
            static ref FIELD_SEPARATOR: Regex = Regex::new(r"\n----\s*\n*").unwrap();
        }

        let fields: Vec<&str> = FIELD_SEPARATOR.split(string).collect();

        let mut data = HashMap::new();

        for field in fields {
            if let Some(pos) = field.find(':') {
                let key = field[..pos].trim().to_lowercase();
                let key = key.replace(['-', ' '], "_");

                if key.is_empty() {
                    continue;
                }

                let value = field[pos + 1..].trim();

                lazy_static::lazy_static! {
                    static ref ESCAPED_DIVIDER: Regex = Regex::new(r"(?m)^\\----").unwrap();
                }

                let unescaped_value = ESCAPED_DIVIDER.replace_all(value, "----");

                data.insert(key, unescaped_value.to_string());
            }
        }

        data
    }

    pub fn encode(data: &HashMap<String, String>) -> String {
        let mut result = Vec::new();

        for (key, value) in data.iter() {
            if key.is_empty() || value.is_empty() {
                continue;
            }

            let formatted_key = Self::format_key(key);

            let encoded_value = Self::encode_value(value);

            let field_result = Self::encode_result(&formatted_key, &encoded_value);

            result.push(field_result);
        }

        result.join("\n\n----\n\n")
    }

    pub fn decode_to_json(string: &str) -> Value {
        let data = Self::decode(string);
        let mut map = Map::new();

        for (key, value) in data {
            map.insert(key, Value::String(value));
        }

        Value::Object(map)
    }

    fn format_key(key: &str) -> String {
        let slug = key.replace([' ', '_'], "-").to_lowercase();
        let mut chars = slug.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    fn encode_value(value: &str) -> String {
        lazy_static::lazy_static! {
            static ref DIVIDER_PATTERN: Regex = Regex::new(r"(?m)^----").unwrap();
        }

        DIVIDER_PATTERN.replace_all(value, "\\----").to_string()
    }

    fn encode_result(key: &str, value: &str) -> String {
        let value = value.trim();
        let mut result = format!("{}:", key);

        if value.contains('\n') {
            result.push_str("\n\n");
        } else {
            result.push(' ');
        }

        result.push_str(value);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_simple() {
        let content = "Title: Hello World\n----\nText: This is content";
        let data = Txt::decode(content);

        assert_eq!(data.get("title").unwrap(), "Hello World");
        assert_eq!(data.get("text").unwrap(), "This is content");
    }

    #[test]
    fn test_decode_multiline() {
        let content = "Title: Hello\n----\nText:\n\nThis is\nmultiline content";
        let data = Txt::decode(content);

        assert_eq!(data.get("title").unwrap(), "Hello");
        assert_eq!(data.get("text").unwrap(), "This is\nmultiline content");
    }

    #[test]
    fn test_decode_with_escaped_divider() {
        let content = "Title: Test\n----\nText:\n\nSome text\n\\----\nMore text";
        let data = Txt::decode(content);

        assert_eq!(data.get("text").unwrap(), "Some text\n----\nMore text");
    }

    #[test]
    fn test_encode_simple() {
        let mut data = HashMap::new();
        data.insert("title".to_string(), "Hello".to_string());
        data.insert("text".to_string(), "World".to_string());

        let encoded = Txt::encode(&data);

        assert!(encoded.contains("Title: Hello"));
        assert!(encoded.contains("Text: World"));
        assert!(encoded.contains("\n----\n"));
    }

    #[test]
    fn test_encode_multiline() {
        let mut data = HashMap::new();
        data.insert("title".to_string(), "Hello".to_string());
        data.insert("text".to_string(), "Line 1\nLine 2".to_string());

        let encoded = Txt::encode(&data);

        assert!(encoded.contains("Text:\n\nLine 1\nLine 2"));
    }

    #[test]
    fn test_roundtrip() {
        let original = "Title: Test Page\n----\nText:\n\nThis is content\nWith multiple lines";
        let decoded = Txt::decode(original);
        let encoded = Txt::encode(&decoded);
        let re_decoded = Txt::decode(&encoded);

        assert_eq!(decoded.get("title"), re_decoded.get("title"));
        assert_eq!(decoded.get("text"), re_decoded.get("text"));
    }
}
