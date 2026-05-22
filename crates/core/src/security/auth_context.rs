#[derive(Debug, Clone)]
pub struct AuthContext {
    pub name: String,

    pub stateless: bool,
}

impl AuthContext {
    pub fn new(name: impl Into<String>, stateless: bool) -> Self {
        Self {
            name: name.into(),
            stateless,
        }
    }

    pub fn web(name: impl Into<String>) -> Self {
        Self::new(name, false)
    }

    pub fn api(name: impl Into<String>) -> Self {
        Self::new(name, true)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_stateless(&self) -> bool {
        self.stateless
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::web("main")
    }
}
