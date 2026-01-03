//! Parser for ftr-site-config format extraction rules

use crate::error::{Error, Result};

/// Site-specific extraction configuration
#[derive(Debug, Clone, Default)]
pub struct SiteConfig {
    /// XPath expressions for title extraction (evaluated in order)
    pub title: Vec<String>,
    /// XPath expressions for body extraction
    pub body: Vec<String>,
    /// XPath expressions for author extraction
    pub author: Vec<String>,
    /// XPath expressions for date extraction
    pub date: Vec<String>,
    /// XPath expressions for elements to strip
    pub strip: Vec<String>,
    /// Substrings to match in @id or @class for stripping
    pub strip_id_or_class: Vec<String>,
    /// Whether to prune non-content elements (default: true)
    pub prune: bool,
    /// Whether to run HTML Tidy preprocessor (default: true)
    pub tidy: bool,
    /// Whether to fall back to generic extraction on failure (default: true)
    pub autodetect_on_failure: bool,
    /// Test URLs for validation
    pub test_urls: Vec<String>,
}

/// Parse a site configuration file in ftr-site-config format
///
/// Format:
/// ```text
/// # Comments start with hash
/// directive: value
/// directive: another value
///
/// # Boolean directives
/// prune: yes
/// tidy: no
/// ```
pub fn parse_config(content: &str) -> Result<SiteConfig> {
    let mut config = SiteConfig { prune: true, tidy: true, autodetect_on_failure: true, ..Default::default() };

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((directive, value)) = line.split_once(':') {
            let directive = directive.trim();
            let value = value.trim();

            match directive {
                "title" => config.title.push(value.to_string()),
                "body" => config.body.push(value.to_string()),
                "author" => config.author.push(value.to_string()),
                "date" => config.date.push(value.to_string()),
                "strip" => config.strip.push(value.to_string()),
                "strip_id_or_class" => config.strip_id_or_class.push(value.to_string()),
                "test_url" => config.test_urls.push(value.to_string()),
                "prune" => config.prune = parse_bool(value)?,
                "tidy" => config.tidy = parse_bool(value)?,
                "autodetect_on_failure" => config.autodetect_on_failure = parse_bool(value)?,
                // TODO: Implement other directives (like http_header)
                _ => {}
            }
        }
    }

    Ok(config)
}

/// Parse a boolean value (yes/no, true/false, 1/0)
fn parse_bool(value: &str) -> Result<bool> {
    match value.to_lowercase().as_str() {
        "yes" | "true" | "1" => Ok(true),
        "no" | "false" | "0" => Ok(false),
        _ => Err(Error::ConfigError(format!("Invalid boolean value: {}", value))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_config() {
        let config = parse_config("").unwrap();
        assert!(config.title.is_empty());
        assert!(config.body.is_empty());
    }

    #[test]
    fn test_parse_arxiv_config() {
        let content = r#"
title: //h1[contains(concat(' ',normalize-space(@class),' '),' title ')]
body: //blockquote[contains(concat(' ',normalize-space(@class),' '),' abstract ')]
date: //meta[@name='citation_date']/@content
author: //meta[@name='citation_author']/@content
test_url: https://arxiv.org/abs/2009.03017
test_url: https://arxiv.org/abs/2012.03780
        "#;

        let config = parse_config(content).unwrap();
        assert_eq!(config.title.len(), 1);
        assert_eq!(config.body.len(), 1);
        assert_eq!(config.author.len(), 1);
        assert_eq!(config.date.len(), 1);
        assert_eq!(config.test_urls.len(), 2);
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"
# This is a comment
title: //h1
# Another comment
body: //article
        "#;

        let config = parse_config(content).unwrap();
        assert_eq!(config.title.len(), 1);
        assert_eq!(config.body.len(), 1);
    }

    #[test]
    fn test_parse_boolean_directives() {
        let content = r#"
prune: no
tidy: yes
autodetect_on_failure: no
        "#;

        let config = parse_config(content).unwrap();
        assert!(!config.prune);
        assert!(config.tidy);
        assert!(!config.autodetect_on_failure);
    }

    #[test]
    fn test_parse_strip_directives() {
        let content = r#"
strip: //div[@class='sidebar']
strip: //div[@id='footer']
strip_id_or_class: advertisement
strip_id_or_class: nav
        "#;

        let config = parse_config(content).unwrap();
        assert_eq!(config.strip.len(), 2);
        assert_eq!(config.strip_id_or_class.len(), 2);
    }
    #[test]
    fn test_parse_invalid_boolean() {
        let content = "prune: perhaps";
        let result = parse_config(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ConfigError(msg) => assert_eq!(msg, "Invalid boolean value: perhaps"),
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_parse_malformed_lines() {
        let content = r#"
title: //h1
malformed line here
another: valid
        "#;
        let config = parse_config(content).unwrap();
        assert_eq!(config.title.len(), 1);
    }
}
