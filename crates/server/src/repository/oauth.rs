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
///
/// Supports both OAuth sessions (with DPoP) and app password sessions (without DPoP).
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub did: String,
    pub pds_url: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub dpop_private_key: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StoredToken {
    /// Reconstruct the DPoP keypair from stored bytes.
    ///
    /// Returns None for app password sessions (no DPoP) or if the key is invalid.
    pub fn dpop_keypair(&self) -> Option<DpopKeypair> {
        let key_bytes_vec = self.dpop_private_key.as_ref()?;
        if key_bytes_vec.len() != 32 {
            return None;
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(key_bytes_vec);
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

/// Request to store OAuth tokens with DPoP.
pub struct StoreTokensRequest<'a> {
    pub did: &'a str,
    pub pds_url: &'a str,
    pub access_token: &'a str,
    pub refresh_token: Option<&'a str>,
    pub token_type: &'a str,
    pub expires_at: Option<DateTime<Utc>>,
    pub dpop_keypair: &'a DpopKeypair,
}

/// Request to store app password session (without DPoP).
pub struct StoreAppPasswordSessionRequest<'a> {
    pub did: &'a str,
    pub pds_url: &'a str,
    pub access_token: &'a str,
    pub refresh_token: Option<&'a str>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Repository trait for OAuth token operations.
#[async_trait]
pub trait OAuthRepository: Send + Sync {
    /// Store OAuth tokens for a user (with DPoP).
    async fn store_tokens(&self, req: StoreTokensRequest<'_>) -> Result<(), OAuthRepoError>;

    /// Store app password session for a user (without DPoP).
    async fn store_app_password_session(&self, req: StoreAppPasswordSessionRequest<'_>) -> Result<(), OAuthRepoError>;

    /// Get stored tokens for a user.
    async fn get_tokens(&self, did: &str) -> Result<StoredToken, OAuthRepoError>;

    /// Get stored tokens by access token.
    async fn get_token_by_access_token(&self, access_token: &str) -> Result<StoredToken, OAuthRepoError>;

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

    async fn store_app_password_session(&self, req: StoreAppPasswordSessionRequest<'_>) -> Result<(), OAuthRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?;

        client
            .execute(
                "INSERT INTO oauth_tokens (did, pds_url, access_token, refresh_token, token_type, expires_at, dpop_private_key)
                 VALUES ($1, $2, $3, $4, 'Bearer', $5, NULL)
                 ON CONFLICT (did) DO UPDATE SET
                     pds_url = EXCLUDED.pds_url,
                     access_token = EXCLUDED.access_token,
                     refresh_token = EXCLUDED.refresh_token,
                     expires_at = EXCLUDED.expires_at,
                     dpop_private_key = NULL,
                     updated_at = NOW()",
                &[&req.did, &req.pds_url, &req.access_token, &req.refresh_token, &req.expires_at],
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

    async fn get_token_by_access_token(&self, access_token: &str) -> Result<StoredToken, OAuthRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?;

        let row = client
            .query_opt(
                "SELECT did, pds_url, access_token, refresh_token, token_type, expires_at, dpop_private_key, created_at, updated_at
                 FROM oauth_tokens WHERE access_token = $1",
                &[&access_token],
            )
            .await
            .map_err(|e| OAuthRepoError::DatabaseError(e.to_string()))?
            .ok_or_else(|| OAuthRepoError::NotFound("Token not found".to_string()))?;

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

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct MockOAuthRepository {
        pub tokens: Arc<Mutex<Vec<StoredToken>>>,
        pub should_fail: Arc<Mutex<bool>>,
    }

    impl MockOAuthRepository {
        pub fn new() -> Self {
            Self { tokens: Arc::new(Mutex::new(Vec::new())), should_fail: Arc::new(Mutex::new(false)) }
        }

        pub fn with_tokens(tokens: Vec<StoredToken>) -> Self {
            Self { tokens: Arc::new(Mutex::new(tokens)), should_fail: Arc::new(Mutex::new(false)) }
        }

        pub fn set_should_fail(&self, should_fail: bool) {
            *self.should_fail.lock().unwrap() = should_fail;
        }
    }

    impl Default for MockOAuthRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl OAuthRepository for MockOAuthRepository {
        async fn store_tokens(&self, req: StoreTokensRequest<'_>) -> Result<(), OAuthRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(OAuthRepoError::DatabaseError("Mock failure".to_string()));
            }

            let token = StoredToken {
                did: req.did.to_string(),
                pds_url: req.pds_url.to_string(),
                access_token: req.access_token.to_string(),
                refresh_token: req.refresh_token.map(String::from),
                token_type: req.token_type.to_string(),
                expires_at: req.expires_at,
                dpop_private_key: Some(req.dpop_keypair.private_key_bytes()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            self.tokens.lock().unwrap().push(token);
            Ok(())
        }

        async fn store_app_password_session(&self, req: StoreAppPasswordSessionRequest<'_>) -> Result<(), OAuthRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(OAuthRepoError::DatabaseError("Mock failure".to_string()));
            }

            let token = StoredToken {
                did: req.did.to_string(),
                pds_url: req.pds_url.to_string(),
                access_token: req.access_token.to_string(),
                refresh_token: req.refresh_token.map(String::from),
                token_type: "Bearer".to_string(),
                expires_at: req.expires_at,
                dpop_private_key: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            self.tokens.lock().unwrap().push(token);
            Ok(())
        }

        async fn get_tokens(&self, did: &str) -> Result<StoredToken, OAuthRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(OAuthRepoError::DatabaseError("Mock failure".to_string()));
            }

            let tokens = self.tokens.lock().unwrap();
            tokens
                .iter()
                .find(|t| t.did == did)
                .cloned()
                .ok_or_else(|| OAuthRepoError::NotFound(format!("No tokens for DID: {}", did)))
        }

        async fn get_token_by_access_token(&self, access_token: &str) -> Result<StoredToken, OAuthRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(OAuthRepoError::DatabaseError("Mock failure".to_string()));
            }

            let tokens = self.tokens.lock().unwrap();
            tokens
                .iter()
                .find(|t| t.access_token == access_token)
                .cloned()
                .ok_or_else(|| OAuthRepoError::NotFound("Token not found".to_string()))
        }

        async fn update_tokens(
            &self, did: &str, access_token: &str, refresh_token: Option<&str>, expires_at: Option<DateTime<Utc>>,
        ) -> Result<(), OAuthRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(OAuthRepoError::DatabaseError("Mock failure".to_string()));
            }

            let mut tokens = self.tokens.lock().unwrap();
            if let Some(token) = tokens.iter_mut().find(|t| t.did == did) {
                token.access_token = access_token.to_string();
                token.refresh_token = refresh_token.map(String::from);
                token.expires_at = expires_at;
                token.updated_at = Utc::now();
                Ok(())
            } else {
                Err(OAuthRepoError::NotFound(format!("No tokens for DID: {}", did)))
            }
        }

        async fn delete_tokens(&self, did: &str) -> Result<(), OAuthRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(OAuthRepoError::DatabaseError("Mock failure".to_string()));
            }

            let mut tokens = self.tokens.lock().unwrap();
            tokens.retain(|t| t.did != did);
            Ok(())
        }
    }
}
