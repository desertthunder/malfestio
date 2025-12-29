//! OAuth 2.1 authorization flow for AT Protocol.
//!
//! Handles the complete OAuth flow including:
//! - Authorization URL generation
//! - Token exchange with PKCE + DPoP
//! - Token refresh

use super::dpop::DpopKeypair;
use super::pkce::{derive_code_challenge, generate_code_verifier};
use super::resolver::{IdentityResolver, ResolveError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// OAuth session state stored during the authorization flow.
#[derive(Clone)]
pub struct OAuthSession {
    /// The PKCE code verifier
    pub code_verifier: String,
    /// The DPoP keypair for this session
    pub dpop_keypair: DpopKeypair,
    /// The user's DID after resolution
    pub did: Option<String>,
    /// The user's PDS URL
    pub pds_url: Option<String>,
    /// When this session was created (for expiry)
    pub created_at: std::time::Instant,
}

/// OAuth tokens received from the authorization server.
#[derive(Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
}

/// In-memory session storage (for development).
/// In production, use a database-backed implementation.
pub type SessionStore = Arc<RwLock<HashMap<String, OAuthSession>>>;

/// Create a new session store.
pub fn new_session_store() -> SessionStore {
    Arc::new(RwLock::new(HashMap::new()))
}

/// OAuth flow manager.
pub struct OAuthFlow {
    resolver: IdentityResolver,
    client: reqwest::Client,
    client_id: String,
    redirect_uri: String,
}

impl OAuthFlow {
    /// Create a new OAuth flow manager.
    pub fn new() -> Self {
        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        Self {
            resolver: IdentityResolver::new(),
            client: reqwest::Client::new(),
            client_id: format!("{}/oauth/client-metadata.json", app_url),
            redirect_uri: format!("{}/oauth/callback", app_url),
        }
    }

    /// Start the OAuth flow for a user handle or DID.
    ///
    /// Returns the authorization URL to redirect the user to.
    pub async fn start_authorization(
        &self, handle_or_did: &str, state: &str, sessions: &SessionStore,
    ) -> Result<String, OAuthFlowError> {
        let (did, pds_url) = if handle_or_did.starts_with("did:") {
            let resolved = self.resolver.resolve_did(handle_or_did).await?;
            (resolved.did, resolved.pds_url)
        } else {
            let did = self.resolver.resolve_handle(handle_or_did).await?;
            let resolved = self.resolver.resolve_did(&did).await?;
            (resolved.did, resolved.pds_url)
        };

        let auth_server = self.get_auth_server_metadata(&pds_url).await?;

        let code_verifier = generate_code_verifier();
        let code_challenge = derive_code_challenge(&code_verifier);

        let dpop_keypair = DpopKeypair::generate();

        let session = OAuthSession {
            code_verifier,
            dpop_keypair,
            did: Some(did.clone()),
            pds_url: Some(pds_url),
            created_at: std::time::Instant::now(),
        };

        sessions.write().unwrap().insert(state.to_string(), session);

        let auth_url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256&login_hint={}",
            auth_server.authorization_endpoint,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode("atproto transition:generic"),
            urlencoding::encode(state),
            urlencoding::encode(&code_challenge),
            urlencoding::encode(&did)
        );

