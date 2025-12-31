//! Handle and DID resolution for AT Protocol.
//!
//! Resolves user identities to discover their PDS (Personal Data Server).

use serde::{Deserialize, Serialize};

/// Result of resolving a handle or DID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedIdentity {
    /// The DID (always populated after resolution)
    pub did: String,
    /// The handle (if resolved from DID)
    pub handle: Option<String>,
    /// The PDS URL for this identity
    pub pds_url: String,
}

/// Error type for resolution failures.
#[derive(Debug, Clone)]
pub enum ResolveError {
    /// Handle not found
    HandleNotFound(String),
    /// DID not found
    DidNotFound(String),
    /// Network error
    NetworkError(String),
    /// Invalid DID format
    InvalidDid(String),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::HandleNotFound(h) => write!(f, "Handle not found: {}", h),
            ResolveError::DidNotFound(d) => write!(f, "DID not found: {}", d),
            ResolveError::NetworkError(e) => write!(f, "Network error: {}", e),
            ResolveError::InvalidDid(d) => write!(f, "Invalid DID: {}", d),
        }
    }
}

impl std::error::Error for ResolveError {}

#[allow(deprecated)]
use hickory_resolver::TokioAsyncResolver;
#[allow(deprecated)]
use hickory_resolver::name_server::TokioConnectionProvider;

/// Resolver for AT Protocol identities.
///
/// Handles resolution of:
/// - Handle -> DID (via DNS TXT or HTTP well-known)
/// - DID -> PDS URL (via PLC directory or did:web)
pub struct IdentityResolver {
    client: reqwest::Client,
    plc_directory: String,
    #[allow(deprecated)]
    dns_resolver: TokioAsyncResolver,
}

impl Default for IdentityResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityResolver {
    /// Create a new resolver with default settings.
    pub fn new() -> Self {
        #[allow(deprecated)]
        let (config, options) = hickory_resolver::system_conf::read_system_conf().expect("Failed to read system conf");
        #[allow(deprecated)]
        let dns_resolver = TokioAsyncResolver::new(config, options, TokioConnectionProvider::default());

        Self { client: reqwest::Client::new(), plc_directory: "https://plc.directory".to_string(), dns_resolver }
    }

    /// Create a resolver with a custom PLC directory URL.
    pub fn with_plc_directory(plc_directory: &str) -> Self {
        #[allow(deprecated)]
        let (config, options) = hickory_resolver::system_conf::read_system_conf().expect("Failed to read system conf");
        #[allow(deprecated)]
        let dns_resolver = TokioAsyncResolver::new(config, options, TokioConnectionProvider::default());

        Self { client: reqwest::Client::new(), plc_directory: plc_directory.to_string(), dns_resolver }
    }

    /// Resolve a handle to a DID.
    ///
    /// Tries HTTP well-known first, then falls back to DNS TXT.
    pub async fn resolve_handle(&self, handle: &str) -> Result<String, ResolveError> {
        if let Ok(did) = self.resolve_handle_http(handle).await {
            return Ok(did);
        }
        self.resolve_handle_dns(handle).await
    }

    /// Resolve handle via DNS TXT record (_atproto.<handle>).
    async fn resolve_handle_dns(&self, handle: &str) -> Result<String, ResolveError> {
        let query = format!("_atproto.{}", handle);

        match self.dns_resolver.txt_lookup(query).await {
            Ok(records) => {
                for record in records.iter() {
                    let text = record.to_string();
                    if let Some(did) = text.strip_prefix("did=") {
                        return Ok(did.trim().to_string());
                    }
                }
                Err(ResolveError::HandleNotFound(handle.to_string()))
            }
            Err(e) => Err(ResolveError::NetworkError(e.to_string())),
        }
    }

