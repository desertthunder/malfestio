//! OAuth API endpoints for AT Protocol authentication.
//!
//! Provides endpoints for:
//! - Starting the OAuth authorization flow
//! - Handling OAuth callbacks
//! - Refreshing tokens

use crate::db::DbPool;
use crate::oauth::flow::{OAuthFlow, SessionStore, generate_state, new_session_store};
use crate::repository::oauth::{DbOAuthRepository, OAuthRepository, StoreTokensRequest};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

/// Shared OAuth state with database repository.
pub struct OAuthState {
    pub flow: OAuthFlow,
    pub sessions: SessionStore,
    pub repo: Arc<dyn OAuthRepository>,
}

impl OAuthState {
    /// Create OAuth state with database connection.
    pub fn with_pool(pool: DbPool) -> Self {
        Self { flow: OAuthFlow::new(), sessions: new_session_store(), repo: Arc::new(DbOAuthRepository::new(pool)) }
    }

    /// Create OAuth state without database (for testing).
    pub fn new() -> Self {
        Self { flow: OAuthFlow::new(), sessions: new_session_store(), repo: Arc::new(MockOAuthRepository) }
    }
}

impl Default for OAuthState {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock repository for testing.
struct MockOAuthRepository;

#[async_trait::async_trait]
impl OAuthRepository for MockOAuthRepository {
    async fn store_tokens(&self, _req: StoreTokensRequest<'_>) -> Result<(), crate::repository::oauth::OAuthRepoError> {
        Ok(())
    }

    async fn get_tokens(
        &self, did: &str,
    ) -> Result<crate::repository::oauth::StoredToken, crate::repository::oauth::OAuthRepoError> {
        Err(crate::repository::oauth::OAuthRepoError::NotFound(did.to_string()))
    }

