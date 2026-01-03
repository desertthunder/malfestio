use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::post,
};
use reqwest::header::CONTENT_TYPE;
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;
use tower::ServiceExt;

fn get_test_data_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/data");
    path.push(filename);
    path
}

#[tokio::test]
async fn test_import_lecture_pdf() {
    let app = Router::new().route(
        "/api/import/lecture",
        post(malfestio_server::api::importer::post_import_lecture),
    );

    let path = get_test_data_path("1904.09828v2.pdf");
    let file_bytes = fs::read(&path).await.expect("Failed to read test PDF");
    let boundary = "------------------------boundary123";
    let body_data = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"lecture.pdf\"\r\nContent-Type: application/pdf\r\n\r\n",
        boundary = boundary
    );
    let mut full_body = body_data.into_bytes();
    full_body.extend_from_slice(&file_bytes);
    full_body.extend_from_slice(format!("\r\n--{boundary}--\r\n", boundary = boundary).as_bytes());

    let req = Request::builder()
        .method("POST")
        .uri("/api/import/lecture")
        .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
        .body(Body::from(full_body))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let status = response.status();

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();

    if status != StatusCode::OK {
        let body_str = String::from_utf8_lossy(&body_bytes);
        println!("Test PDF Failed. Status: {}, Body: {}", status, body_str);
        panic!("Status was not 200 OK");
    }

    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body["filename"], "lecture.pdf");

    let content = body["content"].as_str().unwrap();
    assert!(content.contains("Magic: The Gathering"));

    let chunks = body["chunks"].as_array().unwrap();
    assert!(!chunks.is_empty(), "Should have at least one chunk");

    let has_abstract = chunks.iter().any(|c| {
        c["heading"].as_str().unwrap_or("").to_lowercase().contains("abstract")
            || c["content"].as_str().unwrap_or("").contains("Abstract")
    });
    assert!(has_abstract, "Should contain 'Abstract' in chunks (heading or content)");
}

#[tokio::test]
async fn test_import_lecture_docx() {
    let app = Router::new().route(
        "/api/import/lecture",
        post(malfestio_server::api::importer::post_import_lecture),
    );

    let boundary = "------------------------boundary123";
    let body_data = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"notes.docx\"\r\nContent-Type: application/vnd.openxmlformats-officedocument.wordprocessingml.document\r\n\r\n",
        boundary = boundary
    );
    let mut full_body = body_data.into_bytes();
    full_body.extend_from_slice(b"fake docx content");
    full_body.extend_from_slice(format!("\r\n--{boundary}--\r\n", boundary = boundary).as_bytes());

    let req = Request::builder()
        .method("POST")
        .uri("/api/import/lecture")
        .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
        .body(Body::from(full_body))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    assert!(
        body["content"]
            .as_str()
            .unwrap()
            .contains("DOCX parsing not yet implemented")
    );
}

#[tokio::test]
async fn test_import_lecture_no_file() {
    let app = Router::new().route(
        "/api/import/lecture",
        post(malfestio_server::api::importer::post_import_lecture),
    );

    let boundary = "------------------------boundary123";
    let full_body = format!("--{boundary}--\r\n", boundary = boundary);

    let req = Request::builder()
        .method("POST")
        .uri("/api/import/lecture")
        .header(CONTENT_TYPE, format!("multipart/form-data; boundary={}", boundary))
        .body(Body::from(full_body))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let status = response.status();

    if status != StatusCode::BAD_REQUEST {
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        println!(
            "Test No File Failed. Status: {}, Body: {}",
            status,
            String::from_utf8_lossy(&body_bytes)
        );
    }
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
