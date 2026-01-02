//! PDS client for XRPC operations.
//!
//! Handles communication with a user's Personal Data Server.

use crate::oauth::dpop::DpopKeypair;
use malfestio_core::at_uri::AtUri;
use serde::{Deserialize, Serialize};

/// A client for interacting with a user's PDS.
///
/// Supports both DPoP-bound tokens (OAuth) and Bearer tokens (app passwords).
pub struct PdsClient {
    http_client: reqwest::Client,
    pds_url: String,
    access_token: String,
    dpop_keypair: Option<DpopKeypair>,
}

/// Request body for putRecord XRPC.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PutRecordRequest {
    pub repo: String,
    pub collection: String,
    pub rkey: String,
    pub record: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_record: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate: Option<bool>,
}

/// Response from putRecord XRPC.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PutRecordResponse {
    pub uri: String,
    pub cid: String,
}

/// Request body for deleteRecord XRPC.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRecordRequest {
    pub repo: String,
    pub collection: String,
    pub rkey: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_record: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_commit: Option<String>,
}

/// Response from uploadBlob XRPC.
#[derive(Deserialize)]
pub struct UploadBlobResponse {
    pub blob: BlobRef,
}

/// A reference to an uploaded blob.
#[derive(Clone, Serialize, Deserialize)]
pub struct BlobRef {
    #[serde(rename = "$type")]
    pub blob_type: String,
    #[serde(rename = "ref")]
    pub cid: CidLink,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub size: u64,
}

/// A CID link.
#[derive(Clone, Serialize, Deserialize)]
pub struct CidLink {
    #[serde(rename = "$link")]
    pub link: String,
}

/// Error type for PDS operations.
#[derive(Debug, Clone)]
pub enum PdsError {
    NetworkError(String),
    AuthError(String),
    ValidationError(String),
    NotFound(String),
    ServerError(String),
}

impl std::fmt::Display for PdsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdsError::NetworkError(e) => write!(f, "Network error: {}", e),
            PdsError::AuthError(e) => write!(f, "Authentication error: {}", e),
            PdsError::ValidationError(e) => write!(f, "Validation error: {}", e),
            PdsError::NotFound(e) => write!(f, "Not found: {}", e),
            PdsError::ServerError(e) => write!(f, "Server error: {}", e),
        }
    }
}

impl std::error::Error for PdsError {}

impl PdsClient {
    /// Create a new PDS client with DPoP support (OAuth tokens).
    ///
    /// Uses DPoP proof-of-possession for enhanced security.
    pub fn new_with_dpop(pds_url: String, access_token: String, dpop_keypair: DpopKeypair) -> Self {
        Self { http_client: reqwest::Client::new(), pds_url, access_token, dpop_keypair: Some(dpop_keypair) }
    }

    /// Create a new PDS client with Bearer authentication (app password tokens).
    ///
    /// Uses standard Bearer token authentication without DPoP.
    pub fn new_bearer(pds_url: String, access_token: String) -> Self {
        Self { http_client: reqwest::Client::new(), pds_url, access_token, dpop_keypair: None }
    }

    /// Create a new PDS client (deprecated - use new_with_dpop or new_bearer).
    #[deprecated(since = "0.1.0", note = "Use new_with_dpop or new_bearer instead")]
    pub fn new(pds_url: String, access_token: String, dpop_keypair: DpopKeypair) -> Self {
        Self::new_with_dpop(pds_url, access_token, dpop_keypair)
    }