    async fn update_tokens(
        &self, _did: &str, _access_token: &str, _refresh_token: Option<&str>,
        _expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<(), crate::repository::oauth::OAuthRepoError> {
        Ok(())
    }

    async fn delete_tokens(&self, _did: &str) -> Result<(), crate::repository::oauth::OAuthRepoError> {
        Ok(())
    }
}

/// Request to start OAuth authorization.
#[derive(Deserialize)]
pub struct AuthorizeRequest {
    /// Handle or DID to authenticate
    pub handle: String,
}

/// Response from starting authorization.
#[derive(Serialize)]
pub struct AuthorizeResponse {
    /// URL to redirect the user to
    pub authorization_url: String,
    /// State parameter (for CSRF protection)
    pub state: String,
}

/// Query parameters from OAuth callback.
#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
}

/// Start the OAuth authorization flow.
///
/// POST /api/oauth/authorize
/// Body: { "handle": "alice.bsky.social" }
pub async fn authorize(
    State(oauth): State<Arc<OAuthState>>, Json(payload): Json<AuthorizeRequest>,
) -> impl IntoResponse {
    let state = generate_state();

    match oauth
        .flow
        .start_authorization(&payload.handle, &state, &oauth.sessions)
        .await
    {
        Ok(auth_url) => (
            StatusCode::OK,
            Json(AuthorizeResponse { authorization_url: auth_url, state }),
        )
            .into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

/// Handle OAuth callback from authorization server.
///
/// GET /api/oauth/callback?code=...&state=...
pub async fn callback(State(oauth): State<Arc<OAuthState>>, Query(params): Query<CallbackQuery>) -> impl IntoResponse {
    if let Some(error) = params.error {
        let description = params.error_description.unwrap_or_default();
        return Redirect::to(&format!(
            "/login?error={}&description={}",
            urlencoding::encode(&error),
            urlencoding::encode(&description)
        ))
        .into_response();
    }

    let session = {
        let sessions = oauth.sessions.read().unwrap();
        sessions.get(&params.state).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => {
            return Redirect::to("/login?error=session_not_found").into_response();
        }
    };

    match oauth
        .flow
        .exchange_code(&params.code, &params.state, &oauth.sessions)
        .await
    {
        Ok(tokens) => {
            let did = session.did.unwrap_or_default();
            let pds_url = session.pds_url.unwrap_or_default();
            let expires_at = tokens
                .expires_in
                .map(|secs| Utc::now() + Duration::seconds(secs as i64));

            if let Err(e) = oauth
                .repo
                .store_tokens(StoreTokensRequest {
                    did: &did,
                    pds_url: &pds_url,
                    access_token: &tokens.access_token,
                    refresh_token: tokens.refresh_token.as_deref(),
                    token_type: &tokens.token_type,
                    expires_at,
                    dpop_keypair: &session.dpop_keypair,
                })
                .await
            {
                tracing::error!("Failed to store tokens: {}", e);
                return Redirect::to(&format!("/login?error={}", urlencoding::encode("token_storage_failed")))
                    .into_response();
            }

            Redirect::to(&format!("/login/success?did={}", urlencoding::encode(&did))).into_response()
        }
        Err(e) => Redirect::to(&format!("/login?error={}", urlencoding::encode(&e.to_string()))).into_response(),
    }
}

/// Request to refresh tokens.
#[derive(Deserialize)]
pub struct RefreshRequest {
    pub did: String,
}

/// Response from token refresh.
#[derive(Serialize)]
pub struct RefreshResponse {
    pub success: bool,
    pub expires_at: Option<String>,
}

/// Refresh an access token.
///
/// POST /api/oauth/refresh
/// Body: { "did": "did:plc:..." }
pub async fn refresh(State(oauth): State<Arc<OAuthState>>, Json(payload): Json<RefreshRequest>) -> impl IntoResponse {
    // Get stored tokens from database
    let stored = match oauth.repo.get_tokens(&payload.did).await {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::NOT_FOUND, Json(json!({ "error": e.to_string() }))).into_response();
        }
    };

    // Reconstruct DPoP keypair
    let dpop_keypair = match stored.dpop_keypair() {
        Some(kp) => kp,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Invalid stored keypair" })),
            )
                .into_response();
        }
    };

    // Get refresh token
    let refresh_token = match &stored.refresh_token {
        Some(rt) => rt.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "No refresh token available" })),
            )
                .into_response();
        }
    };

    // Refresh tokens via OAuth flow
    match oauth
        .flow
        .refresh_token(&refresh_token, &stored.pds_url, &dpop_keypair)
        .await
    {
        Ok(new_tokens) => {
            let expires_at = new_tokens
                .expires_in
                .map(|secs| Utc::now() + Duration::seconds(secs as i64));

            if let Err(e) = oauth
                .repo
                .update_tokens(
                    &payload.did,
                    &new_tokens.access_token,
                    new_tokens.refresh_token.as_deref(),
                    expires_at,
                )
                .await
            {
                tracing::error!("Failed to update tokens: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "Failed to update tokens" })),
                )
                    .into_response();
            }

            (
                StatusCode::OK,
                Json(RefreshResponse { success: true, expires_at: expires_at.map(|dt| dt.to_rfc3339()) }),
            )
                .into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e.to_string() }))).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_state_creation() {
        let state = OAuthState::new();
        assert!(state.sessions.read().unwrap().is_empty());
    }

    #[test]
    fn test_authorize_request_deserialization() {
        let json = r#"{"handle": "alice.bsky.social"}"#;
        let request: AuthorizeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.handle, "alice.bsky.social");
    }

    #[test]
    fn test_authorize_response_serialization() {
        let response = AuthorizeResponse {
            authorization_url: "https://example.com/oauth".to_string(),
            state: "abc123".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("authorization_url"));
        assert!(json.contains("state"));
    }

    #[test]
    fn test_callback_query_deserialization() {
        let query = "code=abc123&state=xyz789";
        let parsed: CallbackQuery = serde_qs::from_str(query).unwrap();
        assert_eq!(parsed.code, "abc123");
        assert_eq!(parsed.state, "xyz789");
        assert!(parsed.error.is_none());
    }

    #[test]
    fn test_callback_query_with_error() {
        let query = "code=&state=xyz789&error=access_denied&error_description=User+denied";
        let parsed: CallbackQuery = serde_qs::from_str(query).unwrap();
        assert_eq!(parsed.error, Some("access_denied".to_string()));
    }

    #[test]
    fn test_refresh_request_deserialization() {
        let json = r#"{"did": "did:plc:abc123"}"#;
        let request: RefreshRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.did, "did:plc:abc123");
    }
}
