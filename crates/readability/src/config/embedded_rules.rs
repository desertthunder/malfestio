//! Embedded site-specific extraction rules
//!
//! Rules are compiled into the binary at build time for fast access without filesystem dependencies.

use std::collections::HashMap;

/// Embedded rule files indexed by domain
///
/// Supported domains:
/// - arxiv.org
/// - .wikipedia.org (subdomain wildcard)
pub fn get_embedded_rules() -> HashMap<&'static str, &'static str> {
    let mut rules = HashMap::new();
    rules.insert("arxiv.org", include_str!("../../rules/arxiv.org.txt"));
    rules.insert(".wikipedia.org", include_str!("../../rules/.wikipedia.org.txt"));
    rules
}

/// Get embedded rule content for a domain
pub fn get_rule_for_domain(domain: &str) -> Option<&'static str> {
    let rules = get_embedded_rules();

    if let Some(rule) = rules.get(domain) {
        return Some(rule);
    }

    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() > 2 {
        let parent_domain = parts[1..].join(".");
        let wildcard_key = format!(".{}", parent_domain);
        if let Some(rule) = rules.get(wildcard_key.as_str()) {
            return Some(rule);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_rules_loaded() {
        let rules = get_embedded_rules();
        assert!(rules.contains_key("arxiv.org"));
        assert!(rules.contains_key(".wikipedia.org"));
    }

    #[test]
    fn test_get_arxiv_rule() {
        let rule = get_rule_for_domain("arxiv.org");
        assert!(rule.is_some());
        assert!(rule.unwrap().contains("title:"));
    }

    #[test]
    fn test_get_wikipedia_rule_subdomain() {
        let rule = get_rule_for_domain("en.wikipedia.org");
        assert!(rule.is_some());
        assert!(rule.unwrap().contains("firstHeading"));
    }

    #[test]
    fn test_unknown_domain() {
        let rule = get_rule_for_domain("unknown.com");
        assert!(rule.is_none());
    }
}
