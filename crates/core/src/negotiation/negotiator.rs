use super::{Accept, AcceptHeader};

#[derive(Debug, Default)]
pub struct Negotiator;

impl Negotiator {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_header(&self, header: &str) -> Vec<Accept> {
        let mut accepts: Vec<Accept> = header.split(',').map(|s| Accept::new(s.trim())).collect();

        accepts.sort_by(|a, b| {
            let quality_cmp = b.quality().partial_cmp(&a.quality()).unwrap();
            if quality_cmp != std::cmp::Ordering::Equal {
                return quality_cmp;
            }

            b.match_score().cmp(&a.match_score())
        });

        accepts
    }

    pub fn negotiate(&self, header: &str, priorities: &[&str]) -> Option<Accept> {
        let header_accepts = self.parse_header(header);
        let priority_accepts: Vec<Accept> = priorities.iter().map(|p| Accept::new(p)).collect();

        self.find_best_match(&header_accepts, &priority_accepts)
    }

    fn find_best_match(&self, accepts: &[Accept], priorities: &[Accept]) -> Option<Accept> {
        for accept in accepts {
            if accept.quality() == 0.0 {
                continue;
            }

            for priority in priorities {
                if self.matches(accept, priority) {
                    let combined_quality = accept.quality() * priority.quality();
                    let mut matched = priority.clone();
                    matched.quality = combined_quality;
                    return Some(matched);
                }
            }
        }

        None
    }

    fn matches(&self, accept: &Accept, priority: &Accept) -> bool {
        if accept.is_wildcard() {
            return true;
        }

        if accept.is_wildcard_subtype() {
            return accept
                .main_type()
                .eq_ignore_ascii_case(priority.main_type());
        }

        accept.get_type().eq_ignore_ascii_case(priority.get_type())
    }

    pub fn get_best(&self, header: &str, priorities: &[&str]) -> Option<Accept> {
        self.negotiate(header, priorities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let negotiator = Negotiator::new();
        let accepts = negotiator.parse_header("text/html, application/json;q=0.8, */*;q=0.1");

        assert_eq!(accepts.len(), 3);
        assert_eq!(accepts[0].get_type(), "text/html");
        assert_eq!(accepts[0].quality(), 1.0);
        assert_eq!(accepts[1].get_type(), "application/json");
        assert_eq!(accepts[1].quality(), 0.8);
    }

    #[test]
    fn test_negotiate_exact_match() {
        let negotiator = Negotiator::new();
        let result = negotiator.negotiate("text/html, application/json", &["application/json"]);

        assert!(result.is_some());
        let accept = result.unwrap();
        assert_eq!(accept.get_type(), "application/json");
    }

    #[test]
    fn test_negotiate_with_quality() {
        let negotiator = Negotiator::new();
        let result = negotiator.negotiate(
            "text/html;q=0.9, application/json;q=0.8",
            &["application/json", "text/html"],
        );

        assert!(result.is_some());
        let accept = result.unwrap();
        assert_eq!(accept.get_type(), "text/html");
    }

    #[test]
    fn test_negotiate_wildcard() {
        let negotiator = Negotiator::new();
        let result = negotiator.negotiate("*/*", &["application/json"]);

        assert!(result.is_some());
    }

    #[test]
    fn test_negotiate_wildcard_subtype() {
        let negotiator = Negotiator::new();
        let result = negotiator.negotiate("text/*", &["text/html", "application/json"]);

        assert!(result.is_some());
        let accept = result.unwrap();
        assert_eq!(accept.get_type(), "text/html");
    }

    #[test]
    fn test_negotiate_no_match() {
        let negotiator = Negotiator::new();
        let result = negotiator.negotiate("text/html", &["application/json"]);

        assert!(result.is_none());
    }

    #[test]
    fn test_negotiate_quality_multiplication() {
        let negotiator = Negotiator::new();
        let result = negotiator.negotiate("application/json;q=0.8", &["application/json;q=0.5"]);

        assert!(result.is_some());
        let accept = result.unwrap();
        assert_eq!(accept.quality(), 0.4);
    }
}
