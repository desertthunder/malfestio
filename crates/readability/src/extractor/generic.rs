//! Generic content extraction with a simplified heuristic-based approach
//!
//! ## Implementation Strategy
//!
//! This is a **simplified** content extractor, not a full Mozilla Readability implementation.
//! It uses basic heuristics to find common patterns in HTML documents.
//!
//! ### What This Implementation Does:
//! - Extracts title from `<title>`, `<h1>`, or `og:title` meta tags
//! - Finds body content by looking for semantic HTML5 tags and common class names
//! - Extracts author from meta tags or common byline patterns
//! - Extracts date from meta tags or `<time>` elements
//! - Uses simple CSS selector patterns (no complex scoring algorithm)
//!
//! ### What This Implementation Does NOT Do (Implementation Gaps):
//! - **No content scoring**: Unlike Mozilla Readability, we don't score paragraphs by
//!   text length, link density, or class names to find the "best" content candidate
//! - **No sibling inclusion**: We don't check if siblings of the main content should
//!   be included based on similarity thresholds
//! - **No ancestor scoring**: We don't propagate scores up the DOM tree
//! - **No link density checking**: We don't filter out high link-density sections
//! - **No "unlikely candidate" removal**: We don't remove elements based on negative
//!   class name patterns like "sidebar", "comment", etc.
//! - **Limited fallback chain**: Mozilla Readability tries multiple strategies; we try
//!   a few common patterns and give up
//!
//! ### Design Decisions:
//! - **Semantic HTML first**: We prefer `<article>`, `<main>` over class-based selection
//!   because they're more reliable indicators of content
//! - **Multiple fallbacks**: We try progressively broader selectors to maximize success rate
//! - **Metadata from standards**: We use standard meta tags (Open Graph, Schema.org, etc.)
//!   before falling back to heuristics
//! - **Fail fast**: If we can't find content with our heuristics, we return an error
//!   rather than returning garbage content
//!
//! ## TODOs:
//! - TODO: Implement basic content scoring (count paragraphs, text length)
//! - TODO: Add link density checks to filter navigation/sidebar
//! - TODO: Remove unlikely candidates (ads, footers, etc.) by class name
//! - TODO: Try multiple content candidates and pick the best one
//! - TODO: Clean extracted HTML (remove scripts, styles, empty elements)
//! - TODO: Handle multi-page articles (pagination detection)

use crate::error::{Error, Result};
use scraper::{Html, Selector};

/// Extracted content from generic algorithm
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub title: String,
    pub body_html: String,
    pub author: Option<String>,
    pub date: Option<String>,
}

/// Generic content extractor using simple heuristics
///
/// This extractor attempts to find article content using common HTML patterns.
/// It's designed as a fallback when site-specific XPath rules are not available.
pub struct GenericExtractor {
    html: String,
}

impl GenericExtractor {
    /// Create a new generic extractor
    pub fn new(html: String) -> Self {
        Self { html }
    }

    /// Extract content using simple heuristics
    ///
    /// ## Extraction Strategy:
    /// 1. Title: `<title>` tag, then `<h1>`, then `og:title` meta tag
    /// 2. Body: `<article>`, then `<main>`, then `[role="main"]`, then `.content`
    /// 3. Author: meta tags (author, og:author, article:author), then `.byline`
    /// 4. Date: meta tags (article:published_time, datePublished), then `<time>`
    ///
    /// ## Limitations:
    /// - Returns first match, doesn't evaluate quality
    /// - No cleaning of extracted HTML (scripts, ads, etc. may be included)
    /// - May extract wrong content if page structure is unusual
    pub fn extract(&self) -> Result<ExtractedContent> {
        let document = Html::parse_document(&self.html);

        let title = self
            .extract_title(&document)
            .ok_or_else(|| Error::ExtractionError("Could not extract title".to_string()))?;

        let body_html = self
            .extract_body(&document)
            .ok_or_else(|| Error::ExtractionError("Could not extract body content".to_string()))?;

        let author = self.extract_author(&document);
        let date = self.extract_date(&document);
        Ok(ExtractedContent { title, body_html, author, date })
    }

    /// Extract title from document
    ///
    /// Tries in order:
    /// 1. `<title>` tag content (cleaned of site suffixes)
    /// 2. First `<h1>` tag
    /// 3. `og:title` meta tag
    ///
    /// ## Implementation Gap:
    /// - Doesn't try to clean title (remove " | Site Name" suffixes, etc.)
    /// - Doesn't check title quality or length
    fn extract_title(&self, document: &Html) -> Option<String> {
        if let Ok(selector) = Selector::parse("title")
            && let Some(element) = document.select(&selector).next()
        {
            let text: String = element.text().collect();
            if !text.trim().is_empty() {
                return Some(text.trim().to_string());
            }
        }

        if let Ok(selector) = Selector::parse("h1")
            && let Some(element) = document.select(&selector).next()
        {
            let text: String = element.text().collect();
            if !text.trim().is_empty() {
                return Some(text.trim().to_string());
            }
        }

        if let Ok(selector) = Selector::parse("meta[property='og:title']")
            && let Some(element) = document.select(&selector).next()
            && let Some(content) = element.value().attr("content")
            && !content.trim().is_empty()
        {
            return Some(content.trim().to_string());
        }

        None
    }

