use crate::middleware::auth::UserContext;
use crate::state::SharedState;
use axum::{Json, extract::Extension, http::StatusCode, response::IntoResponse};
use dom_smoothie::Readability;
use malfestio_core::model::Visibility;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
pub struct ImportRequest {
    url: String,
}

#[derive(Serialize)]
pub struct ArticleMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    publish_date: Option<String>,
    source_url: String,
}

#[derive(Serialize)]
pub struct ImportArticleResponse {
    title: String,
    markdown: String,
    metadata: ArticleMetadata,
}

pub async fn import_article(Json(payload): Json<ImportRequest>) -> impl IntoResponse {
    if payload.url.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "URL is required"}))).into_response();
    }

    let url = payload.url.clone();

    // Fetch HTML content
    let html_result = reqwest::get(&url).await;
    let html_content = match html_result {
        Ok(response) => match response.text().await {
            Ok(text) => text,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Failed to fetch content: {}", e)})),
                )
                    .into_response();
            }
        },
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to fetch URL: {}", e)})),
            )
                .into_response();
        }
    };

    // Extract article using dom_smoothie
    let url_for_task = url.clone();
    let result = tokio::task::spawn_blocking(
        move || -> Result<(String, String, Option<String>, Option<String>), String> {
            let mut readability = Readability::new(html_content, Some(&url_for_task), None)
                .map_err(|e| format!("Readability error: {}", e))?;
            let article = readability.parse().map_err(|e| format!("Parse error: {}", e))?;
            Ok((
                article.title,
                article.content.to_string(),
                article.byline,
                article.published_time,
            ))
        },
    )
    .await;

    match result {
        Ok(Ok((title, content, author, publish_date))) => {
            // Convert HTML content to markdown
            let markdown = html2md::parse_html(&content);

            let response = ImportArticleResponse {
                title,
                markdown,
                metadata: ArticleMetadata { author, publish_date, source_url: payload.url },
            };

            Json(response).into_response()
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to extract article: {}", e)})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Task join error: {}", e)})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
pub struct ImportArticleSaveRequest {
    url: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default = "default_visibility")]
    visibility: Visibility,
}

fn default_visibility() -> Visibility {
    Visibility::Private
}

pub async fn import_article_save(
    Extension(user_ctx): Extension<UserContext>, axum::extract::State(state): axum::extract::State<SharedState>,
    Json(payload): Json<ImportArticleSaveRequest>,
) -> impl IntoResponse {
    if payload.url.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "URL is required"}))).into_response();
    }

    let url = payload.url.clone();

    // Fetch HTML content
    let html_result = reqwest::get(&url).await;
    let html_content = match html_result {
        Ok(response) => match response.text().await {
            Ok(text) => text,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": format!("Failed to fetch content: {}", e)})),
                )
                    .into_response();
            }
        },
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to fetch URL: {}", e)})),
            )
                .into_response();
        }
    };

    // Extract article using dom_smoothie
    let url_for_task = url.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<(String, String), String> {
        let mut readability = Readability::new(html_content, Some(&url_for_task), None)
            .map_err(|e| format!("Readability error: {}", e))?;
        let article = readability.parse().map_err(|e| format!("Parse error: {}", e))?;
        Ok((article.title, article.content.to_string()))
    })
    .await;

    match result {
        Ok(Ok((title, content))) => {
            // Convert HTML content to markdown
            let markdown = html2md::parse_html(&content);

            // Merge auto-tags with user-provided tags
            let mut tags = payload.tags.clone();
            if !tags.contains(&"imported".to_string()) {
                tags.push("imported".to_string());
            }
            if !tags.contains(&"article".to_string()) {
                tags.push("article".to_string());
            }

            // Store source URL as first link
            let links = vec![payload.url.clone()];

            // Create note
            match state
                .note_repo
                .create(&user_ctx.did, &title, &markdown, tags, payload.visibility, links)
                .await
            {
                Ok(note) => (StatusCode::CREATED, Json(note)).into_response(),
                Err(e) => {
                    tracing::error!("Failed to create note from import: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Failed to save article"})),
                    )
                        .into_response()
                }
            }
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to extract article: {}", e)})),
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
        // Verify markdown field exists and is non-empty
        let markdown = body_json["markdown"].as_str().unwrap();
        assert!(markdown.len() > 100);
        // Verify no HTML tags leak through
        assert!(!markdown.contains("<div"));
        assert!(!markdown.contains("<p>"));
        // Verify metadata structure exists
        assert!(body_json["metadata"].is_object());
        assert_eq!(
            body_json["metadata"]["source_url"].as_str().unwrap(),
            "https://www.rust-lang.org"
        );
    }

    #[tokio::test]
    async fn test_import_article_empty_url() {
        let payload = ImportRequest { url: "   ".to_string() };
        let response = import_article(Json(payload)).await.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
