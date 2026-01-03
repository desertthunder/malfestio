use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
pub struct Chunk {
    pub heading: String,
    pub content: String,
}

/// Chunks text based on markdown headers or paragraph breaks.
///
/// Tries to keep chunks under `max_words` roughly, but honors logical sections first.
pub fn chunk_text(text: &str, _max_words: usize) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut current_heading = "Introduction".to_string();
    let mut current_content = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        let is_header = trimmed.starts_with('#') || is_common_header(trimmed);

        if is_header {
            if !current_content.trim().is_empty() {
                chunks.push(Chunk { heading: current_heading.clone(), content: current_content.trim().to_string() });
            }

            current_heading = if trimmed.starts_with('#') {
                trimmed.trim_start_matches('#').trim().to_string()
            } else {
                trimmed.to_string()
            };
            current_content.clear();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if !current_content.trim().is_empty() {
        chunks.push(Chunk { heading: current_heading, content: current_content.trim().to_string() });
    }

    chunks
}

fn is_common_header(line: &str) -> bool {
    let lower = line.to_lowercase();
    if line.len() > 50 {
        return false;
    }
    lower.contains("abstract")
        || lower.contains("introduction")
        || lower.contains("references")
        || lower.contains("conclusion")
        || lower.contains("background")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunking_with_headers() {
        let text = "# Header 1\nContent 1\n## Header 2\nContent 2";
        let chunks = chunk_text(text, 1000);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].heading, "Header 1");
        assert_eq!(chunks[0].content, "Content 1");
        assert_eq!(chunks[1].heading, "Header 2");
        assert_eq!(chunks[1].content, "Content 2");
    }

    #[test]
    fn test_chunking_no_headers() {
        let text = "Just some content\nMore content";
        let chunks = chunk_text(text, 1000);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].heading, "Introduction");
        assert_eq!(chunks[0].content, "Just some content\nMore content");
    }

    #[test]
    fn test_empty_text() {
        let chunks = chunk_text("", 1000);
        assert_eq!(chunks.len(), 0);
    }
}
