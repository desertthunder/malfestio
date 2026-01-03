//! Generic content extraction using Mozilla Readability-style heuristics
//!
//! ## Implementation Overview
//!
//! This module implements a content extraction algorithm inspired by Mozilla's Readability.js.
//! It uses heuristic-based scoring to identify the main content of a web page.
//!
//! ### Algorithm Steps:
//! 1. **Preprocessing**: Remove scripts, styles, and other noise
//! 2. **Candidate Identification**: Find elements containing paragraphs
//! 3. **Content Scoring**: Score candidates based on text length, link density, classes
//! 4. **Ancestor Propagation**: Bubble scores up to parent/grandparent elements
//! 5. **Top Candidate Selection**: Pick the highest-scoring element
//! 6. **Sibling Inclusion**: Include relevant siblings of the top candidate
//! 7. **Cleaning**: Remove unlikely elements and normalize output
//!
//! ### Scoring Factors:
//! - Tag type (article > main > div > p)
//! - Class/ID names (positive: "article", "content"; negative: "sidebar", "nav")
//! - Text length (longer content scores higher)
//! - Link density (high link density = navigation, low = content)
//! - Comma count (commas indicate prose)

use crate::cleaner::HtmlCleaner;
use crate::error::{Error, Result};
use crate::extractor::scoring::{ContentScore, calculate_link_density, is_unlikely_candidate, is_viable_candidate};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

/// Extracted content from generic algorithm
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub title: String,
    pub body_html: String,
    pub author: Option<String>,
    pub date: Option<String>,
}

/// Candidate element with its score
#[derive(Debug)]
struct ScoredCandidate {
    /// Index in the candidate list
    _index: usize,
    /// Computed content score
    score: f32,
    /// HTML content
    html: String,
}

/// Generic content extractor using Readability-style heuristics
///
/// This extractor attempts to find article content using content scoring.
/// It's designed as a fallback when site-specific XPath rules are not available.
pub struct GenericExtractor {
    html: String,
}

impl GenericExtractor {
    /// Create a new generic extractor
    pub fn new(html: String) -> Self {
        Self { html }
    }

    /// Extract content using content scoring algorithm
    ///
    /// Strategy
    ///
    /// 1. Preprocess HTML (remove scripts, styles)
    /// 2. Extract metadata (title, author, date) from standard locations
    /// 3. Find content candidates (elements with paragraphs)
    /// 4. Score candidates and select the best one
    /// 5. Clean and return the content
    pub fn extract(&self) -> Result<ExtractedContent> {
        let document = Html::parse_document(&self.html);

        let title = self
            .extract_title(&document)
            .ok_or_else(|| Error::ExtractionError("Could not extract title".to_string()))?;

        let author = self.extract_author(&document);
        let date = self.extract_date(&document);

        let body_html = self
            .extract_body_with_scoring(&document)
            .or_else(|| self.extract_body_simple(&document))
            .ok_or_else(|| Error::ExtractionError("Could not extract body content".to_string()))?;

        let clean_body = HtmlCleaner::clean(&body_html);

        Ok(ExtractedContent { title, body_html: clean_body, author, date })
    }

    /// Extract body content using content scoring algorithm
    fn extract_body_with_scoring(&self, document: &Html) -> Option<String> {
        let candidates = self.find_candidates(document);

        if candidates.is_empty() {
            return None;
        }

        let mut scored: Vec<ScoredCandidate> = candidates
            .iter()
            .enumerate()
            .map(|(index, element)| {
                let score = ContentScore::new(*element);
                ScoredCandidate { _index: index, score: score.total, html: element.html() }
            })
            .collect();

        let mut ancestor_scores: HashMap<usize, f32> = HashMap::new();
        for (i, candidate) in scored.iter().enumerate() {
            if i > 0 {
                *ancestor_scores.entry(i - 1).or_insert(0.0) += candidate.score;
            }
            if i > 1 {
                *ancestor_scores.entry(i - 2).or_insert(0.0) += candidate.score / 2.0;
            }
        }

        for (index, bonus) in ancestor_scores {
            if let Some(candidate) = scored.get_mut(index) {
                candidate.score += bonus;
            }
        }

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.first().map(|c| c.html.clone())
    }