    /// Resolve handle via HTTP well-known.
    async fn resolve_handle_http(&self, handle: &str) -> Result<String, ResolveError> {
        let url = format!("https://{}/.well-known/atproto-did", handle);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ResolveError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ResolveError::HandleNotFound(handle.to_string()));
        }

        let did = response
            .text()
            .await
            .map_err(|e| ResolveError::NetworkError(e.to_string()))?
            .trim()
            .to_string();

        if !did.starts_with("did:") {
            return Err(ResolveError::HandleNotFound(handle.to_string()));
        }

        Ok(did)
    }

    /// Resolve a DID to its PDS URL.
    pub async fn resolve_did(&self, did: &str) -> Result<ResolvedIdentity, ResolveError> {
        if did.starts_with("did:plc:") {
            self.resolve_plc_did(did).await
        } else if did.starts_with("did:web:") {
            self.resolve_web_did(did).await
        } else {
            Err(ResolveError::InvalidDid(did.to_string()))
        }
    }

    /// Resolve a did:plc via the PLC directory.
    async fn resolve_plc_did(&self, did: &str) -> Result<ResolvedIdentity, ResolveError> {
        let url = format!("{}/{}", self.plc_directory, did);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ResolveError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ResolveError::DidNotFound(did.to_string()));
        }

        let doc: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ResolveError::NetworkError(e.to_string()))?;

        let pds_url = doc["service"]
            .as_array()
            .and_then(|services| {
                services.iter().find(|s| {
                    s["id"].as_str() == Some("#atproto_pds") || s["type"].as_str() == Some("AtprotoPersonalDataServer")
                })
            })
            .and_then(|s| s["serviceEndpoint"].as_str())
            .ok_or_else(|| ResolveError::DidNotFound(did.to_string()))?
            .to_string();

        let handle = doc["alsoKnownAs"]
            .as_array()
            .and_then(|aka| {
                aka.iter()
                    .find(|a| a.as_str().map(|s| s.starts_with("at://")).unwrap_or(false))
            })
            .and_then(|a| a.as_str())
            .map(|s| s.strip_prefix("at://").unwrap_or(s).to_string());

        Ok(ResolvedIdentity { did: did.to_string(), handle, pds_url })
    }

    /// Resolve a did:web via HTTP.
    async fn resolve_web_did(&self, did: &str) -> Result<ResolvedIdentity, ResolveError> {
        let domain = did
            .strip_prefix("did:web:")
            .ok_or_else(|| ResolveError::InvalidDid(did.to_string()))?;

        let url = format!("https://{}/.well-known/did.json", domain);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| ResolveError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ResolveError::DidNotFound(did.to_string()));
        }

        let doc: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ResolveError::NetworkError(e.to_string()))?;

        let pds_url = doc["service"]
            .as_array()
            .and_then(|services| {
                services
                    .iter()
                    .find(|s| s["type"].as_str() == Some("AtprotoPersonalDataServer"))
            })
            .and_then(|s| s["serviceEndpoint"].as_str())
            .ok_or_else(|| ResolveError::DidNotFound(did.to_string()))?
            .to_string();

        Ok(ResolvedIdentity { did: did.to_string(), handle: None, pds_url })
    }
}

/// Check if a string is a valid DID.
pub fn is_valid_did(s: &str) -> bool {
    s.starts_with("did:plc:") || s.starts_with("did:web:")
}

/// Check if a string is a valid handle.
pub fn is_valid_handle(s: &str) -> bool {
    s.contains('.') && !s.contains(' ') && !s.starts_with("did:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_did() {
        assert!(is_valid_did("did:plc:abc123"));
        assert!(is_valid_did("did:web:example.com"));
        assert!(!is_valid_did("alice.bsky.social"));
        assert!(!is_valid_did("did:other:xyz"));
    }

    #[test]
    fn test_is_valid_handle() {
        assert!(is_valid_handle("alice.bsky.social"));
        assert!(is_valid_handle("bob.example.com"));
        assert!(!is_valid_handle("did:plc:abc123"));
        assert!(!is_valid_handle("invalid handle"));
        assert!(!is_valid_handle("nodots"));
    }

    #[test]
    fn test_resolver_creation() {
        let resolver = IdentityResolver::new();
        assert_eq!(resolver.plc_directory, "https://plc.directory");

        let custom = IdentityResolver::with_plc_directory("https://custom.plc");
        assert_eq!(custom.plc_directory, "https://custom.plc");
    }

    #[test]
    fn test_resolve_error_display() {
        let err = ResolveError::HandleNotFound("test.handle".to_string());
        assert!(err.to_string().contains("test.handle"));

        let err = ResolveError::InvalidDid("bad:did".to_string());
        assert!(err.to_string().contains("bad:did"));
    }

    #[tokio::test]
    async fn test_resolve_handle_fallback_logic() {
        let resolver = IdentityResolver::new();
        let result = resolver.resolve_handle("nonexistent.invalid").await;
        assert!(matches!(
            result,
            Err(ResolveError::HandleNotFound(_)) | Err(ResolveError::NetworkError(_))
        ));
    }
}
