pub fn matches_pattern(pattern: &str, path: &str) -> bool {
    if pattern.contains(':') {
        match_segment_pattern(pattern, path)
    } else {
        matches_starts_with(pattern, path)
    }
}

fn match_segment_pattern(pattern: &str, path: &str) -> bool {
    let parts: Vec<&str> = pattern.splitn(2, ':').collect();
    if parts.len() != 2 {
        return false;
    }

    let value = parts[0];
    let position = match parts[1].parse::<usize>() {
        Ok(pos) => pos,
        Err(_) => return false,
    };

    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    segments.get(position).map_or(false, |&seg| seg == value)
}

fn matches_starts_with(pattern: &str, path: &str) -> bool {
    path.starts_with(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_matching() {
        assert!(matches_pattern("/api", "/api"));

        assert!(matches_pattern("/api", "/api/users"));
        assert!(matches_pattern("/api", "/api/users/123"));

        assert!(!matches_pattern("/api", "/app"));
        assert!(!matches_pattern("/api", "/application"));
        assert!(!matches_pattern("/admin", "/api"));
    }

    #[test]
    fn test_segment_matching() {
        assert!(matches_pattern("api:0", "/api/users"));
        assert!(matches_pattern("api:0", "/api"));

        assert!(matches_pattern("users:1", "/api/users"));
        assert!(matches_pattern("users:1", "/api/users/123"));

        assert!(matches_pattern("123:2", "/api/users/123"));

        assert!(!matches_pattern("admin:0", "/api/users"));
        assert!(!matches_pattern("posts:1", "/api/users"));
        assert!(!matches_pattern("api:1", "/api/users"));
    }

    #[test]
    fn test_segment_out_of_bounds() {
        assert!(!matches_pattern("foo:10", "/api/users"));
        assert!(!matches_pattern("bar:5", "/api"));
    }

    #[test]
    fn test_invalid_patterns() {
        assert!(!matches_pattern("api:", "/api/users"));

        assert!(!matches_pattern("api:abc", "/api/users"));
    }

    #[test]
    fn test_empty_path() {
        assert!(!matches_pattern("/api", ""));
        assert!(!matches_pattern("api:0", ""));
    }

    #[test]
    fn test_root_path() {
        assert!(matches_pattern("/", "/"));
        assert!(matches_pattern("/", "/api"));
        assert!(matches_pattern("/", "/anything"));
    }
}
