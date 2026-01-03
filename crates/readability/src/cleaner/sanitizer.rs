//! HTML sanitization and cleaning
//!
//! This module provides utilities to clean extracted HTML content,
//! removing scripts, styles, unwanted elements, and normalizing the output.
use crate::extractor::scoring::is_unlikely_candidate;

use scraper::{Html, Selector};

/// HTML cleaner and sanitizer
pub struct HtmlCleaner;

impl HtmlCleaner {
    /// Clean HTML content by applying all cleaning steps
    ///
    /// Steps:
    /// 1. Remove scripts and styles
    /// 2. Remove unlikely candidates (sidebar, comments, etc.)
    /// 3. Remove empty elements
    /// 4. Clean attributes (keep only essential ones)
    /// 5. Normalize whitespace
    pub fn clean(html: &str) -> String {
        let mut result = Self::remove_scripts_and_styles(html);
        result = Self::remove_unlikely_elements(&result);
        result = Self::remove_empty_elements(&result);
        result = Self::clean_attributes(&result);
        result = Self::normalize_whitespace(&result);
        result
    }

    /// Remove script and style tags and their contents
    pub fn remove_scripts_and_styles(html: &str) -> String {
        let document = Html::parse_fragment(html);
        let mut result = html.to_string();

        if let Ok(selector) = Selector::parse("script") {
            for element in document.select(&selector) {
                let element_html = element.html();
                result = result.replace(&element_html, "");
            }
        }

        if let Ok(selector) = Selector::parse("style") {
            for element in document.select(&selector) {
                let element_html = element.html();
                result = result.replace(&element_html, "");
            }
        }

        if let Ok(selector) = Selector::parse("noscript") {
            for element in document.select(&selector) {
                let element_html = element.html();
                result = result.replace(&element_html, "");
            }
        }

        if let Ok(selector) = Selector::parse("link[rel='stylesheet']") {
            for element in document.select(&selector) {
                let element_html = element.html();
                result = result.replace(&element_html, "");
            }
        }

        result
    }

