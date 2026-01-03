use anyhow::Result;
use std::path::Path;

pub mod chunker;
pub mod docx;
pub mod pdf;

/// Trait for parsing documents (PDF, DOCX, etc.) and extracting text.
pub trait DocumentParser {
    /// Parse the document at the given path and return the extracted text.
    fn parse(&self, path: &Path) -> Result<String>;
}
