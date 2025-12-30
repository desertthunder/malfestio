use crate::state::SharedState;

use axum::{
    extract::{Request, State},
    http::{self},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct UserContext {
    pub did: String,
    pub handle: String,
}

/// Cache expiry time (5 minutes)
const CACHE_TTL: Duration = Duration::from_secs(300);

/// Delegated Authentication Strategy:
///
/// We verify the token by calling the PDS `getSession` endpoint.
/// To improve performance, we cache the result for a short duration (TTL).
/// This avoids validating the JWT signature locally, which simplifies key management
/// (no need to fetch/rotate PDS public keys) while maintaining security via the PDS.
///
/// NOTE: This assumes the PDS is trusted.
pub async fn auth_middleware(State(state): State<SharedState>, mut req: Request, next: Next) -> Response {
    let auth_header = req.headers().get(http::header::AUTHORIZATION);

    let token = match auth_header.and_then(|h| h.to_str().ok()) {
        Some(header_val) if header_val.starts_with("Bearer ") => &header_val[7..],
        _ => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(json!({ "error": "Missing or invalid Authorization header" })),
            )
                .into_response();
        }
    };

    {
        let cache = state.auth_cache.read().await;
        if let Some((user_ctx, timestamp)) = cache.get(token)
            && timestamp.elapsed() < CACHE_TTL
        {
            req.extensions_mut().insert(user_ctx.clone());
            return next.run(req).await;
        }
    }

    let client = reqwest::Client::new();
    let pds_url = &state.config.pds_url;

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
            let user_ctx = UserContext { did, handle };

            {
                let mut cache = state.auth_cache.write().await;
                cache.insert(token.to_string(), (user_ctx.clone(), Instant::now()));
            }

            req.extensions_mut().insert(user_ctx);
            next.run(req).await
        }
        _ => (
            axum::http::StatusCode::UNAUTHORIZED,
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
