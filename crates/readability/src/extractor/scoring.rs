//! Content scoring for the Mozilla Readability algorithm
//!
//! TODO: Implement scoring

/// Content score for an element
#[derive(Debug, Clone)]
pub struct ContentScore {
    /// Text length of the element
    pub text_length: usize,
    /// Link density (0.0 to 1.0)
    pub link_density: f32,
    /// Class/ID weight (positive for content, negative for non-content)
    pub class_weight: f32,
    /// Total calculated score
    pub total: f32,
}

/// Positive class/ID patterns indicating content
pub const POSITIVE_PATTERNS: &[&str] = &[
    "article", "body", "content", "entry", "main", "page", "post", "text", "blog", "story",
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
];