        Ok(auth_url)
    }

    /// Exchange an authorization code for tokens.
    pub async fn exchange_code(
        &self, code: &str, state: &str, sessions: &SessionStore,
    ) -> Result<OAuthTokens, OAuthFlowError> {
        let session = sessions
            .read()
            .unwrap()
            .get(state)
            .cloned()
            .ok_or(OAuthFlowError::SessionNotFound)?;

        let pds_url = session.pds_url.as_ref().ok_or(OAuthFlowError::SessionNotFound)?;

        let auth_server = self.get_auth_server_metadata(pds_url).await?;

        let dpop_proof = session
            .dpop_keypair
            .generate_proof("POST", &auth_server.token_endpoint, None);

        let response = self
            .client
            .post(&auth_server.token_endpoint)
            .header("DPoP", dpop_proof)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.redirect_uri),
                ("client_id", &self.client_id),
                ("code_verifier", &session.code_verifier),
            ])
            .send()
            .await
            .map_err(|e| OAuthFlowError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(OAuthFlowError::TokenExchangeFailed(error_body));
        }

        let tokens: OAuthTokens = response
            .json()
            .await
            .map_err(|e| OAuthFlowError::NetworkError(e.to_string()))?;

        sessions.write().unwrap().remove(state);

        Ok(tokens)
    }

    /// Refresh an access token.
    pub async fn refresh_token(
        &self, refresh_token: &str, pds_url: &str, dpop_keypair: &DpopKeypair,
    ) -> Result<OAuthTokens, OAuthFlowError> {
        let auth_server = self.get_auth_server_metadata(pds_url).await?;

        let dpop_proof = dpop_keypair.generate_proof("POST", &auth_server.token_endpoint, None);

        let response = self
            .client
            .post(&auth_server.token_endpoint)
            .header("DPoP", dpop_proof)
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("client_id", &self.client_id),
            ])
            .send()
            .await
            .map_err(|e| OAuthFlowError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(OAuthFlowError::TokenRefreshFailed(error_body));
        }

        response
            .json()
            .await
            .map_err(|e| OAuthFlowError::NetworkError(e.to_string()))
    }

    /// Get authorization server metadata from PDS.
    async fn get_auth_server_metadata(&self, pds_url: &str) -> Result<AuthServerMetadata, OAuthFlowError> {
        // First get the protected resource metadata
        let resource_url = format!("{}/.well-known/oauth-protected-resource", pds_url);

        let resource_response = self
            .client
            .get(&resource_url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| OAuthFlowError::NetworkError(e.to_string()))?;

        if !resource_response.status().is_success() {
            return Err(OAuthFlowError::MetadataFetchFailed(pds_url.to_string()));
        }

        let resource: serde_json::Value = resource_response
            .json()
            .await
            .map_err(|e| OAuthFlowError::NetworkError(e.to_string()))?;

        let auth_server_url = resource["authorization_servers"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .ok_or_else(|| OAuthFlowError::MetadataFetchFailed(pds_url.to_string()))?;

        let auth_meta_url = format!("{}/.well-known/oauth-authorization-server", auth_server_url);

        let auth_response = self
            .client
            .get(&auth_meta_url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| OAuthFlowError::NetworkError(e.to_string()))?;

        if !auth_response.status().is_success() {
            return Err(OAuthFlowError::MetadataFetchFailed(auth_server_url.to_string()));
        }

        auth_response
            .json()
            .await
            .map_err(|e| OAuthFlowError::NetworkError(e.to_string()))
    }
}

impl Default for OAuthFlow {
    fn default() -> Self {
        Self::new()
    }
}

/// Authorization server metadata.
#[derive(Deserialize)]
pub struct AuthServerMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub pushed_authorization_request_endpoint: Option<String>,
}

/// Error type for OAuth flow operations.
#[derive(Debug, Clone)]
pub enum OAuthFlowError {
    SessionNotFound,
    NetworkError(String),
    MetadataFetchFailed(String),
    TokenExchangeFailed(String),
    TokenRefreshFailed(String),
    ResolveError(String),
}

impl From<ResolveError> for OAuthFlowError {
    fn from(err: ResolveError) -> Self {
        OAuthFlowError::ResolveError(err.to_string())
    }
}

impl std::fmt::Display for OAuthFlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthFlowError::SessionNotFound => write!(f, "OAuth session not found"),
            OAuthFlowError::NetworkError(e) => write!(f, "Network error: {}", e),
            OAuthFlowError::MetadataFetchFailed(url) => write!(f, "Failed to fetch metadata from {}", url),
            OAuthFlowError::TokenExchangeFailed(e) => write!(f, "Token exchange failed: {}", e),
            OAuthFlowError::TokenRefreshFailed(e) => write!(f, "Token refresh failed: {}", e),
            OAuthFlowError::ResolveError(e) => write!(f, "Identity resolution failed: {}", e),
        }
    }
}

impl std::error::Error for OAuthFlowError {}

/// Generate a secure random state parameter.
pub fn generate_state() -> String {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("Failed to generate random bytes");
    URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_state() {
        let state1 = generate_state();
        let state2 = generate_state();

        assert_ne!(state1, state2);
        assert_eq!(state1.len(), 22);
    }

    #[test]
    fn test_new_session_store() {
        let store = new_session_store();
        assert!(store.read().unwrap().is_empty());
    }

    #[test]
    fn test_oauth_flow_creation() {
        let flow = OAuthFlow::new();
        assert!(flow.client_id.contains("client-metadata.json"));
        assert!(flow.redirect_uri.contains("callback"));
    }

    #[test]
    fn test_oauth_flow_error_display() {
        let err = OAuthFlowError::SessionNotFound;
        assert!(err.to_string().contains("session not found"));

        let err = OAuthFlowError::NetworkError("timeout".to_string());
        assert!(err.to_string().contains("timeout"));
    }
}
