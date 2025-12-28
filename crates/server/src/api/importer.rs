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
