//! Article extraction library with support for site-specific XPath rules and generic content extraction.
//!
//! This crate provides functionality to extract clean article content from HTML pages using:
//! - Site-specific XPath rules (ftr-site-config format)
//! - Generic content extraction (Mozilla Readability algorithm)
//! - Automatic markdown conversion
//!
//! # Example
//!
//! ```no_run
//! use malfestio_readability::Readability;
//! use std::path::PathBuf;
//!
//! let html = r#"<html><head><title>Article</title></head><body>...</body></html>"#;
//! let readability = Readability::new(html.to_string(), Some("https://example.com/article"))
//!     .with_rules_dir(PathBuf::from("rules"));
//!
//! let article = readability.parse().unwrap();
//! println!("Title: {}", article.title);
//! println!("Markdown: {}", article.markdown);
//! ```

pub mod cleaner;
pub mod config;
pub mod converter;
pub mod error;
pub mod extractor;

use std::path::PathBuf;

pub use error::{Error, Result};

/// Extracted article content
#[derive(Debug, Clone)]
pub struct Article {
    /// Article title
    pub title: String,
    /// Clean HTML content
    pub content: String,
    /// Markdown formatted content
    pub markdown: String,
    /// Article author (if found)
    pub author: Option<String>,
    /// Publication date (if found)
    pub published_date: Option<String>,
    /// Excerpt (first ~200 chars of content)
    pub excerpt: Option<String>,
}

/// Main entry point for article extraction
pub struct Readability {
    html: String,
    url: Option<String>,
    rules_dir: Option<PathBuf>,
}

impl Readability {
    /// Create a new Readability instance
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML content to extract from
    /// * `url` - Optional URL of the article (used for rule matching)
    pub fn new(html: String, url: Option<&str>) -> Self {
        Self { html, url: url.map(String::from), rules_dir: None }
    }

    /// Set the directory containing extraction rules
    ///
    /// Rules files should be named `domain.com.txt` or `.domain.com.txt` for subdomain matching.
    pub fn with_rules_dir(mut self, path: PathBuf) -> Self {
        self.rules_dir = Some(path);
        self
    }

    /// Extract article content from HTML
    ///
    /// ## Extraction Flow:
    /// 1. If URL provided: Try to load site-specific XPath rules from embedded rules
    /// 2. If rules found: Attempt XPath-based extraction
    /// 3. If no rules OR XPath extraction fails: Fall back to generic heuristic extraction
    /// 4. Convert extracted HTML to markdown
    /// 5. Generate excerpt from markdown
    /// 6. Return complete Article struct
    ///
    /// ## Implementation Gaps:
    /// - XPath extraction doesn't handle complex expressions with `contains()`, `normalize-space()`, etc.
    ///   These will fall back to generic extraction
    /// - No content cleaning between XPath/generic extraction and markdown conversion
    ///   (scripts, styles, etc. may be present in extracted HTML)
    /// - Generic extraction may include non-content elements (nav, footer, etc.)
    ///
    /// ## Design Decision:
    /// We prefer to return *something* (via generic extraction) rather than fail completely.
    /// This maximizes success rate at the cost of potentially lower quality extraction.
    ///
    /// TODO: Add HTML cleaning step before markdown conversion
    /// TODO: Implement XPath strip directives to remove unwanted elements
    /// TODO: Add content validation (minimum length, etc.)
    pub fn parse(&self) -> Result<Article> {
        use config::ConfigLoader;
        use converter::to_markdown;
        use extractor::XPathExtractor;

        let (title, content, author, date) = if let Some(ref url) = self.url {
            let loader = ConfigLoader::new();

            if let Some(config) = loader.load_for_url(url)? {
                let xpath_extractor = XPathExtractor::new(self.html.clone());
                let xpath_result = xpath_extractor.extract(&config)?;

                if let (Some(title), Some(body)) = (xpath_result.title, xpath_result.body_html) {
                    (title, body, xpath_result.author, xpath_result.date)
                } else {
                    self.extract_with_generic()?
                }
            } else {
                self.extract_with_generic()?
            }
        } else {
            self.extract_with_generic()?
        };

        let markdown = to_markdown(&content);
        let excerpt = Some(converter::html2md::generate_excerpt(&markdown, 200));
        Ok(Article { title, content, markdown, author, published_date: date, excerpt })
    }

    /// Extract using generic heuristic-based algorithm
    fn extract_with_generic(&self) -> Result<(String, String, Option<String>, Option<String>)> {
        let generic_extractor = extractor::GenericExtractor::new(self.html.clone());
        let result = generic_extractor.extract()?;
        Ok((result.title, result.body_html, result.author, result.date))
    }
}
