//! Content scoring for the Mozilla Readability algorithm
//!
//! This module implements the heuristic-based scoring system used to identify
//! main content in HTML documents. Based on the Arc90/Mozilla Readability algorithm.

use scraper::{ElementRef, Selector};

/// Content score for an element
#[derive(Debug, Clone, Default)]
pub struct ContentScore {
    /// Base score from tag type
    pub tag_score: f32,
    /// Bonus/penalty from class/id names
    pub class_weight: f32,
    /// Link density (0.0 to 1.0) - lower is better for content
    pub link_density: f32,
    /// Text length bonus
    pub text_length_bonus: f32,
    /// Comma count bonus (indicates prose)
    pub comma_bonus: f32,
    /// Total calculated score
    pub total: f32,
}

impl ContentScore {
    /// Create a new score for an element
    ///
    /// Total score is calculated as:
    ///
    /// ```text
    /// tag_score + class_weight + text_length_bonus + comma_bonus - (link_density * 10.0)
    /// ```
    ///
    /// High link density is penalized (navigation/sidebar content)
    pub fn new(element: ElementRef) -> Self {
        let tag_score = calculate_tag_score(element);
        let class_weight = calculate_class_weight(element);
        let (text_length_bonus, comma_bonus) = calculate_text_bonuses(element);
        let link_density = calculate_link_density(element);
        let total = tag_score + class_weight + text_length_bonus + comma_bonus - (link_density * 10.0);
        Self { tag_score, class_weight, link_density, text_length_bonus, comma_bonus, total }
    }
}

/// Positive class/ID patterns indicating content
pub const POSITIVE_PATTERNS: &[&str] = &[
    "article",
    "body",
    "content",
    "entry",
    "main",
    "page",
    "post",
    "text",
    "blog",
    "story",
    "hentry",
    "h-entry",
    "entry-content",
    "post-content",
    "article-content",
];

/// Negative class/ID patterns indicating non-content
pub const NEGATIVE_PATTERNS: &[&str] = &[
    "combx",
    "comment",
    "community",
    "disqus",
    "extra",
    "footer",
    "header",
    "menu",
    "remark",
    "rss",
    "share",
    "sidebar",
    "sponsor",
    "ad-",
    "agegate",
    "pagination",
    "nav",
    "related",
    "social",
    "widget",
    "promo",
    "masthead",
    "meta",
    "outbrain",
    "taboola",
];

/// Tags that are likely to contain main content
const POSITIVE_TAGS: &[&str] = &["article", "main", "section", "div", "p", "td", "pre"];

/// Tags unlikely to contain main content
const NEGATIVE_TAGS: &[&str] = &[
    "nav",
    "aside",
    "footer",
    "header",
    "form",
    "iframe",
    "figure",
    "figcaption",
];

/// Calculate base score from element tag name
fn calculate_tag_score(element: ElementRef) -> f32 {
    let tag_name = element.value().name();

    for tag in POSITIVE_TAGS {
        if tag_name == *tag {
            return match *tag {
                "article" => 10.0,
                "main" => 8.0,
                "section" => 5.0,
                "div" => 5.0,
                "p" => 3.0,
                "pre" => 3.0,
                "td" => 3.0,
                _ => 0.0,
            };
        }
    }

    for tag in NEGATIVE_TAGS {
        if tag_name == *tag {
            return -5.0;
        }
    }

    0.0
}

/// Calculate class/id weight based on positive/negative patterns
pub fn calculate_class_weight(element: ElementRef) -> f32 {
    let mut weight: f32 = 0.0;

    let class_str = element.value().attr("class").unwrap_or("");
    let id_str = element.value().attr("id").unwrap_or("");
    let combined = format!("{} {}", class_str, id_str).to_lowercase();
    for pattern in POSITIVE_PATTERNS {
        if combined.contains(pattern) {
            weight += 25.0;
        }
    }

    for pattern in NEGATIVE_PATTERNS {
        if combined.contains(pattern) {
            weight -= 25.0;
        }
    }

    weight
}

/// Calculate text length and comma bonuses
fn calculate_text_bonuses(element: ElementRef) -> (f32, f32) {
    let text: String = element.text().collect();
    let text_length = text.len();
    let comma_count = text.matches(',').count();

    let text_length_bonus = ((text_length as f32).sqrt() / 5.0).min(10.0);
    let comma_bonus = (comma_count as f32).min(3.0);

    (text_length_bonus, comma_bonus)
}

