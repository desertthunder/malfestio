use malfestio_server::import::{DocumentParser, docx::DocxParser, pdf::PdfParser};
use std::path::PathBuf;

fn get_test_data_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/data");
    path.push(filename);
    path
}

#[test]
fn test_pdf_extraction() {
    let path = get_test_data_path("1904.09828v2.pdf");
    assert!(path.exists(), "Test PDF not found at {:?}", path);

    let parser = PdfParser;
    let result = parser.parse(&path);
    assert!(result.is_ok(), "PDF parsing should succeed");

    let content = result.unwrap();
    assert!(!content.is_empty(), "Extracted content should not be empty");
    assert!(
        content.contains("Magic: The Gathering"),
        "Content should contain 'Magic: The Gathering'"
    );
    assert!(
        content.contains("Turing Complete"),
        "Content should contain 'Turing Complete'"
    );

    assert!(
        content.contains("Alex Churchill"),
        "Content should contain author 'Alex Churchill'"
    );

    let content_lower = content.to_lowercase();
    assert!(content_lower.contains("abstract"), "Content should contain 'Abstract'");

    assert!(
        content_lower.contains("introduction"),
        "Content should contain 'Introduction'"
    );
    assert!(
        content_lower.contains("references"),
        "Content should contain 'References'"
    );

    assert!(
        content.len() > 5000,
        "Content should be substantial (likely > 5000 chars)"
    );
}

#[test]
fn test_docx_stub_extraction() {
    let path = get_test_data_path("dummy.docx");
    let parser = DocxParser;
    let result = parser.parse(&path);

    assert!(result.is_ok(), "DOCX stub should return Ok");
    let content = result.unwrap();
    assert!(
        content.contains("not yet implemented"),
        "Content should indicate stub implementation"
    );
}

#[test]
fn test_pdf_missing_file() {
    let path = get_test_data_path("non_existent.pdf");
    let parser = PdfParser;
    let result = parser.parse(&path);
    assert!(result.is_err(), "Parsing missing file should return error");
}
