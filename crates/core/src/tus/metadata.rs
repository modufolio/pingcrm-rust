use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct UploadMetadata {
    pub metadata: HashMap<String, String>,
}

impl UploadMetadata {
    pub fn parse(header_value: &str) -> Self {
        let mut metadata = HashMap::new();

        if header_value.trim().is_empty() {
            return Self { metadata };
        }

        for item in header_value.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }

            if let Some((key, value)) = item.split_once(' ') {
                if let Ok(decoded) = STANDARD.decode(value) {
                    if let Ok(decoded_str) = String::from_utf8(decoded) {
                        metadata.insert(key.to_string(), decoded_str);
                    }
                }
            }
        }

        Self { metadata }
    }

    pub fn filename(&self) -> Option<&String> {
        self.metadata.get("filename")
    }

    pub fn format(&self) -> String {
        self.metadata
            .iter()
            .map(|(k, v)| format!("{} {}", k, STANDARD.encode(v)))
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metadata() {
        let input = "filename d29ybGRfZG9taW5hdGlvbl9wbGFuLnBkZg==,filetype YXBwbGljYXRpb24vcGRm";
        let metadata = UploadMetadata::parse(input);

        assert_eq!(
            metadata.filename(),
            Some(&"world_domination_plan.pdf".to_string())
        );
        assert_eq!(
            metadata.metadata.get("filetype"),
            Some(&"application/pdf".to_string())
        );
    }

    #[test]
    fn test_format_metadata() {
        let mut metadata = UploadMetadata::default();
        metadata
            .metadata
            .insert("filename".to_string(), "test.pdf".to_string());
        metadata
            .metadata
            .insert("filetype".to_string(), "application/pdf".to_string());

        let formatted = metadata.format();
        assert!(formatted.contains("filename"));
        assert!(formatted.contains("filetype"));
    }
}
