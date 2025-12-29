//! OAuth token repository for database storage.
//!
//! Handles storage and retrieval of OAuth tokens and sessions.

use crate::db::DbPool;
use crate::oauth::dpop::DpopKeypair;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

/// Stored OAuth token record.
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub did: String,
    pub pds_url: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub dpop_private_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StoredToken {
    /// Reconstruct the DPoP keypair from stored bytes.
    pub fn dpop_keypair(&self) -> Option<DpopKeypair> {
        if self.dpop_private_key.len() != 32 {
            return None;
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&self.dpop_private_key);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        Some(DpopKeypair::from_signing_key(signing_key))
    }
}

/// Error type for OAuth repository operations.
#[derive(Debug, Clone)]
pub enum OAuthRepoError {
    DatabaseError(String),
    NotFound(String),
    SerializationError(String),
}

impl std::fmt::Display for OAuthRepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthRepoError::DatabaseError(e) => write!(f, "Database error: {}", e),
            OAuthRepoError::NotFound(e) => write!(f, "Not found: {}", e),
            OAuthRepoError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for OAuthRepoError {}

/// Request to store OAuth tokens.
pub struct StoreTokensRequest<'a> {
    pub did: &'a str,
    pub pds_url: &'a str,
    pub access_token: &'a str,
    pub refresh_token: Option<&'a str>,
    pub token_type: &'a str,
    pub expires_at: Option<DateTime<Utc>>,
    pub dpop_keypair: &'a DpopKeypair,
}

/// Repository trait for OAuth token operations.
#[async_trait]
pub trait OAuthRepository: Send + Sync {
    /// Store OAuth tokens for a user.
    async fn store_tokens(&self, req: StoreTokensRequest<'_>) -> Result<(), OAuthRepoError>;

    /// Get stored tokens for a user.
    async fn get_tokens(&self, did: &str) -> Result<StoredToken, OAuthRepoError>;

    /// Update tokens after refresh.
    async fn update_tokens(
        &self, did: &str, access_token: &str, refresh_token: Option<&str>, expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), OAuthRepoError>;

    /// Delete tokens for a user (logout).
    async fn delete_tokens(&self, did: &str) -> Result<(), OAuthRepoError>;
}

/// Database-backed OAuth repository.
pub struct DbOAuthRepository {
    pool: DbPool,
}

impl DbOAuthRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OAuthRepository for DbOAuthRepository {
    async fn store_tokens(&self, req: StoreTokensRequest<'_>) -> Result<(), OAuthRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?;

        let dpop_bytes = req.dpop_keypair.private_key_bytes();

        client
            .execute(
                "INSERT INTO oauth_tokens (did, pds_url, access_token, refresh_token, token_type, expires_at, dpop_private_key)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 ON CONFLICT (did) DO UPDATE SET
                     pds_url = EXCLUDED.pds_url,
                     access_token = EXCLUDED.access_token,
                     refresh_token = EXCLUDED.refresh_token,
                     token_type = EXCLUDED.token_type,
                     expires_at = EXCLUDED.expires_at,
                     dpop_private_key = EXCLUDED.dpop_private_key,
                     updated_at = NOW()",
                &[&req.did, &req.pds_url, &req.access_token, &req.refresh_token, &req.token_type, &req.expires_at, &dpop_bytes.as_slice()],
            )
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_tokens(&self, did: &str) -> Result<StoredToken, OAuthRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?;

        let row = client
            .query_opt(
                "SELECT did, pds_url, access_token, refresh_token, token_type, expires_at, dpop_private_key, created_at, updated_at
                 FROM oauth_tokens WHERE did = $1",
                &[&did],
            )
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?
            .ok_or_else(|| OAuthRepoError::NotFound(format!("No tokens for DID: {}", did)))?;

        Ok(StoredToken {
            did: row.get("did"),
            pds_url: row.get("pds_url"),
            access_token: row.get("access_token"),
            refresh_token: row.get("refresh_token"),
            token_type: row.get("token_type"),
            expires_at: row.get("expires_at"),
            dpop_private_key: row.get("dpop_private_key"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn update_tokens(
        &self, did: &str, access_token: &str, refresh_token: Option<&str>, expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), OAuthRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?;

        let result = client
            .execute(
                "UPDATE oauth_tokens SET access_token = $2, refresh_token = $3, expires_at = $4, updated_at = NOW()
                 WHERE did = $1",
                &[&did, &access_token, &refresh_token, &expires_at],
            )
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?;

        if result == 0 {
            return Err(OAuthRepoError::NotFound(format!("No tokens for DID: {}", did)));
        }

        Ok(())
    }

    async fn delete_tokens(&self, did: &str) -> Result<(), OAuthRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?;

        client
            .execute("DELETE FROM oauth_tokens WHERE did = $1", &[&did])
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_repo_error_display() {
        let err = OAuthRepoError::NotFound("test".to_string());
        assert!(err.to_string().contains("test"));

        let err = OAuthRepoError::DatabaseError("connection failed".to_string());
        assert!(err.to_string().contains("connection failed"));
    }
}
