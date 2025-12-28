use axum::response::IntoResponse;
use axum::{
    extract::Request,
    http::{self, StatusCode},
    middleware::Next,
    response::Response,
};
use serde_json::json;

#[derive(Clone, Debug)]
pub struct UserContext {
    pub did: String,
    pub handle: String,
}

/// TODO: Cache or use signature verification for performance
pub async fn auth_middleware(mut req: Request, next: Next) -> Response {
    let auth_header = req.headers().get(http::header::AUTHORIZATION);

    let token = match auth_header.and_then(|h| h.to_str().ok()) {
        Some(header_val) if header_val.starts_with("Bearer ") => &header_val[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                axum::Json(json!({ "error": "Missing or invalid Authorization header" })),
            )
                .into_response();
        }
    };

    let client = reqwest::Client::new();
    let pds_url = std::env::var("PDS_URL").unwrap_or_else(|_| "https://bsky.social".to_string());

    let resp = client
        .get(format!("{}/xrpc/com.atproto.server.getSession", pds_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;

    match resp {
        Ok(response) if response.status().is_success() => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            let did = body["did"].as_str().unwrap_or("").to_string();
            let handle = body["handle"].as_str().unwrap_or("").to_string();

            req.extensions_mut().insert(UserContext { did, handle });
            next.run(req).await
        }
        _ => (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({ "error": "Invalid session" })),
        )
            .into_response(),
    }
}

/// Optional auth middleware - populates UserContext if valid token is present,
/// but continues without error if no token or invalid token.
///
/// Used by endpoints that need to check permissions but don't require authentication.
pub async fn optional_auth_middleware(mut req: Request, next: Next) -> Response {
    let auth_header = req.headers().get(http::header::AUTHORIZATION);

    let token = match auth_header.and_then(|h| h.to_str().ok()) {
        Some(header_val) if header_val.starts_with("Bearer ") => &header_val[7..],
        _ => {
            return next.run(req).await;
        }
    };

    let client = reqwest::Client::new();
    let pds_url = std::env::var("PDS_URL").unwrap_or_else(|_| "https://bsky.social".to_string());

    match client
        .get(format!("{}/xrpc/com.atproto.server.getSession", pds_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            let did = body["did"].as_str().unwrap_or("").to_string();
            let handle = body["handle"].as_str().unwrap_or("").to_string();

            req.extensions_mut().insert(UserContext { did, handle });
        }
        _ => {}
    }

    next.run(req).await
}
