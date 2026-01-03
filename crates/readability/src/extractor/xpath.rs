//! XPath-based content extraction using site-specific rules
//!
//! This module provides content extraction from HTML documents using XPath-like expressions.
//!
//! ## Strategy
//!
//! Since Rust doesn't have a robust HTML-compatible XPath library, we use a hybrid approach:
//! 1. Convert simple XPath expressions to CSS selectors (scraper handles these well)
//! 2. Handle complex patterns (contains(), normalize-space()) with custom matchers
//! 3. Use regex parsing for XPath syntax to extract selector components
//!
//! ## Supported XPath Patterns
//!
//! - `//tag` - Simple tag selection
//! - `//tag[@id='value']` - ID selection
//! - `//tag[@class='value']` - Exact class match
//! - `//tag[contains(@class, 'value')]` - Class contains match
//! - `//tag[contains(concat(' ',normalize-space(@class),' '),' value ')]` - Normalized class match
//! - `//meta[@name='value']/@content` - Attribute extraction from meta tags

use crate::config::SiteConfig;
use crate::error::{Error, Result};
use regex::Regex;
use scraper::{ElementRef, Html, Selector};

static VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr",
];

/// Extracted content from XPath rules
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub title: Option<String>,
    pub body_html: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
}

/// XPath-based extractor
pub struct XPathExtractor {
    html: String,
}

impl XPathExtractor {
    /// Create a new XPath extractor
    pub fn new(html: String) -> Self {
        Self { html }
    }

    /// Extract content using site-specific rules
    pub fn extract(&self, config: &SiteConfig) -> Result<ExtractedContent> {
        let cleaned_html = self.apply_strip_rules(&self.html, config)?;
        let document = Html::parse_document(&cleaned_html);

        let title = self.extract_field(&document, &config.title, false)?;
        let body_html = self.extract_field(&document, &config.body, true)?;
        let author = self.extract_field(&document, &config.author, false)?;
        let date = self.extract_field(&document, &config.date, false)?;

        Ok(ExtractedContent { title, body_html, author, date })
    }

    /// Apply strip rules to remove unwanted elements
    ///
    /// Processes both `strip` (XPath) and `strip_id_or_class` (substring match) directives.
    fn apply_strip_rules(&self, html: &str, config: &SiteConfig) -> Result<String> {
        let document = Html::parse_document(html);
        let mut elements_to_remove: Vec<String> = Vec::new();

        for substring in &config.strip_id_or_class {
            let substring_lower = substring.to_lowercase();
            for element in document.tree.nodes() {
                if let Some(el) = ElementRef::wrap(element) {
                    let should_remove = el
                        .value()
                        .id()
                        .is_some_and(|id| id.to_lowercase().contains(&substring_lower))
                        || el
                            .value()
                            .classes()
                            .any(|class| class.to_lowercase().contains(&substring_lower));

                    if should_remove {
                        elements_to_remove.push(self.element_signature(&el));
                    }
                }
            }
        }

        for xpath in &config.strip {
            if let Some((css, _)) = self.xpath_to_css_with_attr(xpath)
                && let Ok(selector) = Selector::parse(&css)
            {
                for el in document.select(&selector) {
                    elements_to_remove.push(self.element_signature(&el));
                }
            }
        }

        self.rebuild_html_without_elements(&document, &elements_to_remove)
    }

    /// Generate a signature for an element to identify it during rebuild
    fn element_signature(&self, el: &ElementRef) -> String {
        let tag = el.value().name();
        let id = el.value().id().unwrap_or("");
        let classes: Vec<&str> = el.value().classes().collect();
        format!("{}#{}#{}", tag, id, classes.join(","))
    }

    /// Rebuild HTML without specified elements
    fn rebuild_html_without_elements(&self, document: &Html, to_remove: &[String]) -> Result<String> {
        if to_remove.is_empty() {
            return Ok(self.html.clone());
        }

        let mut result = String::new();
        self.rebuild_node(&document.root_element(), to_remove, &mut result);
        Ok(result)
    }

    /// Recursively rebuild a node and its children, skipping removed elements
    fn rebuild_node(&self, element: &ElementRef, to_remove: &[String], output: &mut String) {
        let sig = self.element_signature(element);
        if to_remove.contains(&sig) {
            return;
        }

        let tag = element.value().name();
        output.push('<');
        output.push_str(tag);

        for (name, value) in element.value().attrs() {
            output.push(' ');
            output.push_str(name);
            output.push_str("=\"");
            output.push_str(&html_escape::encode_double_quoted_attribute(value));
            output.push('"');
        }
        output.push('>');

        for child in element.children() {
            if let Some(el) = ElementRef::wrap(child) {
                self.rebuild_node(&el, to_remove, output);
            } else if let Some(text) = child.value().as_text() {
                output.push_str(&html_escape::encode_text(&text.to_string()));
            }
        }

        if !VOID_ELEMENTS.contains(&tag) {
            output.push_str("</");
            output.push_str(tag);
            output.push('>');
        }
    }

