//! Markdown conversion using html2md crate

/// Convert HTML to Markdown
pub fn to_markdown(html: &str) -> String {
    html2md::parse_html(html)
}

/// Generate an excerpt from markdown (first ~200 chars)
pub fn generate_excerpt(markdown: &str, max_length: usize) -> String {
    let cleaned: String = markdown.chars().filter(|c| !c.is_control() || *c == '\n').collect();

    if cleaned.len() <= max_length {
        cleaned
    } else {
        let truncated = &cleaned[..max_length];
        format!("{}...", truncated.trim_end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_excerpt() {
        let markdown =
            "This is a long piece of markdown text that should be truncated to approximately 200 characters or so.";
        let excerpt = generate_excerpt(markdown, 50);
        assert!(excerpt.len() <= 53);
        assert!(excerpt.ends_with("..."));
    }

    #[test]
    fn test_generate_excerpt_short() {
        let markdown = "Short text";
        let excerpt = generate_excerpt(markdown, 50);
        assert_eq!(excerpt, "Short text");
    }
}