/// Calculate link density (ratio of link text to total text)
pub fn calculate_link_density(element: ElementRef) -> f32 {
    let text: String = element.text().collect();
    let total_length = text.len();

    if total_length == 0 {
        return 0.0;
    }

    let mut link_length = 0usize;

    if let Ok(selector) = Selector::parse("a") {
        for link in element.select(&selector) {
            let link_text: String = link.text().collect();
            link_length += link_text.len();
        }
    }

    link_length as f32 / total_length as f32
}

/// Check if an element is an "unlikely candidate" (sidebar, comment, etc.)
pub fn is_unlikely_candidate(element: ElementRef) -> bool {
    let class_str = element.value().attr("class").unwrap_or("");
    let id_str = element.value().attr("id").unwrap_or("");
    let combined = format!("{} {}", class_str, id_str).to_lowercase();

    for pattern in NEGATIVE_PATTERNS {
        if combined.contains(pattern) {
            for positive in POSITIVE_PATTERNS {
                if combined.contains(positive) {
                    return false;
                }
            }
            return true;
        }
    }

    false
}

/// Check if an element has enough content to be a candidate
pub fn is_viable_candidate(element: ElementRef) -> bool {
    let text: String = element.text().collect();
    let text_length = text.len();

    if text_length < 25 {
        return false;
    }
    if let Ok(selector) = Selector::parse("p") {
        let p_count = element.select(&selector).count();
        if p_count > 0 {
            return true;
        }
    }

    text_length >= 100
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_positive_patterns_detection() {
        let html = r#"<div id="content" class="article-body">Test content</div>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("div").unwrap();
        let element = document.select(&selector).next().unwrap();
        let weight = calculate_class_weight(element);
        assert!(weight > 0.0, "Should have positive weight for content/article classes");
    }

    #[test]
    fn test_negative_patterns_detection() {
        let html = r#"<div id="sidebar" class="comment-section">Test content</div>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("div").unwrap();
        let element = document.select(&selector).next().unwrap();
        let weight = calculate_class_weight(element);
        assert!(weight < 0.0, "Should have negative weight for sidebar/comment classes");
    }

    #[test]
    fn test_link_density_calculation() {
        let html = r#"<div>Some text here <a href="\#">link one</a> and <a href="\#">link two</a> more text</div>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("div").unwrap();
        let element = document.select(&selector).next().unwrap();
        let density = calculate_link_density(element);
        assert!(density > 0.0 && density < 1.0, "Link density should be between 0 and 1");
    }

    #[test]
    fn test_high_link_density() {
        let html = r#"<div><a href="\#">link</a><a href="\#">link</a><a href="\#">link</a></div>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("div").unwrap();
        let element = document.select(&selector).next().unwrap();
        let density = calculate_link_density(element);
        assert!(
            density > 0.8,
            "Should detect high link density in navigation-like content"
        );
    }

    #[test]
    fn test_unlikely_candidate() {
        let html = r#"<div class="sidebar">Sidebar content</div>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("div").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert!(is_unlikely_candidate(element), "Sidebar should be unlikely candidate");
    }

    #[test]
    fn test_viable_candidate_with_paragraphs() {
        let html = r#"<div><p>This is a paragraph with enough content to be considered viable.</p></div>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("div").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert!(is_viable_candidate(element), "Div with paragraph should be viable");
    }

    #[test]
    fn test_content_score_creation() {
        let html =
            r#"<article class="post-content"><p>This is article content with some commas, here, there.</p></article>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("article").unwrap();
        let element = document.select(&selector).next().unwrap();

        let score = ContentScore::new(element);
        assert!(score.tag_score > 0.0, "Article tag should have positive score");
        assert!(score.class_weight > 0.0, "post-content class should be positive");
        assert!(score.comma_bonus > 0.0, "Should detect commas");
    }

    #[test]
    fn test_tag_score_article() {
        let html = r#"<article>Content</article>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("article").unwrap();
        let element = document.select(&selector).next().unwrap();

        let score = calculate_tag_score(element);
        assert_eq!(score, 10.0, "Article tag should score 10");
    }

    #[test]
    fn test_tag_score_nav() {
        let html = r#"<nav>Navigation</nav>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("nav").unwrap();
        let element = document.select(&selector).next().unwrap();

        let score = calculate_tag_score(element);
        assert_eq!(score, -5.0, "Nav tag should score -5");
    }
    #[test]
    fn test_mixed_signals() {
        let html = r#"<div class="sidebar article-content">Content</div>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("div").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert!(
            !is_unlikely_candidate(element),
            "Mixed signals with positive pattern should be valid"
        );
    }

    #[test]
    fn test_empty_link_density() {
        let html = r#"<div></div>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("div").unwrap();
        let element = document.select(&selector).next().unwrap();

        assert_eq!(calculate_link_density(element), 0.0);
    }
}