    /// Find candidate elements for content extraction
    fn find_candidates<'a>(&self, document: &'a Html) -> Vec<ElementRef<'a>> {
        let mut candidates: Vec<ElementRef<'a>> = Vec::new();

        let container_selectors = ["article", "main", "section", "div", "[role='main']"];

        for selector_str in container_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if is_unlikely_candidate(element) {
                        continue;
                    }
                    if is_viable_candidate(element) {
                        let density = calculate_link_density(element);
                        if density < 0.5 {
                            candidates.push(element);
                        }
                    }
                }
            }
        }

        candidates
    }

    /// Simple fallback body extraction using common patterns
    fn extract_body_simple(&self, document: &Html) -> Option<String> {
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

    /// Extract title from document
    ///
    /// Tries in order:
    /// 1. `og:title` meta tag (usually cleanest)
    /// 2. `<h1>` tag within likely content area
    /// 3. `<title>` tag content (may include site name)
    fn extract_title(&self, document: &Html) -> Option<String> {
        if let Ok(selector) = Selector::parse("meta[property='og:title']")
            && let Some(element) = document.select(&selector).next()
            && let Some(content) = element.value().attr("content")
            && !content.trim().is_empty()
        {
            return Some(content.trim().to_string());
        }

        for container in ["article h1", "main h1", "h1"] {
            if let Ok(selector) = Selector::parse(container)
                && let Some(element) = document.select(&selector).next()
            {
                let text: String = element.text().collect();
                if !text.trim().is_empty() {
                    return Some(text.trim().to_string());
                }
            }
        }

        if let Ok(selector) = Selector::parse("title")
            && let Some(element) = document.select(&selector).next()
        {
            let text: String = element.text().collect();
            if !text.trim().is_empty() {
                let title = Self::clean_title(&text);
                return Some(title);
            }
        }

        None
    }

    /// Clean title by removing common site name suffixes
    fn clean_title(title: &str) -> String {
        let title = title.trim();
        let separators = [" | ", " - ", " — ", " :: ", " » ", " · "];

        for sep in separators {
            if let Some(pos) = title.find(sep) {
                let candidate = title[..pos].trim();
                if candidate.len() > 10 {
                    return candidate.to_string();
                }
            }
        }

        title.to_string()
    }

    /// Extract author from document
    ///
    /// Tries in order:
    /// 1. `<meta name="author">` tag
    /// 2. `<meta property="og:author">` tag
    /// 3. `<meta property="article:author">` tag
    /// 4. Element with class "author", "byline", or "by"
    /// 5. Schema.org author markup
    fn extract_author(&self, document: &Html) -> Option<String> {
        let meta_selectors = vec![
            "meta[name='author']",
            "meta[property='og:author']",
            "meta[property='article:author']",
            "[itemprop='author']",
            "[rel='author']",
        ];

        for selector_str in meta_selectors {
            if let Ok(selector) = Selector::parse(selector_str)
                && let Some(element) = document.select(&selector).next()
            {
                if let Some(content) = element.value().attr("content")
                    && !content.trim().is_empty()
                {
                    return Some(content.trim().to_string());
                }

                let text: String = element.text().collect();
                if !text.trim().is_empty() {
                    return Some(text.trim().to_string());
                }
            }
        }

        let class_selectors = vec![".author", ".byline", ".by", ".post-author", ".entry-author"];

        for selector_str in class_selectors {
            if let Ok(selector) = Selector::parse(selector_str)
                && let Some(element) = document.select(&selector).next()
            {
                let text: String = element.text().collect();
                if !text.trim().is_empty() {
                    return Some(Self::clean_author(&text));
                }
            }
        }

        None
    }

    /// Clean author text (remove "By " prefix, etc.)
    fn clean_author(author: &str) -> String {
        let author = author.trim();

        let prefixes = ["By ", "by ", "Author: ", "Written by "];
        for prefix in prefixes {
            if let Some(rest) = author.strip_prefix(prefix) {
                return rest.trim().to_string();
            }
        }

        author.to_string()
    }

    /// Extract publication date from document
    ///
    /// Tries in order:
    /// 1. `<meta property="article:published_time">` (Open Graph)
    /// 2. `<meta itemprop="datePublished">` (Schema.org)
    /// 3. `<time datetime="...">` attribute
    /// 4. `<time>` element text content
    fn extract_date(&self, document: &Html) -> Option<String> {
        let meta_selectors = vec![
            "meta[property='article:published_time']",
            "meta[itemprop='datePublished']",
            "meta[name='date']",
            "meta[name='DC.date.issued']",
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

        if let Ok(selector) = Selector::parse("[itemprop='datePublished']")
            && let Some(element) = document.select(&selector).next()
        {
            if let Some(datetime) = element.value().attr("datetime")
                && !datetime.trim().is_empty()
            {
                return Some(datetime.trim().to_string());
            }

            if let Some(content) = element.value().attr("content")
                && !content.trim().is_empty()
            {
                return Some(content.trim().to_string());
            }
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
    fn test_extract_title_from_og() {
        let html = r#"
            <html>
                <head>
                    <meta property="og:title" content="OG Title">
                    <title>Page Title | Site Name</title>
                </head>
                <body></body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let title = extractor.extract_title(&document);

        assert_eq!(title, Some("OG Title".to_string()));
    }

    #[test]
    fn test_clean_title_suffix() {
        let title = "My Article Title | Some News Site";
        let cleaned = GenericExtractor::clean_title(title);
        assert_eq!(cleaned, "My Article Title");
    }

    #[test]
    fn test_extract_title_from_h1() {
        let html = r#"
            <html>
                <body>
                    <article>
                        <h1>Article Heading</h1>
                    </article>
                </body>
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
                <head><title>Test</title></head>
                <body>
                    <nav>Navigation here</nav>
                    <article>
                        <p>This is the main article content with enough text to be considered viable content.</p>
                        <p>Another paragraph here with more content to ensure we have substantial text.</p>
                    </article>
                    <aside>Sidebar content</aside>
                </body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let result = extractor.extract().unwrap();

        assert!(result.body_html.contains("main article content"));
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
    fn test_extract_author_from_byline() {
        let html = r#"
            <html>
                <body>
                    <span class="byline">By Jane Smith</span>
                </body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let author = extractor.extract_author(&document);

        assert_eq!(author, Some("Jane Smith".to_string()));
    }

    #[test]
    fn test_extract_date_from_meta() {
        let html = r#"
            <html>
                <head>
                    <meta property="article:published_time" content="2024-01-15T10:30:00Z">
                </head>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let date = extractor.extract_date(&document);

        assert_eq!(date, Some("2024-01-15T10:30:00Z".to_string()));
    }

    #[test]
    fn test_full_extraction() {
        let html = r#"
            <html>
                <head>
                    <meta property="og:title" content="Test Article">
                    <meta name="author" content="Jane Smith">
                    <meta property="article:published_time" content="2024-01-15">
                </head>
                <body>
                    <header>Site Header</header>
                    <article>
                        <h1>Article Title</h1>
                        <p>This is the main article content. It contains several paragraphs of text that make up the body of the article. The content should be substantial enough to score well.</p>
                        <p>This is another paragraph with additional content. More words here to ensure we have a proper article body.</p>
                    </article>
                    <footer>Site Footer</footer>
                </body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let result = extractor.extract().unwrap();

        assert_eq!(result.title, "Test Article");
        assert!(result.body_html.contains("main article content"));
        assert_eq!(result.author, Some("Jane Smith".to_string()));
        assert_eq!(result.date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_find_candidates_skips_nav() {
        let html = r#"
            <html>
                <body>
                    <nav class="navigation">
                        <p>Nav item 1</p>
                        <p>Nav item 2</p>
                    </nav>
                    <article>
                        <p>Real content here that should be selected as the main candidate.</p>
                    </article>
                </body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let candidates = extractor.find_candidates(&document);

        assert!(!candidates.is_empty());
        for candidate in &candidates {
            assert_ne!(candidate.value().name(), "nav");
        }
    }

    #[test]
    fn test_scored_extraction_prefers_article() {
        let html = r#"
            <html>
                <head><title>Test</title></head>
                <body>
                    <div class="sidebar">
                        <p>Sidebar content here.</p>
                    </div>
                    <article class="post-content">
                        <p>This is the main article content with plenty of text to score well in the content scoring algorithm.</p>
                        <p>Multiple paragraphs help boost the score significantly.</p>
                    </article>
                </body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let result = extractor.extract().unwrap();

        assert!(result.body_html.contains("main article content"));
    }
    #[test]
    fn test_extract_body_simple_fallback() {
        let html = r#"
            <html>
                <body>
                    <div class="article-content">
                        Short content.
                    </div>
                </body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let body = extractor.extract_body_simple(&document);

        assert!(body.is_some());
        assert!(body.unwrap().contains("Short content"));
    }

    #[test]
    fn test_extract_title_fallback_tag() {
        let html = r#"
            <html>
                <head>
                    <title>Fallback Title</title>
                </head>
                <body></body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let title = extractor.extract_title(&document);

        assert_eq!(title, Some("Fallback Title".to_string()));
    }

    #[test]
    fn test_extract_date_fallback_time_element() {
        let html = r#"
            <html>
                <body>
                    <time datetime="2025-12-25">Christmas 2025</time>
                </body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let date = extractor.extract_date(&document);
        assert_eq!(date, Some("2025-12-25".to_string()));
    }

    #[test]
    fn test_extract_date_fallback_schema() {
        let html = r#"
            <html>
                <body>
                    <span itemprop="datePublished" content="2025-01-01">Jan 1st</span>
                </body>
            </html>
        "#;

        let extractor = GenericExtractor::new(html.to_string());
        let document = Html::parse_document(html);
        let date = extractor.extract_date(&document);
        assert_eq!(date, Some("2025-01-01".to_string()));
    }
}
