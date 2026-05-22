use crate::security::firewall::matches_pattern;
use crate::security::user::UserRole;

#[derive(Debug, Clone)]
pub struct AccessControlRule {
    pub path: String,

    pub roles: Vec<UserRole>,
}

impl AccessControlRule {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            roles: Vec::new(),
        }
    }

    pub fn with_roles(mut self, roles: Vec<UserRole>) -> Self {
        self.roles = roles;
        self
    }

    pub fn matches(&self, path: &str) -> bool {
        matches_pattern(&self.path, path)
    }

    pub fn check_roles(&self, user_roles: &[UserRole]) -> bool {
        if self.roles.is_empty() {
            return true;
        }

        for required_role in &self.roles {
            for user_role in user_roles {
                if user_role.has_permission(required_role) {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone, Default)]
pub struct AccessControlConfig {
    pub rules: Vec<AccessControlRule>,
}

impl AccessControlConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(mut self, rule: AccessControlRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn find_rule(&self, path: &str) -> Option<&AccessControlRule> {
        self.rules.iter().find(|rule| rule.matches(path))
    }

    pub fn default_rules() -> Self {
        Self::new()
            .add_rule(AccessControlRule::new("/assets"))
            .add_rule(AccessControlRule::new("/api/login"))
            .add_rule(AccessControlRule::new("/api/register"))
            .add_rule(AccessControlRule::new("/admin").with_roles(vec![UserRole::Admin]))
            .add_rule(AccessControlRule::new("/api").with_roles(vec![UserRole::User]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_path_matching() {
        let rule = AccessControlRule::new("/admin");

        assert!(rule.matches("/admin"));
        assert!(rule.matches("/admin/users"));
        assert!(!rule.matches("/api"));
    }

    #[test]
    fn test_rule_segment_matching() {
        let rule = AccessControlRule::new("admin:0");

        assert!(rule.matches("/admin/users"));
        assert!(!rule.matches("/api/users"));
    }

    #[test]
    fn test_role_checking_no_roles_required() {
        let rule = AccessControlRule::new("/public");
        let user_roles = vec![UserRole::Guest];

        assert!(rule.check_roles(&user_roles));
    }

    #[test]
    fn test_role_checking_exact_match() {
        let rule = AccessControlRule::new("/admin").with_roles(vec![UserRole::Admin]);

        let admin_roles = vec![UserRole::Admin];
        assert!(rule.check_roles(&admin_roles));

        let user_roles = vec![UserRole::User];
        assert!(!rule.check_roles(&user_roles));
    }

    #[test]
    fn test_role_checking_hierarchy() {
        let rule = AccessControlRule::new("/admin").with_roles(vec![UserRole::Admin]);

        let super_admin_roles = vec![UserRole::SuperAdmin];
        assert!(rule.check_roles(&super_admin_roles));

        let admin_roles = vec![UserRole::Admin];
        assert!(rule.check_roles(&admin_roles));

        let user_roles = vec![UserRole::User];
        assert!(!rule.check_roles(&user_roles));
    }

    #[test]
    fn test_role_checking_multiple_roles() {
        let rule = AccessControlRule::new("/api").with_roles(vec![UserRole::User, UserRole::Admin]);

        let user_roles = vec![UserRole::User];
        assert!(rule.check_roles(&user_roles));

        let admin_roles = vec![UserRole::Admin];
        assert!(rule.check_roles(&admin_roles));

        let guest_roles = vec![UserRole::Guest];
        assert!(!rule.check_roles(&guest_roles));
    }

    #[test]
    fn test_config_find_rule() {
        let config = AccessControlConfig::new()
            .add_rule(AccessControlRule::new("/admin").with_roles(vec![UserRole::Admin]))
            .add_rule(AccessControlRule::new("/api").with_roles(vec![UserRole::User]));

        let admin_rule = config.find_rule("/admin/users");
        assert!(admin_rule.is_some());
        assert_eq!(admin_rule.unwrap().path, "/admin");

        let api_rule = config.find_rule("/api/contacts");
        assert!(api_rule.is_some());
        assert_eq!(api_rule.unwrap().path, "/api");

        let no_rule = config.find_rule("/public");
        assert!(no_rule.is_none());
    }

    #[test]
    fn test_config_first_match_wins() {
        let config = AccessControlConfig::new()
            .add_rule(AccessControlRule::new("/api/public").with_roles(vec![]))
            .add_rule(AccessControlRule::new("/api").with_roles(vec![UserRole::User]));

        let rule = config.find_rule("/api/public/data");
        assert!(rule.is_some());
        assert!(rule.unwrap().roles.is_empty());
    }

    #[test]
    fn test_default_rules() {
        let config = AccessControlConfig::default_rules();

        let admin_rule = config.find_rule("/admin/dashboard");
        assert!(admin_rule.is_some());
        assert!(admin_rule.unwrap().roles.contains(&UserRole::Admin));

        let api_rule = config.find_rule("/api/users");
        assert!(api_rule.is_some());
        assert!(api_rule.unwrap().roles.contains(&UserRole::User));
    }
}
