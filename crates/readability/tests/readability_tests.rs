use malfestio_readability::Readability;
use std::fs;
use std::path::PathBuf;

fn get_test_html(filename: &str) -> Option<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/data");
    path.push(filename);

    if path.exists() {
        Some(fs::read_to_string(path).unwrap())
    } else {
        println!("Test data file not found: {:?}. Skipping test.", path);
        None
    }
}

#[test]
fn test_arxiv_extraction() {
    let html = match get_test_html("arxiv.html") {
        Some(h) => h,
        None => return,
    };
    let url = "https://arxiv.org/abs/2009.03017";

    let readability = Readability::new(html, Some(url));
    let article = readability.parse().unwrap();

    assert!(!article.title.is_empty(), "Title should be extracted");
    assert!(article.title.contains("Non-exponentially weighted aggregation"));

    assert!(!article.markdown.is_empty(), "Body/markdown should be extracted");
    assert!(article.markdown.len() > 50, "Abstract should have substantial content");

    // Arxiv meta tag uses "Lastname, Firstname" format: <meta name="citation_author" content="Alquier, Pierre" />
    assert_eq!(article.author.as_deref(), Some("Alquier, Pierre"));
    assert_eq!(article.published_date.as_deref(), Some("2020/09/07"));
}

#[test]
fn test_wikipedia_extraction() {
    let html = match get_test_html("wikipedia.html") {
        Some(h) => h,
        None => return,
    };
    let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";

    let readability = Readability::new(html, Some(url));
    let article = readability.parse().unwrap();

    assert!(article.title.contains("Rust"), "Title should contain 'Rust'");
    assert!(
        article.markdown.len() > 1000,
        "Wikipedia article should have substantial content"
    );

    assert!(
        !article.content.contains("mw-editsection"),
        "Edit section elements (mw-editsection) should be stripped"
    );
}

#[test]
fn test_generic_fallback_extraction() {
    let html = match get_test_html("generic.html") {
        Some(h) => h,
        None => return,
    };
    let url = "https://www.rust-lang.org/";

    let readability = Readability::new(html, Some(url));
    let article = readability.parse().unwrap();

    assert!(!article.title.is_empty(), "Title should be extracted via generic");
    assert!(!article.markdown.is_empty(), "Content should be extracted via generic");
}

#[test]
fn test_substack_extraction() {
    let html = match get_test_html("substack.html") {
        Some(h) => h,
        None => return,
    };
    let url = "https://taibbi.substack.com/p/glenn-greenwald-on-his-resignation";

    let readability = Readability::new(html, Some(url));
    let article = readability.parse().unwrap();

    assert!(!article.title.is_empty(), "Title should be extracted");
    assert!(
        article.title.contains("Glenn Greenwald"),
        "Title should match expectation"
    );
}

#[test]
fn test_theonion_extraction() {
    let html = match get_test_html("theonion.html") {
        Some(h) => h,
        None => return,
    };
    let url = "https://www.theonion.com/theresa-may-narrowly-manages-to-survive-parliamentary-f-1831077604";

    let readability = Readability::new(html, Some(url));
    let article = readability.parse().unwrap();

    assert!(!article.title.is_empty(), "Title should be extracted");
    // The onion uses JSON-LD or meta tags usually, check if our rules caught it
    // TODO: we should implement JSON-LD support
}

#[test]
fn test_readthedocs_extraction() {
    let html = match get_test_html("readthedocs.html") {
        Some(h) => h,
        None => return,
    };
    let url = "http://docs.readthedocs.io/en/latest/getting_started.html";

    let readability = Readability::new(html, Some(url));
    let article = readability.parse().unwrap();

    assert!(!article.title.is_empty(), "Title should be extracted");
}