    /// Extract body content from document
    ///
    /// Tries in order:
    /// 1. `<article>` tag (semantic HTML5)
    /// 2. `<main>` tag (semantic HTML5)
    /// 3. `[role="main"]` attribute (ARIA landmark)
    /// 4. First element with class containing "content", "article", "post", "entry"
    /// 5. `<body>` tag as last resort (usually includes nav, footer, etc.)
    ///
    /// ## Implementation Gaps:
    /// - Doesn't score multiple candidates to find the best one
    /// - Doesn't clean the HTML (may include ads, sidebars, etc.)
    /// - Doesn't check content length or quality
    /// - Doesn't exclude navigation, footers, comments within the selected element
    /// - Returns inner HTML as-is without any processing
    ///
    /// TODO: Add basic cleaning (remove script, style, nav, footer, aside)
    /// TODO: Check content length (minimum threshold)
    /// TODO: If multiple candidates, pick the one with most <p> tags
    fn extract_body(&self, document: &Html) -> Option<String> {
        let selectors = vec![
            "article",
            "main",
            "[role='main']",
            "[class*='content']",
            "[class*='article']",
            "[class*='post']",
            "[class*='entry']",
            "body",
        ];

        for selector_str in selectors {
            if let Ok(selector) = Selector::parse(selector_str)
                && let Some(element) = document.select(&selector).next()
            {
                let html = element.html();
                if !html.trim().is_empty() {
                    return Some(html);
                }
            }
        }

        None
    }

    /// Extract author from document
    ///
    /// Tries in order:
    /// 1. `<meta name="author">` tag
    /// 2. `<meta property="og:author">` tag
    /// 3. `<meta property="article:author">` tag
    /// 4. Element with class "author", "byline", or "by"
    ///
    /// ## Implementation Gaps:
    /// - Doesn't parse structured data (JSON-LD, Schema.org)
    /// - Doesn't extract from "By John Doe" patterns in text
    /// - Returns first match without validation
    fn extract_author(&self, document: &Html) -> Option<String> {
        let meta_selectors = vec![
            "meta[name='author']",
            "meta[property='og:author']",
            "meta[property='article:author']",
        ];

        for selector_str in meta_selectors {
            if let Ok(selector) = Selector::parse(selector_str)
                && let Some(element) = document.select(&selector).next()
                && let Some(content) = element.value().attr("content")
                && !content.trim().is_empty()
            {
                return Some(content.trim().to_string());
            }
        }

        let class_selectors = vec![".author", ".byline", ".by"];

        for selector_str in class_selectors {
            if let Ok(selector) = Selector::parse(selector_str)
                && let Some(element) = document.select(&selector).next()
            {
                let text: String = element.text().collect();
                if !text.trim().is_empty() {
                    return Some(text.trim().to_string());
                }
            }
        }

        None
    }

    /// Extract publication date from document
    ///
    /// Tries in order:
    /// 1. `<meta property="article:published_time">` (Open Graph)
    /// 2. `<meta itemprop="datePublished">` (Schema.org)
    /// 3. `<time datetime="...">` attribute
    /// 4. `<time>` element text content
    ///
    /// ## Implementation Gaps:
    /// - Doesn't parse or normalize date formats
    /// - Doesn't validate date values
    /// - Doesn't extract from text patterns ("Published on Jan 1, 2020")
    fn extract_date(&self, document: &Html) -> Option<String> {
        let meta_selectors = vec![
            "meta[property='article:published_time']",
            "meta[itemprop='datePublished']",
        ];

        for selector_str in meta_selectors {
            if let Ok(selector) = Selector::parse(selector_str)
                && let Some(element) = document.select(&selector).next()
                && let Some(content) = element.value().attr("content")
                && !content.trim().is_empty()
            {
                return Some(content.trim().to_string());
            }
        }

        if let Ok(selector) = Selector::parse("time[datetime]")
            && let Some(element) = document.select(&selector).next()
            && let Some(datetime) = element.value().attr("datetime")
            && !datetime.trim().is_empty()
        {
            return Some(datetime.trim().to_string());
        }

        if let Ok(selector) = Selector::parse("time")
            && let Some(element) = document.select(&selector).next()
        {
            let text: String = element.text().collect();
            if !text.trim().is_empty() {
                return Some(text.trim().to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_from_title_tag() {
        let html = r#"
            <html>
                <head><title>Test Article Title</title></head>
                <body></body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let title = extractor.extract_title(&document);

        assert_eq!(title, Some("Test Article Title".to_string()));
    }

    #[test]
    fn test_extract_title_from_h1() {
        let html = r#"
            <html>
                <body><h1>Article Heading</h1></body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let title = extractor.extract_title(&document);

        assert_eq!(title, Some("Article Heading".to_string()));
    }

    #[test]
    fn test_extract_body_from_article() {
        let html = r#"
            <html>
                <body>
                    <article>
                        <p>This is the article content.</p>
                    </article>
                </body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let body = extractor.extract_body(&document);

        assert!(body.is_some());
        assert!(body.unwrap().contains("This is the article content"));
    }

    #[test]
    fn test_extract_author_from_meta() {
        let html = r#"
            <html>
                <head>
                    <meta name="author" content="John Doe">
                </head>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let author = extractor.extract_author(&document);

        assert_eq!(author, Some("John Doe".to_string()));
    }

    #[test]
    fn test_extract_date_from_meta() {
        let html = r#"
            <html>
                <head>
                    <meta property="article:published_time" content="2024-01-15">
                </head>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let date = extractor.extract_date(&document);

        assert_eq!(date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_full_extraction() {
        let html = r#"
            <html>
                <head>
                    <title>Test Article</title>
                    <meta name="author" content="Jane Smith">
                    <meta property="article:published_time" content="2024-01-15">
                </head>
                <body>
                    <article>
                        <h1>Article Title</h1>
                        <p>Article content goes here.</p>
                    </article>
                </body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let result = extractor.extract().unwrap();

        assert_eq!(result.title, "Test Article");
        assert!(result.body_html.contains("Article content goes here"));
        assert_eq!(result.author, Some("Jane Smith".to_string()));
        assert_eq!(result.date, Some("2024-01-15".to_string()));
    }
}
