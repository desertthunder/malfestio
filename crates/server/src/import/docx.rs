use super::DocumentParser;
use anyhow::Result;
use std::path::Path;

pub struct DocxParser;

impl DocumentParser for DocxParser {
    fn parse(&self, path: &Path) -> Result<String> {
        // TODO: Implement DOCX parsing using docx-rs
        Ok(format!(
            "DOCX parsing not yet implemented. File: {:?}",
            path.file_name()
        ))
    }
}