    /// Extract a field using XPath expressions (tries each in order)
    fn extract_field(&self, document: &Html, xpaths: &[String], extract_html: bool) -> Result<Option<String>> {
        for xpath_expr in xpaths {
            if let Some(result) = self.evaluate_xpath(document, xpath_expr, extract_html)? {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    /// Evaluate an XPath expression against the document
    fn evaluate_xpath(&self, document: &Html, xpath: &str, extract_html: bool) -> Result<Option<String>> {
        let (xpath_part, attr_to_extract) = if let Some(pos) = xpath.rfind("/@") {
            (&xpath[..pos], Some(&xpath[pos + 2..]))
        } else {
            (xpath, None)
        };

        let (css, class_filter) = match self.xpath_to_css_with_attr(xpath_part) {
            Some(result) => result,
            None => return Ok(None),
        };

        let selector =
            Selector::parse(&css).map_err(|e| Error::XPathError(format!("Invalid CSS selector '{}': {:?}", css, e)))?;

        for element in document.select(&selector) {
            if let Some(ref filter) = class_filter
                && !self.element_has_class_containing(&element, filter)
            {
                continue;
            }

            if let Some(attr) = attr_to_extract {
                if let Some(value) = element.value().attr(attr) {
                    return Ok(Some(value.to_string()));
                }
                continue;
            }

            let content =
                if extract_html { element.inner_html() } else { element.text().collect::<Vec<_>>().join(" ") };

            let content = content.trim().to_string();
            if !content.is_empty() {
                return Ok(Some(content));
            }
        }

        Ok(None)
    }

    /// Convert XPath to CSS selector with optional class filter
    fn xpath_to_css_with_attr(&self, xpath: &str) -> Option<(String, Option<String>)> {
        let xpath = xpath.trim();

        if xpath.starts_with("//") && !xpath.contains('[') && !xpath.contains('@') {
            let tag = xpath.trim_start_matches("//");
            return Some((tag.to_string(), None));
        }

        if let Some(css) = self.parse_id_selector(xpath) {
            return Some((css, None));
        }

        if let Some((css, class_filter)) = self.parse_contains_class_normalized(xpath) {
            return Some((css, Some(class_filter)));
        }

        if let Some((css, class_filter)) = self.parse_contains_class_simple(xpath) {
            return Some((css, Some(class_filter)));
        }

        if let Some(css) = self.parse_exact_class(xpath) {
            return Some((css, None));
        }

        if let Some(css) = self.parse_exact_class(xpath) {
            return Some((css, None));
        }

        if let Some(css) = self.parse_any_tag_with_id(xpath) {
            return Some((css, None));
        }

        if let Some(css) = self.parse_meta_selector(xpath) {
            return Some((css, None));
        }

        if let Some(css) = self.parse_meta_selector(xpath) {
            return Some((css, None));
        }

        None
    }

    /// Parse //tag[@id='value'] pattern
    fn parse_id_selector(&self, xpath: &str) -> Option<String> {
        let re = Regex::new(r#"//(\w+)\[@id\s*=\s*['"]([^'"]+)['"]\]"#).ok()?;
        let caps = re.captures(xpath)?;
        let tag = caps.get(1)?.as_str();
        let id = caps.get(2)?.as_str();
        Some(format!("{}#{}", tag, id))
    }

    /// Parse //*[@id='value'] pattern
    fn parse_any_tag_with_id(&self, xpath: &str) -> Option<String> {
        let re = Regex::new(r#"//\*\[@id\s*=\s*['"]([^'"]+)['"]\]"#).ok()?;
        let caps = re.captures(xpath)?;
        let id = caps.get(1)?.as_str();
        Some(format!("#{}", id))
    }

    /// Parse //tag[@class='value'] pattern (exact class match)
    fn parse_exact_class(&self, xpath: &str) -> Option<String> {
        if xpath.contains("contains") {
            return None;
        }
        let re = Regex::new(r#"//(\w+)\[@class\s*=\s*['"]([^'"]+)['"]\]"#).ok()?;
        let caps = re.captures(xpath)?;
        let tag = caps.get(1)?.as_str();
        let class = caps.get(2)?.as_str();
        Some(format!("{}[class=\"{}\"]", tag, class))
    }

    /// Parse //tag[contains(@class, 'value')] pattern
    fn parse_contains_class_simple(&self, xpath: &str) -> Option<(String, String)> {
        let re = Regex::new(r#"//(\w+)\[contains\s*\(\s*@class\s*,\s*['"]([^'"]+)['"]\s*\)\]"#).ok()?;
        let caps = re.captures(xpath)?;
        let tag = caps.get(1)?.as_str();
        let class_substr = caps.get(2)?.as_str();
        Some((tag.to_string(), class_substr.to_string()))
    }

    /// Parse //tag[contains(concat(' ',normalize-space(@class),' '),' value ')] pattern
    fn parse_contains_class_normalized(&self, xpath: &str) -> Option<(String, String)> {
        let re = Regex::new(r#"//(\w+)\[contains\s*\(\s*concat\s*\(.+\)\s*,\s*['"]([^'"]+)['"]\s*\)\]"#).ok()?;
        let caps = re.captures(xpath)?;
        let tag = caps.get(1)?.as_str();
        let class_name = caps.get(2)?.as_str().trim();
        Some((tag.to_string(), class_name.to_string()))
    }

    /// Parse //meta[@name='value'] pattern
    fn parse_meta_selector(&self, xpath: &str) -> Option<String> {
        let re = Regex::new(r#"//meta\[@(\w+)\s*=\s*['"]([^'"]+)['"]\]"#).ok()?;
        let caps = re.captures(xpath)?;
        let attr_name = caps.get(1)?.as_str();
        let attr_value = caps.get(2)?.as_str();
        Some(format!("meta[{}=\"{}\"]", attr_name, attr_value))
    }

    /// Check if element has a class containing the given substring
    fn element_has_class_containing(&self, element: &ElementRef, class_filter: &str) -> bool {
        element.value().classes().any(|class| class.contains(class_filter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parser::SiteConfig;

    #[test]
    fn test_xpath_to_css_simple_tag() {
        let extractor = XPathExtractor::new(String::new());
        let (css, filter) = extractor.xpath_to_css_with_attr("//h1").unwrap();
        assert_eq!(css, "h1");
        assert!(filter.is_none());
    }

    #[test]
    fn test_xpath_to_css_id_selector() {
        let extractor = XPathExtractor::new(String::new());
        let (css, filter) = extractor.xpath_to_css_with_attr("//h1[@id='firstHeading']").unwrap();
        assert_eq!(css, "h1#firstHeading");
        assert!(filter.is_none());
    }

    #[test]
    fn test_xpath_to_css_any_tag_with_id() {
        let extractor = XPathExtractor::new(String::new());
        let (css, filter) = extractor.xpath_to_css_with_attr("//*[@id='bodyContent']").unwrap();
        assert_eq!(css, "#bodyContent");
        assert!(filter.is_none());
    }

    #[test]
    fn test_xpath_contains_class_simple() {
        let extractor = XPathExtractor::new(String::new());
        let (css, filter) = extractor
            .xpath_to_css_with_attr("//div[contains(@class, 'content')]")
            .unwrap();
        assert_eq!(css, "div");
        assert_eq!(filter, Some("content".to_string()));
    }

    #[test]
    fn test_xpath_contains_class_normalized() {
        let extractor = XPathExtractor::new(String::new());
        let xpath = "//h1[contains(concat(' ',normalize-space(@class),' '),' title ')]";
        let (css, filter) = extractor.xpath_to_css_with_attr(xpath).unwrap();
        assert_eq!(css, "h1");
        assert_eq!(filter, Some("title".to_string()));
    }

    #[test]
    fn test_extract_meta_attribute() {
        let html = r#"
            <html>
                <head>
                    <meta name="citation_date" content="2020-09-07">
                    <meta name="citation_author" content="John Doe">
                </head>
            </html>
        "#;

        let extractor = XPathExtractor::new(html.to_string());
        let document = Html::parse_document(html);

        let date = extractor
            .evaluate_xpath(&document, "//meta[@name='citation_date']/@content", false)
            .unwrap();
        assert_eq!(date, Some("2020-09-07".to_string()));

        let author = extractor
            .evaluate_xpath(&document, "//meta[@name='citation_author']/@content", false)
            .unwrap();
        assert_eq!(author, Some("John Doe".to_string()));
    }

    #[test]
    fn test_extract_with_contains_class() {
        let html = r#"
            <html>
                <body>
                    <h1 class="page-title title main">Article Title</h1>
                    <div class="article-content">Content here</div>
                </body>
            </html>
        "#;

        let extractor = XPathExtractor::new(html.to_string());
        let document = Html::parse_document(html);

        let title = extractor
            .evaluate_xpath(&document, "//h1[contains(@class, 'title')]", false)
            .unwrap();
        assert_eq!(title, Some("Article Title".to_string()));
    }

    #[test]
    fn test_strip_id_or_class() {
        let html = r#"
            <html>
                <body>
                    <div id="main-content">Main content</div>
                    <div class="sidebar-widget">Sidebar</div>
                    <div class="advertisement-banner">Ad</div>
                </body>
            </html>
        "#;

        let config = SiteConfig {
            strip_id_or_class: vec!["sidebar".to_string(), "advertisement".to_string()],
            ..Default::default()
        };

        let extractor = XPathExtractor::new(html.to_string());
        let cleaned = extractor.apply_strip_rules(html, &config).unwrap();

        assert!(cleaned.contains("Main content"));
        assert!(!cleaned.contains("Sidebar"));
        assert!(!cleaned.contains("Ad"));
    }

    #[test]
    fn test_strip_xpath() {
        let html = r#"
            <html>
                <body>
                    <div id="content">Main content</div>
                    <div id="toc">Table of contents</div>
                    <div id="footer">Footer</div>
                </body>
            </html>
        "#;

        let config = SiteConfig {
            strip: vec!["//*[@id='toc']".to_string(), "//div[@id='footer']".to_string()],
            ..Default::default()
        };

        let extractor = XPathExtractor::new(html.to_string());
        let cleaned = extractor.apply_strip_rules(html, &config).unwrap();

        assert!(cleaned.contains("Main content"));
        assert!(!cleaned.contains("Table of contents"));
        assert!(!cleaned.contains("Footer"));
    }

    #[test]
    fn test_full_extraction() {
        let html = r#"
            <html>
                <head>
                    <meta name="author" content="Test Author">
                    <meta name="date" content="2024-01-15">
                </head>
                <body>
                    <h1 id="title">Test Title</h1>
                    <article class="content">
                        <p>Article content here.</p>
                    </article>
                    <div class="sidebar">Sidebar content</div>
                </body>
            </html>
        "#;

        let config = SiteConfig {
            title: vec!["//h1[@id='title']".to_string()],
            body: vec!["//article".to_string()],
            author: vec!["//meta[@name='author']/@content".to_string()],
            date: vec!["//meta[@name='date']/@content".to_string()],
            strip_id_or_class: vec!["sidebar".to_string()],
            ..Default::default()
        };

        let extractor = XPathExtractor::new(html.to_string());
        let result = extractor.extract(&config).unwrap();

        assert_eq!(result.title, Some("Test Title".to_string()));
        assert!(result.body_html.unwrap().contains("Article content here"));
        assert_eq!(result.author, Some("Test Author".to_string()));
        assert_eq!(result.date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_strip_elements_inside_body() {
        let html = r#"
            <html>
                <body>
                    <div id="bodyContent">
                        <h2>Section Title <span class="mw-editsection">[edit]</span></h2>
                        <p>Main content here.</p>
                        <h2>Another Section <span class="mw-editsection-bracket">[</span></h2>
                    </div>
                </body>
            </html>
        "#;

        let config = SiteConfig {
            body: vec!["//*[@id='bodyContent']".to_string()],
            strip_id_or_class: vec!["editsection".to_string()],
            ..Default::default()
        };

        let extractor = XPathExtractor::new(html.to_string());
        let result = extractor.extract(&config).unwrap();

        let body = result.body_html.expect("Should extract body");
        println!("Extracted body: {}", body);

        assert!(!body.contains("mw-editsection"), "mw-editsection should be stripped");
        assert!(!body.contains("[edit]"), "[edit] text should be stripped");
        assert!(body.contains("Main content here"));
        assert!(body.contains("Section Title"));
    }
    #[test]
    fn test_rebuild_void_elements() {
        let html = r#"
            <html>
                <body>
                    <p>Text <br> with break</p>
                    <img src="test.jpg">
                    <div id="remove">Remove me</div>
                </body>
            </html>
        "#;

        let config = SiteConfig { strip: vec!["//*[@id='remove']".to_string()], ..Default::default() };
        let extractor = XPathExtractor::new(html.to_string());
        let result = extractor.apply_strip_rules(html, &config).unwrap();

        assert!(result.contains("<br>"));
        assert!(!result.contains("</br>"));
        assert!(result.contains("<img src=\"test.jpg\">"));
        assert!(!result.contains("</img>"));
        assert!(!result.contains("Remove me"));
    }

    #[test]
    fn test_unsupported_xpath() {
        let html = "<html></html>";
        let extractor = XPathExtractor::new(html.to_string());
        let document = Html::parse_document(html);

        // TODO: implement complex axis navigation
        let result = extractor.evaluate_xpath(&document, "//div/following-sibling::p", false);
        assert!(matches!(result, Err(Error::XPathError(_))));
    }
}

#[test]
fn test_wikipedia_xpath_patterns() {
    let extractor = XPathExtractor::new(String::new());
    let (css, filter) = extractor.xpath_to_css_with_attr("//h1[@id='firstHeading']").unwrap();
    assert_eq!(css, "h1#firstHeading");
    assert!(filter.is_none());

    let (css, filter) = extractor.xpath_to_css_with_attr("//div[@id = 'bodyContent']").unwrap();
    assert_eq!(css, "div#bodyContent");
    assert!(filter.is_none());
}