    /// Create or update a record in the repository.
    ///
    /// # Arguments
    ///
    /// * `did` - The user's DID (repository owner)
    /// * `collection` - The collection NSID (e.g., "app.malfestio.deck")
    /// * `rkey` - The record key (TID)
    /// * `record` - The record data as JSON
    pub async fn put_record(
        &self, did: &str, collection: &str, rkey: &str, record: serde_json::Value,
    ) -> Result<AtUri, PdsError> {
        let url = format!("{}/xrpc/com.atproto.repo.putRecord", self.pds_url);

        let request = PutRecordRequest {
            repo: did.to_string(),
            collection: collection.to_string(),
            rkey: rkey.to_string(),
            record,
            swap_record: None,
            swap_commit: None,
            validate: Some(true),
        };

        let mut request_builder = self.http_client.post(&url);

        // Conditionally add DPoP or Bearer authentication
        if let Some(ref dpop_keypair) = self.dpop_keypair {
            // OAuth with DPoP
            let dpop_proof = dpop_keypair.generate_proof("POST", &url, Some(&self.access_token));
            request_builder = request_builder
                .header("Authorization", format!("DPoP {}", self.access_token))
                .header("DPoP", dpop_proof);
        } else {
            // App password with Bearer
            request_builder = request_builder.header("Authorization", format!("Bearer {}", self.access_token));
        }

        let response = request_builder
            .json(&request)
            .send()
            .await
            .map_err(|e| PdsError::NetworkError(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Delete a record from the repository.
    pub async fn delete_record(&self, did: &str, collection: &str, rkey: &str) -> Result<(), PdsError> {
        let url = format!("{}/xrpc/com.atproto.repo.deleteRecord", self.pds_url);

        let request = DeleteRecordRequest {
            repo: did.to_string(),
            collection: collection.to_string(),
            rkey: rkey.to_string(),
            swap_record: None,
            swap_commit: None,
        };

        let mut request_builder = self.http_client.post(&url);

        // Conditionally add DPoP or Bearer authentication
        if let Some(ref dpop_keypair) = self.dpop_keypair {
            // OAuth with DPoP
            let dpop_proof = dpop_keypair.generate_proof("POST", &url, Some(&self.access_token));
            request_builder = request_builder
                .header("Authorization", format!("DPoP {}", self.access_token))
                .header("DPoP", dpop_proof);
        } else {
            // App password with Bearer
            request_builder = request_builder.header("Authorization", format!("Bearer {}", self.access_token));
        }

        let response = request_builder
            .json(&request)
            .send()
            .await
            .map_err(|e| PdsError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(self.map_error_status(status, body))
        }
    }

    /// Upload a blob (media attachment) to the repository.
    pub async fn upload_blob(&self, data: Vec<u8>, mime_type: &str) -> Result<BlobRef, PdsError> {
        let url = format!("{}/xrpc/com.atproto.repo.uploadBlob", self.pds_url);

        let mut request_builder = self.http_client.post(&url);

        // Conditionally add DPoP or Bearer authentication
        if let Some(ref dpop_keypair) = self.dpop_keypair {
            // OAuth with DPoP
            let dpop_proof = dpop_keypair.generate_proof("POST", &url, Some(&self.access_token));
            request_builder = request_builder
                .header("Authorization", format!("DPoP {}", self.access_token))
                .header("DPoP", dpop_proof);
        } else {
            // App password with Bearer
            request_builder = request_builder.header("Authorization", format!("Bearer {}", self.access_token));
        }

        let response = request_builder
            .header("Content-Type", mime_type)
            .body(data)
            .send()
            .await
            .map_err(|e| PdsError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(self.map_error_status(status, body));
        }

        let upload_response: UploadBlobResponse = response
            .json()
            .await
            .map_err(|e| PdsError::NetworkError(e.to_string()))?;

        Ok(upload_response.blob)
    }

    /// Handle response and parse AT-URI from success.
    async fn handle_response(&self, response: reqwest::Response) -> Result<AtUri, PdsError> {
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(self.map_error_status(status, body));
        }

        let put_response: PutRecordResponse = response
            .json()
            .await
            .map_err(|e| PdsError::NetworkError(e.to_string()))?;

        AtUri::parse(&put_response.uri).map_err(|e| PdsError::ValidationError(e.to_string()))
    }

    /// Map HTTP status to PdsError.
    fn map_error_status(&self, status: reqwest::StatusCode, body: String) -> PdsError {
        match status.as_u16() {
            401 => PdsError::AuthError(body),
            400 => PdsError::ValidationError(body),
            404 => PdsError::NotFound(body),
            _ => PdsError::ServerError(format!("{}: {}", status, body)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_record_request_serialization() {
        let request = PutRecordRequest {
            repo: "did:plc:abc123".to_string(),
            collection: "app.malfestio.deck".to_string(),
            rkey: "3k5abc123".to_string(),
            record: serde_json::json!({
                "title": "Test Deck",
                "createdAt": "2024-01-01T00:00:00Z"
            }),
            swap_record: None,
            swap_commit: None,
            validate: Some(true),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"repo\":\"did:plc:abc123\""));
        assert!(json.contains("\"collection\":\"app.malfestio.deck\""));
        assert!(json.contains("\"rkey\":\"3k5abc123\""));
        assert!(json.contains("\"validate\":true"));
    }

    #[test]
    fn test_delete_record_request_serialization() {
        let request = DeleteRecordRequest {
            repo: "did:plc:abc123".to_string(),
            collection: "app.malfestio.deck".to_string(),
            rkey: "3k5abc123".to_string(),
            swap_record: None,
            swap_commit: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"repo\":\"did:plc:abc123\""));
        assert!(!json.contains("swapRecord")); // Should be omitted when None
    }

    #[test]
    fn test_blob_ref_serialization() {
        let blob_ref = BlobRef {
            blob_type: "blob".to_string(),
            cid: CidLink { link: "bafyreiabc123".to_string() },
            mime_type: "image/jpeg".to_string(),
            size: 12345,
        };

        let json = serde_json::to_string(&blob_ref).unwrap();
        assert!(json.contains("\"$type\":\"blob\""));
        assert!(json.contains("\"$link\":\"bafyreiabc123\""));
        assert!(json.contains("\"mimeType\":\"image/jpeg\""));
    }

    #[test]
    fn test_pds_error_display() {
        let err = PdsError::AuthError("Invalid token".to_string());
        assert!(err.to_string().contains("Invalid token"));

        let err = PdsError::NetworkError("Connection refused".to_string());
        assert!(err.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_pds_client_new_with_dpop() {
        use crate::oauth::dpop::DpopKeypair;

        let keypair = DpopKeypair::generate();
        let client = PdsClient::new_with_dpop("https://bsky.social".to_string(), "test_token".to_string(), keypair);

        assert_eq!(client.pds_url, "https://bsky.social");
        assert_eq!(client.access_token, "test_token");
        assert!(client.dpop_keypair.is_some());
    }

    #[test]
    fn test_pds_client_new_bearer() {
        let client = PdsClient::new_bearer("https://bsky.social".to_string(), "test_token".to_string());

        assert_eq!(client.pds_url, "https://bsky.social");
        assert_eq!(client.access_token, "test_token");
        assert!(client.dpop_keypair.is_none());
    }

    #[test]
    #[allow(deprecated)]
    fn test_pds_client_new_deprecated() {
        use crate::oauth::dpop::DpopKeypair;

        let keypair = DpopKeypair::generate();
        let client = PdsClient::new("https://bsky.social".to_string(), "test_token".to_string(), keypair);

        assert_eq!(client.pds_url, "https://bsky.social");
        assert_eq!(client.access_token, "test_token");
        assert!(client.dpop_keypair.is_some());
    }
}
