use super::{AcceptEncoding, AcceptHeader};

#[derive(Debug, Default)]
pub struct EncodingNegotiator;

impl EncodingNegotiator {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_header(&self, header: &str) -> Vec<AcceptEncoding> {
        let mut encodings: Vec<AcceptEncoding> = header
            .split(',')
            .map(|s| AcceptEncoding::new(s.trim()))
            .collect();

        encodings.sort_by(|a, b| b.quality().partial_cmp(&a.quality()).unwrap());

        encodings
    }

    pub fn negotiate(&self, header: &str, priorities: &[&str]) -> Option<AcceptEncoding> {
        let header_encodings = self.parse_header(header);
        let priority_encodings: Vec<AcceptEncoding> =
            priorities.iter().map(|p| AcceptEncoding::new(p)).collect();

        self.find_best_match(&header_encodings, &priority_encodings)
    }

    fn find_best_match(
        &self,
        encodings: &[AcceptEncoding],
        priorities: &[AcceptEncoding],
    ) -> Option<AcceptEncoding> {
        for encoding in encodings {
            if encoding.quality() == 0.0 {
                continue;
            }

            for priority in priorities {
                if self.matches(encoding, priority) {
                    let combined_quality = encoding.quality() * priority.quality();
                    let mut matched = priority.clone();
                    matched.quality = combined_quality;
                    return Some(matched);
                }
            }
        }

        None
    }

    fn matches(&self, encoding: &AcceptEncoding, priority: &AcceptEncoding) -> bool {
        if encoding.is_wildcard() {
            return true;
        }

        encoding
            .get_type()
            .eq_ignore_ascii_case(priority.get_type())
    }

    pub fn get_best(&self, header: &str, priorities: &[&str]) -> Option<AcceptEncoding> {
        self.negotiate(header, priorities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let negotiator = EncodingNegotiator::new();
        let encodings = negotiator.parse_header("gzip, deflate;q=0.8");

        assert_eq!(encodings.len(), 2);
        assert_eq!(encodings[0].get_type(), "gzip");
        assert_eq!(encodings[0].quality(), 1.0);
        assert_eq!(encodings[1].get_type(), "deflate");
        assert_eq!(encodings[1].quality(), 0.8);
    }

    #[test]
    fn test_negotiate_exact_match() {
        let negotiator = EncodingNegotiator::new();
        let result = negotiator.negotiate("gzip, deflate", &["gzip"]);

        assert!(result.is_some());
        let encoding = result.unwrap();
        assert_eq!(encoding.get_type(), "gzip");
    }

    #[test]
    fn test_negotiate_wildcard() {
        let negotiator = EncodingNegotiator::new();
        let result = negotiator.negotiate("*", &["gzip"]);

        assert!(result.is_some());
    }

    #[test]
    fn test_negotiate_case_insensitive() {
        let negotiator = EncodingNegotiator::new();
        let result = negotiator.negotiate("GZIP", &["gzip"]);

        assert!(result.is_some());
    }

    #[test]
    fn test_negotiate_no_match() {
        let negotiator = EncodingNegotiator::new();
        let result = negotiator.negotiate("gzip", &["deflate"]);

        assert!(result.is_none());
    }
}
