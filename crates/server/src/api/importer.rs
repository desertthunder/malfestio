use axum::{Json, http::StatusCode, response::IntoResponse};
use readability::extractor;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct ImportRequest {
    url: String,
}

pub async fn import_article(Json(payload): Json<ImportRequest>) -> impl IntoResponse {
    if payload.url.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "URL is required"}))).into_response();
    }

    let url = payload.url.clone();
    let url_for_task = url.clone();

    let result = tokio::task::spawn_blocking(move || extractor::scrape(&url_for_task)).await;

    match result {
        Ok(Ok(product)) => Json(json!({
            "title": product.title,
            "content": product.content,
            "text": product.text,
            "url": url
        }))
        .into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Readability error: {}", e)})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Task join error: {}", e)})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn test_import_article_wikipedia() {
        let payload = ImportRequest { url: "https://www.rust-lang.org".to_string() };
        let response = import_article(Json(payload)).await.into_response();
        let status = response.status();
        if status != StatusCode::OK {
            let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
            panic!("Test failed with status {}. Body: {}", status, body_str);
        }

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let title = body_json["title"].as_str().unwrap();
        assert!(title.contains("Rust"));
        assert!(body_json["text"].as_str().unwrap().len() > 100);
    }

    #[tokio::test]
    async fn test_import_article_empty_url() {
        let payload = ImportRequest { url: "   ".to_string() };
        let response = import_article(Json(payload)).await.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