    /// Remove elements that are unlikely to be main content
    pub fn remove_unlikely_elements(html: &str) -> String {
        let document = Html::parse_fragment(html);
        let mut result = html.to_string();

        let unlikely_selectors = [
            "nav",
            "aside",
            "footer",
            "header",
            "[role='navigation']",
            "[role='banner']",
            "[role='contentinfo']",
            "[role='complementary']",
            ".sidebar",
            ".advertisement",
            ".ad",
            ".ads",
            ".social-share",
            ".share-buttons",
            ".related-posts",
            ".comments",
            "#comments",
            ".comment-section",
        ];

        for selector_str in unlikely_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let element_html = element.html();
                    result = result.replace(&element_html, "");
                }
            }
        }

        if let Ok(div_selector) = Selector::parse("div, section, aside, span") {
            for element in document.select(&div_selector) {
                if is_unlikely_candidate(element) {
                    let element_html = element.html();
                    result = result.replace(&element_html, "");
                }
            }
        }

        result
    }

    /// Clean attributes - keep only essential ones
    ///
    /// Keeps: href, src, alt, title, datetime, class (filtered)
    /// Removes: onclick, onload, style, data-*, etc.
    pub fn clean_attributes(html: &str) -> String {
        use regex::Regex;

        let mut result = html.to_string();

        let event_attrs = [
            "onclick",
            "onload",
            "onerror",
            "onmouseover",
            "onmouseout",
            "onkeydown",
            "onkeyup",
            "onfocus",
            "onblur",
            "onsubmit",
        ];

        for attr in event_attrs {
            if let Ok(regex) = Regex::new(&format!(r#"\s+{}="[^"]*""#, attr)) {
                result = regex.replace_all(&result, "").to_string();
            }
            if let Ok(regex) = Regex::new(&format!(r#"\s+{}='[^']*'"#, attr)) {
                result = regex.replace_all(&result, "").to_string();
            }
        }

        if let Ok(regex) = Regex::new(r#"\s+style="[^"]*""#) {
            result = regex.replace_all(&result, "").to_string();
        }

        if let Ok(regex) = Regex::new(r#"\s+data-[a-z-]+="[^"]*""#) {
            result = regex.replace_all(&result, "").to_string();
        }

        result
    }

    /// Normalize whitespace - collapse multiple spaces/newlines
    pub fn normalize_whitespace(html: &str) -> String {
        use regex::Regex;

        let mut result = html.to_string();

        if let Ok(regex) = Regex::new(r"[ \t]+") {
            result = regex.replace_all(&result, " ").to_string();
        }

        if let Ok(regex) = Regex::new(r"\n{3,}") {
            result = regex.replace_all(&result, "\n\n").to_string();
        }

        if let Ok(regex) = Regex::new(r"\n{3,}") {
            result = regex.replace_all(&result, "\n\n").to_string();
        }

        if let Ok(regex) = Regex::new(r"(?m)^[ \t]+|[ \t]+$") {
            result = regex.replace_all(&result, "").to_string();
        }

        result.trim().to_string()
    }

    /// Remove empty elements (paragraphs, divs, spans with no content)
    pub fn remove_empty_elements(html: &str) -> String {
        use regex::Regex;

        let mut result = html.to_string();

        if let Ok(regex) = Regex::new(r"<p[^>]*>\s*</p>") {
            result = regex.replace_all(&result, "").to_string();
        }

        if let Ok(regex) = Regex::new(r"<div[^>]*>\s*</div>") {
            result = regex.replace_all(&result, "").to_string();
        }

        if let Ok(regex) = Regex::new(r"<span[^>]*>\s*</span>") {
            result = regex.replace_all(&result, "").to_string();
        }

        if let Ok(regex) = Regex::new(r"<p[^>]*>(&nbsp;|\s)*</p>") {
            result = regex.replace_all(&result, "").to_string();
        }

        result
    }

    /// Remove elements by class or id containing specific patterns
    pub fn remove_by_class_or_id(html: &str, patterns: &[&str]) -> String {
        let document = Html::parse_fragment(html);
        let mut result = html.to_string();

        if let Ok(all_selector) = Selector::parse("*") {
            for element in document.select(&all_selector) {
                let class_str = element.value().attr("class").unwrap_or("");
                let id_str = element.value().attr("id").unwrap_or("");
                let combined = format!("{} {}", class_str, id_str).to_lowercase();

                for pattern in patterns {
                    if combined.contains(&pattern.to_lowercase()) {
                        let element_html = element.html();
                        result = result.replace(&element_html, "");
                        break;
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_scripts() {
        let html = r#"<div>Content<script>alert('test');</script>More content</div>"#;
        let result = HtmlCleaner::remove_scripts_and_styles(html);
        assert!(!result.contains("script"));
        assert!(!result.contains("alert"));
    }

    #[test]
    fn test_remove_styles() {
        let html = r#"<div>Content<style>.test { color: red; }</style>More content</div>"#;
        let result = HtmlCleaner::remove_scripts_and_styles(html);
        assert!(!result.contains("style"));
        assert!(!result.contains("color"));
    }

    #[test]
    fn test_remove_empty_paragraphs() {
        let html = r#"<div><p>Content</p><p></p><p>   </p><p>More</p></div>"#;
        let result = HtmlCleaner::remove_empty_elements(html);
        assert!(result.contains("Content"));
        assert!(result.contains("More"));

        let p_count = result.matches("<p").count();
        assert_eq!(p_count, 2, "Should have exactly 2 paragraphs");
    }

    #[test]
    fn test_normalize_whitespace() {
        let html = "  Content    with   multiple    spaces  ";
        let result = HtmlCleaner::normalize_whitespace(html);
        assert_eq!(result, "Content with multiple spaces");
    }

    #[test]
    fn test_clean_inline_events() {
        let html = r#"<a href="\#" onclick="doSomething()">Link</a>"#;
        let result = HtmlCleaner::clean_attributes(html);
        assert!(!result.contains("onclick"));
        assert!(result.contains("href"));
    }

    #[test]
    fn test_clean_data_attributes() {
        let html = r#"<div data-tracking="abc" data-id="123">Content</div>"#;
        let result = HtmlCleaner::clean_attributes(html);
        assert!(!result.contains("data-tracking"));
        assert!(!result.contains("data-id"));
    }

    #[test]
    fn test_full_clean() {
        let html = r#"
            <div>
                <script>evil();</script>
                <style>.bad {}</style>
                <p>Good content here.</p>
                <p></p>
                <nav>Navigation</nav>
                <p onclick="bad()">More content</p>
            </div>
        "#;

        let result = HtmlCleaner::clean(html);
        assert!(!result.contains("script"));
        assert!(!result.contains("style"));
        assert!(!result.contains("onclick"));
        assert!(result.contains("Good content"));
        assert!(result.contains("More content"));
    }

    #[test]
    fn test_remove_by_class_or_id() {
        let html = r#"<div><p class="sidebar">Sidebar</p><p class="content">Main</p></div>"#;
        let result = HtmlCleaner::remove_by_class_or_id(html, &["sidebar"]);
        assert!(!result.contains("Sidebar"));
        assert!(result.contains("Main"));
    }
}
