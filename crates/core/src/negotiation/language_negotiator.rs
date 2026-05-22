use super::{AcceptHeader, AcceptLanguage};

#[derive(Debug, Default)]
pub struct LanguageNegotiator;

impl LanguageNegotiator {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_header(&self, header: &str) -> Vec<AcceptLanguage> {
        let mut languages: Vec<AcceptLanguage> = header
            .split(',')
            .map(|s| AcceptLanguage::new(s.trim()))
            .collect();

        languages.sort_by(|a, b| b.quality().partial_cmp(&a.quality()).unwrap());

        languages
    }

    pub fn negotiate(&self, header: &str, priorities: &[&str]) -> Option<AcceptLanguage> {
        let header_languages = self.parse_header(header);
        let priority_languages: Vec<AcceptLanguage> =
            priorities.iter().map(|p| AcceptLanguage::new(p)).collect();

        self.find_best_match(&header_languages, &priority_languages)
    }

    fn find_best_match(
        &self,
        languages: &[AcceptLanguage],
        priorities: &[AcceptLanguage],
    ) -> Option<AcceptLanguage> {
        for lang in languages {
            if lang.quality() == 0.0 {
                continue;
            }

            for priority in priorities {
                if self.matches(lang, priority) {
                    let combined_quality = lang.quality() * priority.quality();
                    let mut matched = priority.clone();
                    matched.quality = combined_quality;
                    return Some(matched);
                }
            }
        }

        None
    }

    fn matches(&self, lang: &AcceptLanguage, priority: &AcceptLanguage) -> bool {
        if lang.is_wildcard() {
            return true;
        }

        if !lang.primary().eq_ignore_ascii_case(priority.primary()) {
            return false;
        }

        match (lang.sub(), priority.sub()) {
            (Some(lang_sub), Some(priority_sub)) => lang_sub.eq_ignore_ascii_case(priority_sub),
            (None, _) => true,
            (Some(_), None) => false,
        }
    }

    pub fn get_best(&self, header: &str, priorities: &[&str]) -> Option<AcceptLanguage> {
        self.negotiate(header, priorities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let negotiator = LanguageNegotiator::new();
        let languages = negotiator.parse_header("en-US, en;q=0.9, fr;q=0.8");

        assert_eq!(languages.len(), 3);
        assert_eq!(languages[0].get_type(), "en-US");
        assert_eq!(languages[0].quality(), 1.0);
        assert_eq!(languages[1].get_type(), "en");
        assert_eq!(languages[1].quality(), 0.9);
    }

    #[test]
    fn test_negotiate_exact_match() {
        let negotiator = LanguageNegotiator::new();
        let result = negotiator.negotiate("en-US, fr", &["en-US"]);

        assert!(result.is_some());
        let lang = result.unwrap();
        assert_eq!(lang.get_type(), "en-US");
    }

    #[test]
    fn test_negotiate_primary_match() {
        let negotiator = LanguageNegotiator::new();
        let result = negotiator.negotiate("en", &["en-US", "en-GB"]);

        assert!(result.is_some());
        let lang = result.unwrap();
        assert_eq!(lang.primary(), "en");
    }

    #[test]
    fn test_negotiate_wildcard() {
        let negotiator = LanguageNegotiator::new();
        let result = negotiator.negotiate("*", &["en"]);

        assert!(result.is_some());
    }

    #[test]
    fn test_negotiate_no_match() {
        let negotiator = LanguageNegotiator::new();
        let result = negotiator.negotiate("en", &["fr"]);

        assert!(result.is_none());
    }
}
