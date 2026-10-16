use regex::Regex;

/// WebExtensions URL match pattern matcher
/// Implements Chrome / Firefox match pattern spec:
/// <scheme>://<host>/<path> or <all_urls>
pub struct PatternMatcher {
    regex: Regex,
}

impl PatternMatcher {
    pub fn new(pattern: &str) -> Option<Self> {
        if pattern == "<all_urls>" {
            let regex = Regex::new(r#"^https?://.*$"#).ok()?;
            return Some(Self { regex });
        }

        let regex_str = match_pattern_to_regex(pattern)?;
        let regex = Regex::new(&regex_str).ok()?;
        Some(Self { regex })
    }

    pub fn matches(&self, url: &str) -> bool {
        self.regex.is_match(url)
    }
}

/// Converts a Chrome/Firefox match pattern into a regular expression
fn match_pattern_to_regex(pattern: &str) -> Option<String> {
    // Expected format: scheme://host/path
    let parts: Vec<&str> = pattern.splitn(2, "://").collect();
    if parts.len() != 2 {
        return None;
    }

    let scheme = parts[0];
    let rest = parts[1];

    let host_and_path: Vec<&str> = rest.splitn(2, '/').collect();
    let host = host_and_path[0];
    let path = if host_and_path.len() > 1 {
        host_and_path[1]
    } else {
        ""
    };

    // Scheme conversion
    let scheme_regex = match scheme {
        "*" => "(http|https)".to_string(),
        "http" => "http".to_string(),
        "https" => "https".to_string(),
        "file" => "file".to_string(),
        _ => regex::escape(scheme),
    };

    // Host conversion
    let host_regex = if host == "*" {
        r#"[^/]+"#.to_string()
    } else if host.starts_with("*.") {
        format!(r#"([^/]+\.)?{}"#, regex::escape(&host[2..]))
    } else {
        regex::escape(host)
    };

    // Path conversion
    let path_regex = if path == "*" || path.is_empty() {
        r#".*"#.to_string()
    } else {
        regex::escape(path).replace(r#"\*"#, ".*")
    };

    Some(format!(r#"^{}://{}/?{}$"#, scheme_regex, host_regex, path_regex))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_urls() {
        let m = PatternMatcher::new("<all_urls>").unwrap();
        assert!(m.matches("https://google.com/search"));
        assert!(m.matches("http://example.com/"));
    }

    #[test]
    fn test_wildcard_host() {
        let m = PatternMatcher::new("*://*.google.com/*").unwrap();
        assert!(m.matches("https://www.google.com/search?q=rust"));
        assert!(m.matches("http://google.com/"));
        assert!(!m.matches("https://yahoo.com/"));
    }
}
