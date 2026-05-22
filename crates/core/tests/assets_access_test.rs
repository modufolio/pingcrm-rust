use appkit_core::security::access_control::AccessControlConfig;
use appkit_core::security::firewall::{FirewallConfig, FirewallRule};

#[test]
fn test_firewall_allows_assets() {
    let configs = vec![FirewallConfig::new("/assets", FirewallRule::Public)];

    let path = "/assets/css/app.css";
    let rule = configs
        .iter()
        .find(|config| appkit_core::security::firewall::matches_pattern(&config.pattern, path))
        .map(|config| config.rule.clone());

    assert!(
        rule.is_some(),
        "No firewall rule matched /assets/css/app.css"
    );
    assert_eq!(
        rule.unwrap(),
        FirewallRule::Public,
        "/assets should be public"
    );

    let path = "/assets/js/app.js";
    let rule = configs
        .iter()
        .find(|config| appkit_core::security::firewall::matches_pattern(&config.pattern, path))
        .map(|config| config.rule.clone());

    assert!(rule.is_some(), "No firewall rule matched /assets/js/app.js");
    assert_eq!(
        rule.unwrap(),
        FirewallRule::Public,
        "/assets should be public"
    );
}

#[test]
fn test_access_control_allows_assets() {
    let config = AccessControlConfig::default_rules();

    let css_rule = config.find_rule("/assets/css/app.css");
    assert!(
        css_rule.is_some(),
        "No access control rule matched /assets/css/app.css"
    );
    assert!(
        css_rule.unwrap().roles.is_empty(),
        "/assets should have no role requirements"
    );

    let js_rule = config.find_rule("/assets/js/app.js");
    assert!(
        js_rule.is_some(),
        "No access control rule matched /assets/js/app.js"
    );
    assert!(
        js_rule.unwrap().roles.is_empty(),
        "/assets should have no role requirements"
    );
}

#[test]
fn test_pattern_matching_for_assets() {
    assert!(
        appkit_core::security::firewall::matches_pattern("/assets", "/assets/css/app.css"),
        "/assets pattern should match /assets/css/app.css"
    );
    assert!(
        appkit_core::security::firewall::matches_pattern("/assets", "/assets/js/app.js"),
        "/assets pattern should match /assets/js/app.js"
    );
    assert!(
        appkit_core::security::firewall::matches_pattern("/assets", "/assets"),
        "/assets pattern should match /assets exactly"
    );

    assert!(
        !appkit_core::security::firewall::matches_pattern("/css", "/assets/css/app.css"),
        "/css pattern should NOT match /assets/css/app.css (this was the bug)"
    );
    assert!(
        !appkit_core::security::firewall::matches_pattern("/js", "/assets/js/app.js"),
        "/js pattern should NOT match /assets/js/app.js (this was the bug)"
    );
}
