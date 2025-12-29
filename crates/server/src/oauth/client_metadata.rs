//! OAuth client metadata endpoint.
//!
//! Serves the client_metadata.json for AT Protocol OAuth discovery.

use axum::{Json, response::IntoResponse};
use serde::Serialize;

/// OAuth client metadata for AT Protocol.
#[derive(Serialize, Clone)]
pub struct ClientMetadata {
    pub client_id: String,
    pub application_type: String,
    pub grant_types: Vec<String>,
    pub scope: String,
    pub response_types: Vec<String>,
    pub redirect_uris: Vec<String>,
    pub client_name: String,
    pub client_uri: String,
    pub token_endpoint_auth_method: String,
    pub dpop_bound_access_tokens: bool,
}

impl Default for ClientMetadata {
    fn default() -> Self {
        Self::from_env()
    }
}

impl ClientMetadata {
    /// Create client metadata from environment variables.
    pub fn from_env() -> Self {
        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
        let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Malfestio".to_string());

        Self {
            client_id: format!("{}/oauth/client-metadata.json", app_url),
            application_type: "web".to_string(),
            grant_types: vec!["authorization_code".to_string(), "refresh_token".to_string()],
            scope: "atproto transition:generic".to_string(),
            response_types: vec!["code".to_string()],
            redirect_uris: vec![format!("{}/oauth/callback", app_url)],
            client_name: app_name,
            client_uri: app_url,
            token_endpoint_auth_method: "none".to_string(),
            dpop_bound_access_tokens: true,
        }
    }
}

/// Handler for `/.well-known/oauth-client-metadata` endpoint.
pub async fn client_metadata_handler() -> impl IntoResponse {
    Json(ClientMetadata::from_env())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_metadata() {
        let meta = ClientMetadata::default();
        assert!(meta.client_id.contains("client-metadata.json"));
        assert_eq!(meta.application_type, "web");
        assert!(meta.grant_types.contains(&"authorization_code".to_string()));
        assert!(meta.dpop_bound_access_tokens);
    }

    #[test]
    fn test_metadata_serialization() {
        let meta = ClientMetadata::default();
        let json = serde_json::to_string(&meta).unwrap();

        assert!(json.contains("client_id"));
        assert!(json.contains("dpop_bound_access_tokens"));
        assert!(json.contains("atproto"));
    }
}
