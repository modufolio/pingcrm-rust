use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Test,
    Production,
}

impl Environment {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Some(Self::Development),
            "test" | "testing" => Some(Self::Test),
            "prod" | "production" => Some(Self::Production),
            _ => None,
        }
    }

    pub fn from_env() -> Self {
        std::env::var("APP_ENV")
            .ok()
            .and_then(|s| Self::from_str(&s))
            .unwrap_or(Self::Production)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Development => "dev",
            Self::Test => "test",
            Self::Production => "prod",
        }
    }

    pub fn is_dev(&self) -> bool {
        matches!(self, Self::Development)
    }

    pub fn is_test(&self) -> bool {
        matches!(self, Self::Test)
    }

    pub fn is_prod(&self) -> bool {
        matches!(self, Self::Production)
    }

    pub fn set_current(&self) {
        std::env::set_var("APP_ENV", self.as_str());
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::Production
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(Environment::from_str("dev"), Some(Environment::Development));
        assert_eq!(Environment::from_str("test"), Some(Environment::Test));
        assert_eq!(Environment::from_str("prod"), Some(Environment::Production));
        assert_eq!(Environment::from_str("invalid"), None);
    }

    #[test]
    fn test_as_str() {
        assert_eq!(Environment::Development.as_str(), "dev");
        assert_eq!(Environment::Test.as_str(), "test");
        assert_eq!(Environment::Production.as_str(), "prod");
    }
}
