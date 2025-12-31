use crate::oauth::dpop::{DpopVerifyRequest, generate_nonce, verify_proof};
use crate::state::SharedState;

use axum::{
    extract::{Request, State},
    http::{self, HeaderValue},
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

/// DPoP nonce expiry time (5 minutes)
const NONCE_TTL: Duration = Duration::from_secs(300);

/// Parsed authorization header.
enum AuthScheme {
    Bearer(String),
    DPoP(String),
}

/// Parse the Authorization header to extract scheme and token.
fn parse_auth_header(header_val: &str) -> Option<AuthScheme> {
    if let Some(token) = header_val.strip_prefix("Bearer ") {
        Some(AuthScheme::Bearer(token.to_string()))
    } else {
        header_val
            .strip_prefix("DPoP ")
            .map(|token| AuthScheme::DPoP(token.to_string()))
    }
}

/// Delegated Authentication Strategy with DPoP Support:
///
/// We verify the token by calling the PDS `getSession` endpoint.
/// For DPoP-bound tokens, we also verify the DPoP proof JWT.
/// To improve performance, we cache the session result for a short duration (TTL).
///
/// NOTE: This assumes the PDS is trusted.
pub async fn auth_middleware(State(state): State<SharedState>, mut req: Request, next: Next) -> Response {
    let auth_header = req.headers().get(http::header::AUTHORIZATION);
    let dpop_header = req.headers().get("DPoP");

    let (token, is_dpop) = match auth_header.and_then(|h| h.to_str().ok()).and_then(parse_auth_header) {
        Some(AuthScheme::Bearer(t)) => (t, false),
        Some(AuthScheme::DPoP(t)) => (t, true),
        None => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(json!({ "error": "Missing or invalid Authorization header" })),
            )
                .into_response();
        }
    };

    if is_dpop {
        let dpop_proof = match dpop_header.and_then(|h| h.to_str().ok()) {
            Some(p) => p,
            None => {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    axum::Json(json!({ "error": "Missing DPoP proof header" })),
                )
                    .into_response();
            }
        };

        let method = req.method().as_str();
        let uri = req.uri().to_string();

        let expected_nonce = {
            let nonces = state.dpop_nonces.read().await;
            nonces
                .get(&token)
                .filter(|created_at| created_at.elapsed() < NONCE_TTL)
                .map(|_| token.clone())
        };

        let verify_result = verify_proof(DpopVerifyRequest::new(dpop_proof, method, &uri, Some(&token), None));

        if let Err(e) = verify_result {
            tracing::warn!("DPoP verification failed: {}", e);
            let nonce = generate_nonce();
            {
                let mut nonces = state.dpop_nonces.write().await;
                nonces.insert(token.clone(), Instant::now());
            }
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                [(
                    http::header::HeaderName::from_static("dpop-nonce"),
                    HeaderValue::from_str(&nonce).unwrap(),
                )],
                axum::Json(json!({ "error": format!("DPoP verification failed: {}", e) })),
            )
                .into_response();
        }

        if expected_nonce.is_none() {
            let mut nonces = state.dpop_nonces.write().await;
            nonces.insert(token.clone(), Instant::now());
        }
    }

    {
        let cache = state.auth_cache.read().await;
        if let Some((user_ctx, timestamp)) = cache.get(&token)
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

    let token = match auth_header.and_then(|h| h.to_str().ok()).and_then(parse_auth_header) {
        Some(AuthScheme::Bearer(t)) | Some(AuthScheme::DPoP(t)) => t,
        None => {
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

/// Cleanup expired nonces from the cache.
/// This should be called periodically (e.g., via a background task).
#[allow(dead_code)]
pub async fn cleanup_expired_nonces(state: &SharedState) {
    let mut nonces = state.dpop_nonces.write().await;
    nonces.retain(|_, created_at| created_at.elapsed() < NONCE_TTL);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_auth_header_bearer() {
        let result = parse_auth_header("Bearer abc123");
        assert!(matches!(result, Some(AuthScheme::Bearer(t)) if t == "abc123"));
    }

    #[test]
    fn test_parse_auth_header_dpop() {
        let result = parse_auth_header("DPoP xyz789");
        assert!(matches!(result, Some(AuthScheme::DPoP(t)) if t == "xyz789"));
    }

    #[test]
    fn test_parse_auth_header_invalid() {
        assert!(parse_auth_header("Basic abc").is_none());
        assert!(parse_auth_header("InvalidScheme token").is_none());
    }
}
