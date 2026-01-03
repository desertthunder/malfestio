use super::DocumentParser;
use anyhow::{Context, Result};
use std::path::Path;

pub struct PdfParser;

impl DocumentParser for PdfParser {
    fn parse(&self, path: &Path) -> Result<String> {
        let text =
            pdf_extract::extract_text(path).with_context(|| format!("Failed to extract text from PDF: {:?}", path))?;
        Ok(text)
    }
}
