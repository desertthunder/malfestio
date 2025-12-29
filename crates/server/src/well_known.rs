//! Well-known endpoints for AT Protocol.
//!
//! Provides:
//! - `/.well-known/atproto-did` - Returns the server's DID for domain verification

use axum::response::IntoResponse;

/// Handler for `/.well-known/atproto-did`.
///
/// Returns the server's DID from the `ATPROTO_SERVER_DID` environment variable.
/// Used for domain verification in AT Protocol.
pub async fn atproto_did_handler() -> impl IntoResponse {
    std::env::var("ATPROTO_SERVER_DID").unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_atproto_did_handler_empty_when_not_set() {
        let original = std::env::var("ATPROTO_SERVER_DID").ok();
        unsafe {
            std::env::remove_var("ATPROTO_SERVER_DID");
        }

        let result = atproto_did_handler().await.into_response();
        assert_eq!(result.status(), axum::http::StatusCode::OK);

        if let Some(val) = original {
            unsafe {
                std::env::set_var("ATPROTO_SERVER_DID", val);
            }
        }
    }
}
