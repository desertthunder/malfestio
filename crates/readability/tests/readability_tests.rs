use malfestio_readability::Readability;

#[tokio::test]
#[ignore = "requires network access"]
async fn test_arxiv_extraction() {
    let url = "https://arxiv.org/abs/2009.03017";

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; MalfestioBot/1.0)")
        .build()
        .unwrap();

    let response = client.get(url).send().await.unwrap();
    let html = response.text().await.unwrap();

    let readability = Readability::new(html, Some(url));
    let article = readability.parse().unwrap();

    assert!(!article.title.is_empty(), "Title should be extracted");
    println!("Title: {}", article.title);

    assert!(!article.markdown.is_empty(), "Body/markdown should be extracted");
    assert!(article.markdown.len() > 50, "Abstract should have substantial content");
    println!("Markdown length: {} chars", article.markdown.len());

    assert!(article.author.is_some(), "Author should be extracted from meta tag");
    println!("Author: {:?}", article.author);

    assert!(
        article.published_date.is_some(),
        "Date should be extracted from meta tag"
    );
    println!("Date: {:?}", article.published_date);
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_wikipedia_extraction() {
    let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; MalfestioBot/1.0)")
        .build()
        .unwrap();

    let response = client.get(url).send().await.unwrap();
    let html = response.text().await.unwrap();

    let readability = Readability::new(html, Some(url));
    let article = readability.parse().unwrap();

    assert!(article.title.contains("Rust"), "Title should contain 'Rust'");
    println!("Title: {}", article.title);

    assert!(
        article.markdown.len() > 1000,
        "Wikipedia article should have substantial content"
    );
    println!("Markdown length: {} chars", article.markdown.len());

    // Verify strip rules worked: mw-editsection elements should be removed
    assert!(
        !article.content.contains("mw-editsection"),
        "Edit section elements (mw-editsection) should be stripped"
    );
}

/// Test extraction for site without specific rules (falls back to generic)
#[tokio::test]
#[ignore = "requires network access"]
async fn test_generic_fallback_extraction() {
    let url = "https://www.rust-lang.org/";

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; MalfestioBot/1.0)")
        .build()
        .unwrap();

    let response = client.get(url).send().await.unwrap();
    let html = response.text().await.unwrap();

    let readability = Readability::new(html, Some(url));
    let article = readability.parse().unwrap();

    assert!(!article.title.is_empty(), "Title should be extracted via generic");
    assert!(!article.markdown.is_empty(), "Content should be extracted via generic");

    println!("Title: {}", article.title);
    println!("Markdown length: {} chars", article.markdown.len());
}
