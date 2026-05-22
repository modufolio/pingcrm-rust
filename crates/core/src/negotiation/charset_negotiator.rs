use super::{AcceptCharset, AcceptHeader};

#[derive(Debug, Default)]
pub struct CharsetNegotiator;

impl CharsetNegotiator {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_header(&self, header: &str) -> Vec<AcceptCharset> {
        let mut charsets: Vec<AcceptCharset> = header
            .split(',')
            .map(|s| AcceptCharset::new(s.trim()))
            .collect();

        charsets.sort_by(|a, b| b.quality().partial_cmp(&a.quality()).unwrap());

        charsets
    }

    pub fn negotiate(&self, header: &str, priorities: &[&str]) -> Option<AcceptCharset> {
        let header_charsets = self.parse_header(header);
        let priority_charsets: Vec<AcceptCharset> =
            priorities.iter().map(|p| AcceptCharset::new(p)).collect();

        self.find_best_match(&header_charsets, &priority_charsets)
    }

    fn find_best_match(
        &self,
        charsets: &[AcceptCharset],
        priorities: &[AcceptCharset],
    ) -> Option<AcceptCharset> {
        for charset in charsets {
            if charset.quality() == 0.0 {
                continue;
            }

            for priority in priorities {
                if self.matches(charset, priority) {
                    let combined_quality = charset.quality() * priority.quality();
                    let mut matched = priority.clone();
                    matched.quality = combined_quality;
                    return Some(matched);
                }
            }
        }

        None
    }

    fn matches(&self, charset: &AcceptCharset, priority: &AcceptCharset) -> bool {
        if charset.is_wildcard() {
            return true;
        }

        charset.get_type().eq_ignore_ascii_case(priority.get_type())
    }

    pub fn get_best(&self, header: &str, priorities: &[&str]) -> Option<AcceptCharset> {
        self.negotiate(header, priorities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let negotiator = CharsetNegotiator::new();
        let charsets = negotiator.parse_header("utf-8, iso-8859-1;q=0.8");

        assert_eq!(charsets.len(), 2);
        assert_eq!(charsets[0].get_type(), "utf-8");
        assert_eq!(charsets[0].quality(), 1.0);
        assert_eq!(charsets[1].get_type(), "iso-8859-1");
        assert_eq!(charsets[1].quality(), 0.8);
    }

    #[test]
    fn test_negotiate_exact_match() {
        let negotiator = CharsetNegotiator::new();
        let result = negotiator.negotiate("utf-8, iso-8859-1", &["utf-8"]);

        assert!(result.is_some());
        let charset = result.unwrap();
        assert_eq!(charset.get_type(), "utf-8");
    }

    #[test]
    fn test_negotiate_wildcard() {
        let negotiator = CharsetNegotiator::new();
        let result = negotiator.negotiate("*", &["utf-8"]);

        assert!(result.is_some());
    }

    #[test]
    fn test_negotiate_case_insensitive() {
        let negotiator = CharsetNegotiator::new();
        let result = negotiator.negotiate("UTF-8", &["utf-8"]);

        assert!(result.is_some());
    }

    #[test]
    fn test_negotiate_no_match() {
        let negotiator = CharsetNegotiator::new();
        let result = negotiator.negotiate("utf-8", &["iso-8859-1"]);

        assert!(result.is_none());
    }
}
