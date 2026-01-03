//! Load site-specific configuration files based on URL

use crate::config::embedded_rules;
use crate::config::parser::{SiteConfig, parse_config};
use crate::error::Result;
use std::path::{Path, PathBuf};
use url::Url;

/// Loads site-specific configuration files
///
/// First checks embedded rules, then falls back to external rules_dir if provided.
#[derive(Default)]
pub struct ConfigLoader {
    rules_dir: Option<PathBuf>,
}

impl ConfigLoader {
    /// Create a new config loader with embedded rules only
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a config loader with an external rules directory
    ///
    /// External rules take precedence over embedded rules.
    pub fn with_rules_dir(rules_dir: PathBuf) -> Self {
        Self { rules_dir: Some(rules_dir) }
    }

    /// Load configuration for a given URL
    ///
    /// Priority:
    /// 1. External rules (if rules_dir provided)
    /// 2. Embedded rules
    /// 3. None (if no match found)
    pub fn load_for_url(&self, url: &str) -> Result<Option<SiteConfig>> {
        let Some(domain) = Self::extract_domain(url) else {
            return Ok(None);
        };

        if let Some(ref rules_dir) = self.rules_dir
            && let Some(config) = self.try_load_from_dir(rules_dir, &domain)?
        {
            return Ok(Some(config));
        }

        if let Some(rule_content) = embedded_rules::get_rule_for_domain(&domain) {
            return Ok(Some(parse_config(rule_content)?));
        }

        Ok(None)
    }

    /// Try to load config from external directory
    fn try_load_from_dir(&self, rules_dir: &Path, domain: &str) -> Result<Option<SiteConfig>> {
        let exact_path = rules_dir.join(format!("{}.txt", domain));
        if exact_path.exists() {
            let content = std::fs::read_to_string(&exact_path)?;
            return Ok(Some(parse_config(&content)?));
        }

        let wildcard_path = rules_dir.join(format!(".{}.txt", domain));
        if wildcard_path.exists() {
            let content = std::fs::read_to_string(&wildcard_path)?;
            return Ok(Some(parse_config(&content)?));
        }

        if let Some(parent_domain) = Self::extract_parent_domain(domain) {
            let parent_wildcard = rules_dir.join(format!(".{}.txt", parent_domain));
            if parent_wildcard.exists() {
                let content = std::fs::read_to_string(&parent_wildcard)?;
                return Ok(Some(parse_config(&content)?));
            }
        }

        Ok(None)
    }

    /// Extract domain from URL
    fn extract_domain(url: &str) -> Option<String> {
        Url::parse(url).ok().and_then(|u| u.host_str().map(String::from))
    }

    /// Extract parent domain (e.g., "en.wikipedia.org" -> "wikipedia.org")
    fn extract_parent_domain(domain: &str) -> Option<String> {
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() > 2 { Some(parts[1..].join(".")) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            ConfigLoader::extract_domain("https://arxiv.org/abs/123"),
            Some("arxiv.org".to_string())
        );
        assert_eq!(
            ConfigLoader::extract_domain("https://en.wikipedia.org/wiki/Article"),
            Some("en.wikipedia.org".to_string())
        );
        assert_eq!(ConfigLoader::extract_domain("invalid"), None);
    }

    #[test]
    fn test_load_embedded_arxiv() {
        let loader = ConfigLoader::new();
        let config = loader
            .load_for_url("https://arxiv.org/abs/2009.03017")
            .unwrap()
            .expect("Should find embedded arxiv config");

        assert_eq!(config.title.len(), 1);
        assert_eq!(config.body.len(), 1);
    }

    #[test]
    fn test_load_embedded_wikipedia() {
        let loader = ConfigLoader::new();
        let config = loader
            .load_for_url("https://en.wikipedia.org/wiki/Article")
            .unwrap()
            .expect("Should find embedded wikipedia config");

        assert_eq!(config.title.len(), 1);
        assert_eq!(config.body.len(), 1);
        assert!(!config.prune);
    }
}
