//! Sync repository for tracking synchronization state.
//!
//! Manages the sync status of entities between local database and PDS.

use std::str::FromStr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Sync status for an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    /// Never synced to PDS
    LocalOnly,
    /// In sync with PDS
    Synced,
    /// Local changes need to be pushed
    PendingPush,
    /// Local and remote both changed (conflict)
    Conflict,
}

impl SyncStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncStatus::LocalOnly => "local_only",
            SyncStatus::Synced => "synced",
            SyncStatus::PendingPush => "pending_push",
            SyncStatus::Conflict => "conflict",
        }
    }
}

impl FromStr for SyncStatus {
    type Err = std::fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local_only" => Ok(SyncStatus::LocalOnly),
            "synced" => Ok(SyncStatus::Synced),
            "pending_push" => Ok(SyncStatus::PendingPush),
            "conflict" => Ok(SyncStatus::Conflict),
            _ => Err(std::fmt::Error),
        }
    }
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending sync operation.
#[derive(Debug, Clone)]
pub struct PendingSync {
    pub entity_type: String,
    pub entity_id: String,
    pub owner_did: String,
    pub version: i32,
    pub status: SyncStatus,
}

/// Sync metadata for an entity.
#[derive(Debug, Clone)]
pub struct SyncMetadata {
    pub entity_type: String,
    pub entity_id: String,
    pub version: i32,
    pub pds_cid: Option<String>,
    pub pds_uri: Option<String>,
    pub status: SyncStatus,
    pub last_synced_at: Option<DateTime<Utc>>,
}

/// Entry in the sync log.
#[derive(Debug, Clone)]
pub struct SyncLogEntry {
    pub id: String,
    pub owner_did: String,
    pub entity_type: String,
    pub entity_id: String,
    pub operation: String,
    pub status: String,
    pub pds_cid: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Parameters for logging a sync operation.
#[derive(Debug, Clone)]
pub struct LogOperationParams<'a> {
    pub owner_did: &'a str,
    pub entity_type: &'a str,
    pub entity_id: &'a str,
    pub operation: &'a str,
    pub status: &'a str,
    pub pds_cid: Option<&'a str>,
    pub error_message: Option<&'a str>,
}

/// Error type for sync repository operations.
#[derive(Debug)]
pub enum SyncRepoError {
    DatabaseError(String),
    NotFound(String),
    InvalidArgument(String),
}

impl std::fmt::Display for SyncRepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncRepoError::DatabaseError(e) => write!(f, "Database error: {}", e),
            SyncRepoError::NotFound(e) => write!(f, "Not found: {}", e),
            SyncRepoError::InvalidArgument(e) => write!(f, "Invalid argument: {}", e),
        }
    }
}

impl std::error::Error for SyncRepoError {}

/// Repository trait for sync operations.
#[async_trait]
pub trait SyncRepository: Send + Sync {
    /// Get sync metadata for an entity.
    async fn get_sync_metadata(&self, entity_type: &str, id: &str) -> Result<SyncMetadata, SyncRepoError>;

    /// Mark an entity as synced with given PDS CID and URI.
    async fn mark_synced(&self, entity_type: &str, id: &str, pds_cid: &str, pds_uri: &str)
    -> Result<(), SyncRepoError>;

    /// Mark an entity as pending push.
    async fn mark_pending(&self, entity_type: &str, id: &str) -> Result<(), SyncRepoError>;

    /// Mark an entity as having a conflict.
    async fn mark_conflict(&self, entity_type: &str, id: &str) -> Result<(), SyncRepoError>;

    /// Get all pending items for a user.
    async fn get_pending_items(&self, owner_did: &str) -> Result<Vec<PendingSync>, SyncRepoError>;

    /// Get all conflicts for a user.
    async fn get_conflicts(&self, owner_did: &str) -> Result<Vec<PendingSync>, SyncRepoError>;

    /// Increment version for an entity (used when resolving conflicts).
    async fn increment_version(&self, entity_type: &str, id: &str) -> Result<i32, SyncRepoError>;

    /// Log a sync operation.
    async fn log_operation(&self, params: LogOperationParams<'_>) -> Result<String, SyncRepoError>;

    /// Mark a sync log entry as completed.
    async fn complete_log_entry(
        &self, log_id: &str, status: &str, pds_cid: Option<&str>, error_message: Option<&str>,
    ) -> Result<(), SyncRepoError>;
}

/// Database implementation of SyncRepository.
pub struct DbSyncRepository {
    pool: crate::db::DbPool,
}

impl DbSyncRepository {
    pub fn new(pool: crate::db::DbPool) -> Self {
        Self { pool }
    }

    fn table_for_entity(&self, entity_type: &str) -> Result<&'static str, SyncRepoError> {
        match entity_type {
            "deck" => Ok("decks"),
            "card" => Ok("cards"),
            "note" => Ok("notes"),
            _ => Err(SyncRepoError::InvalidArgument(format!(
                "Unknown entity type: {}",
                entity_type
            ))),
        }
    }
}

#[async_trait]
impl SyncRepository for DbSyncRepository {
    async fn get_sync_metadata(&self, entity_type: &str, id: &str) -> Result<SyncMetadata, SyncRepoError> {
        let table = self.table_for_entity(entity_type)?;
        let uuid = Uuid::parse_str(id).map_err(|e| SyncRepoError::InvalidArgument(format!("Invalid UUID: {}", e)))?;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = format!(
            "SELECT version, pds_cid, pds_uri, sync_status::text, last_synced_at FROM {} WHERE id = $1",
            table
        );

        let row = client
            .query_opt(&query, &[&uuid])
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to query: {}", e)))?
            .ok_or_else(|| SyncRepoError::NotFound(format!("{} not found: {}", entity_type, id)))?;

        let version: i32 = row.get("version");
        let pds_cid: Option<String> = row.get("pds_cid");
        let pds_uri: Option<String> = row.get("pds_uri");
        let status_str: String = row.get("sync_status");
        let last_synced_at: Option<DateTime<Utc>> = row.get("last_synced_at");

        Ok(SyncMetadata {
            entity_type: entity_type.to_string(),
            entity_id: id.to_string(),
            version,
            pds_cid,
            pds_uri,
            status: SyncStatus::from_str(&status_str).unwrap_or(SyncStatus::LocalOnly),
            last_synced_at,
        })
    }

    async fn mark_synced(
        &self, entity_type: &str, id: &str, pds_cid: &str, pds_uri: &str,
    ) -> Result<(), SyncRepoError> {
        let table = self.table_for_entity(entity_type)?;
        let uuid = Uuid::parse_str(id).map_err(|e| SyncRepoError::InvalidArgument(format!("Invalid UUID: {}", e)))?;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = format!(
            "UPDATE {} SET sync_status = 'synced', pds_cid = $1, pds_uri = $2, last_synced_at = NOW() WHERE id = $3",
            table
        );

        client
            .execute(&query, &[&pds_cid, &pds_uri, &uuid])
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to update: {}", e)))?;

        Ok(())
    }

    async fn mark_pending(&self, entity_type: &str, id: &str) -> Result<(), SyncRepoError> {
        let table = self.table_for_entity(entity_type)?;
        let uuid = Uuid::parse_str(id).map_err(|e| SyncRepoError::InvalidArgument(format!("Invalid UUID: {}", e)))?;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = format!("UPDATE {} SET sync_status = 'pending_push' WHERE id = $1", table);

        client
            .execute(&query, &[&uuid])
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to update: {}", e)))?;

        Ok(())
    }

    async fn mark_conflict(&self, entity_type: &str, id: &str) -> Result<(), SyncRepoError> {
        let table = self.table_for_entity(entity_type)?;
        let uuid = Uuid::parse_str(id).map_err(|e| SyncRepoError::InvalidArgument(format!("Invalid UUID: {}", e)))?;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = format!("UPDATE {} SET sync_status = 'conflict' WHERE id = $1", table);

        client
            .execute(&query, &[&uuid])
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to update: {}", e)))?;

        Ok(())
    }

    async fn get_pending_items(&self, owner_did: &str) -> Result<Vec<PendingSync>, SyncRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut pending = Vec::new();

        for (entity_type, table) in [("deck", "decks"), ("card", "cards"), ("note", "notes")] {
            let query = format!(
                "SELECT id, version, sync_status::text FROM {} WHERE owner_did = $1 AND sync_status = 'pending_push'",
                table
            );

            let rows = client
                .query(&query, &[&owner_did])
                .await
                .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to query: {}", e)))?;

            for row in rows {
                let id: Uuid = row.get("id");
                let version: i32 = row.get("version");
                let status_str: String = row.get("sync_status");

                pending.push(PendingSync {
                    entity_type: entity_type.to_string(),
                    entity_id: id.to_string(),
                    owner_did: owner_did.to_string(),
                    version,
                    status: SyncStatus::from_str(&status_str).unwrap_or(SyncStatus::PendingPush),
                });
            }
        }

        Ok(pending)
    }

    async fn get_conflicts(&self, owner_did: &str) -> Result<Vec<PendingSync>, SyncRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let mut conflicts = Vec::new();

        for (entity_type, table) in [("deck", "decks"), ("card", "cards"), ("note", "notes")] {
            let query = format!(
                "SELECT id, version, sync_status::text FROM {} WHERE owner_did = $1 AND sync_status = 'conflict'",
                table
            );

            let rows = client
                .query(&query, &[&owner_did])
                .await
                .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to query: {}", e)))?;

            for row in rows {
                let id: Uuid = row.get("id");
                let version: i32 = row.get("version");

                conflicts.push(PendingSync {
                    entity_type: entity_type.to_string(),
                    entity_id: id.to_string(),
                    owner_did: owner_did.to_string(),
                    version,
                    status: SyncStatus::Conflict,
                });
            }
        }

        Ok(conflicts)
    }

    async fn increment_version(&self, entity_type: &str, id: &str) -> Result<i32, SyncRepoError> {
        let table = self.table_for_entity(entity_type)?;
        let uuid = Uuid::parse_str(id).map_err(|e| SyncRepoError::InvalidArgument(format!("Invalid UUID: {}", e)))?;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = format!(
            "UPDATE {} SET version = version + 1 WHERE id = $1 RETURNING version",
            table
        );

        let row = client
            .query_one(&query, &[&uuid])
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to update: {}", e)))?;

        let version: i32 = row.get("version");
        Ok(version)
    }

    async fn log_operation(&self, params: LogOperationParams<'_>) -> Result<String, SyncRepoError> {
        let entity_uuid = Uuid::parse_str(params.entity_id)
            .map_err(|e| SyncRepoError::InvalidArgument(format!("Invalid UUID: {}", e)))?;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let row = client
            .query_one(
                "INSERT INTO sync_log (owner_did, entity_type, entity_id, operation, status, pds_cid, error_message)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 RETURNING id",
                &[
                    &params.owner_did,
                    &params.entity_type,
                    &entity_uuid,
                    &params.operation,
                    &params.status,
                    &params.pds_cid,
                    &params.error_message,
                ],
            )
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to insert log: {}", e)))?;

        let id: Uuid = row.get("id");
        Ok(id.to_string())
    }

    async fn complete_log_entry(
        &self, log_id: &str, status: &str, pds_cid: Option<&str>, error_message: Option<&str>,
    ) -> Result<(), SyncRepoError> {
        let uuid =
            Uuid::parse_str(log_id).map_err(|e| SyncRepoError::InvalidArgument(format!("Invalid UUID: {}", e)))?;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        client
            .execute(
                "UPDATE sync_log
                 SET status = $1, pds_cid = COALESCE($2, pds_cid), error_message = $3, completed_at = NOW()
                 WHERE id = $4",
                &[&status, &pds_cid, &error_message, &uuid],
            )
            .await
            .map_err(|e| SyncRepoError::DatabaseError(format!("Failed to update log: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct MockSyncRepository {
        metadata: Arc<Mutex<HashMap<String, SyncMetadata>>>,
        logs: Arc<Mutex<Vec<SyncLogEntry>>>,
    }

    impl MockSyncRepository {
        pub fn new() -> Self {
            Self { metadata: Arc::new(Mutex::new(HashMap::new())), logs: Arc::new(Mutex::new(Vec::new())) }
        }

        pub fn with_metadata(metadata: Vec<SyncMetadata>) -> Self {
            let map: HashMap<String, SyncMetadata> = metadata
                .into_iter()
                .map(|m| (format!("{}:{}", m.entity_type, m.entity_id), m))
                .collect();
            Self { metadata: Arc::new(Mutex::new(map)), logs: Arc::new(Mutex::new(Vec::new())) }
        }

        fn key(entity_type: &str, id: &str) -> String {
            format!("{}:{}", entity_type, id)
        }
    }

    impl Default for MockSyncRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl SyncRepository for MockSyncRepository {
        async fn get_sync_metadata(&self, entity_type: &str, id: &str) -> Result<SyncMetadata, SyncRepoError> {
            let key = Self::key(entity_type, id);
            self.metadata
                .lock()
                .unwrap()
                .get(&key)
                .cloned()
                .ok_or_else(|| SyncRepoError::NotFound(format!("{} not found: {}", entity_type, id)))
        }

        async fn mark_synced(
            &self, entity_type: &str, id: &str, pds_cid: &str, pds_uri: &str,
        ) -> Result<(), SyncRepoError> {
            let key = Self::key(entity_type, id);
            let mut map = self.metadata.lock().unwrap();
            if let Some(meta) = map.get_mut(&key) {
                meta.status = SyncStatus::Synced;
                meta.pds_cid = Some(pds_cid.to_string());
                meta.pds_uri = Some(pds_uri.to_string());
                meta.last_synced_at = Some(Utc::now());
            }
            Ok(())
        }

        async fn mark_pending(&self, entity_type: &str, id: &str) -> Result<(), SyncRepoError> {
            let key = Self::key(entity_type, id);
            let mut map = self.metadata.lock().unwrap();
            if let Some(meta) = map.get_mut(&key) {
                meta.status = SyncStatus::PendingPush;
            }
            Ok(())
        }

        async fn mark_conflict(&self, entity_type: &str, id: &str) -> Result<(), SyncRepoError> {
            let key = Self::key(entity_type, id);
            let mut map = self.metadata.lock().unwrap();
            if let Some(meta) = map.get_mut(&key) {
                meta.status = SyncStatus::Conflict;
            }
            Ok(())
        }

        async fn get_pending_items(&self, owner_did: &str) -> Result<Vec<PendingSync>, SyncRepoError> {
            let map = self.metadata.lock().unwrap();
            let pending: Vec<_> = map
                .values()
                .filter(|m| m.status == SyncStatus::PendingPush)
                .map(|m| PendingSync {
                    entity_type: m.entity_type.clone(),
                    entity_id: m.entity_id.clone(),
                    owner_did: owner_did.to_string(),
                    version: m.version,
                    status: m.status,
                })
                .collect();
            Ok(pending)
        }

        async fn get_conflicts(&self, owner_did: &str) -> Result<Vec<PendingSync>, SyncRepoError> {
            let map = self.metadata.lock().unwrap();
            let conflicts: Vec<_> = map
                .values()
                .filter(|m| m.status == SyncStatus::Conflict)
                .map(|m| PendingSync {
                    entity_type: m.entity_type.clone(),
                    entity_id: m.entity_id.clone(),
                    owner_did: owner_did.to_string(),
                    version: m.version,
                    status: m.status,
                })
                .collect();
            Ok(conflicts)
        }

        async fn increment_version(&self, entity_type: &str, id: &str) -> Result<i32, SyncRepoError> {
            let key = Self::key(entity_type, id);
            let mut map = self.metadata.lock().unwrap();
            if let Some(meta) = map.get_mut(&key) {
                meta.version += 1;
                Ok(meta.version)
            } else {
                Err(SyncRepoError::NotFound(format!("{} not found: {}", entity_type, id)))
            }
        }

        async fn log_operation(&self, params: LogOperationParams<'_>) -> Result<String, SyncRepoError> {
            let id = Uuid::new_v4().to_string();
            let entry = SyncLogEntry {
                id: id.clone(),
                owner_did: params.owner_did.to_string(),
                entity_type: params.entity_type.to_string(),
                entity_id: params.entity_id.to_string(),
                operation: params.operation.to_string(),
                status: params.status.to_string(),
                pds_cid: params.pds_cid.map(String::from),
                error_message: params.error_message.map(String::from),
                created_at: Utc::now(),
                completed_at: None,
            };
            self.logs.lock().unwrap().push(entry);
            Ok(id)
        }

        async fn complete_log_entry(
            &self, log_id: &str, status: &str, pds_cid: Option<&str>, error_message: Option<&str>,
        ) -> Result<(), SyncRepoError> {
            let mut logs = self.logs.lock().unwrap();
            if let Some(entry) = logs.iter_mut().find(|e| e.id == log_id) {
                entry.status = status.to_string();
                entry.pds_cid = pds_cid.map(String::from).or(entry.pds_cid.clone());
                entry.error_message = error_message.map(String::from);
                entry.completed_at = Some(Utc::now());
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mock::MockSyncRepository;
    use super::*;

    #[test]
    fn test_sync_status_as_str() {
        assert_eq!(SyncStatus::LocalOnly.as_str(), "local_only");
        assert_eq!(SyncStatus::Synced.as_str(), "synced");
        assert_eq!(SyncStatus::PendingPush.as_str(), "pending_push");
        assert_eq!(SyncStatus::Conflict.as_str(), "conflict");
    }

    #[test]
    fn test_sync_status_from_str() {
        assert_eq!(SyncStatus::from_str("local_only").unwrap(), SyncStatus::LocalOnly);
        assert_eq!(SyncStatus::from_str("synced").unwrap(), SyncStatus::Synced);
        assert_eq!(SyncStatus::from_str("pending_push").unwrap(), SyncStatus::PendingPush);
        assert_eq!(SyncStatus::from_str("conflict").unwrap(), SyncStatus::Conflict);
        assert!(SyncStatus::from_str("unknown").is_err());
    }

    #[tokio::test]
    async fn test_mock_sync_repo_get_metadata() {
        let metadata = SyncMetadata {
            entity_type: "deck".to_string(),
            entity_id: "123".to_string(),
            version: 1,
            pds_cid: None,
            pds_uri: None,
            status: SyncStatus::LocalOnly,
            last_synced_at: None,
        };
        let repo = MockSyncRepository::with_metadata(vec![metadata]);

        let result = repo.get_sync_metadata("deck", "123").await;
        assert!(result.is_ok());
        let meta = result.unwrap();
        assert_eq!(meta.entity_type, "deck");
        assert_eq!(meta.version, 1);
        assert_eq!(meta.status, SyncStatus::LocalOnly);
    }

    #[tokio::test]
    async fn test_mock_sync_repo_mark_synced() {
        let metadata = SyncMetadata {
            entity_type: "deck".to_string(),
            entity_id: "123".to_string(),
            version: 1,
            pds_cid: None,
            pds_uri: None,
            status: SyncStatus::PendingPush,
            last_synced_at: None,
        };
        let repo = MockSyncRepository::with_metadata(vec![metadata]);

        repo.mark_synced("deck", "123", "bafycid123", "at://did:plc:test/deck/123")
            .await
            .unwrap();

        let meta = repo.get_sync_metadata("deck", "123").await.unwrap();
        assert_eq!(meta.status, SyncStatus::Synced);
        assert_eq!(meta.pds_cid, Some("bafycid123".to_string()));
        assert!(meta.last_synced_at.is_some());
    }

    #[tokio::test]
    async fn test_mock_sync_repo_increment_version() {
        let metadata = SyncMetadata {
            entity_type: "note".to_string(),
            entity_id: "456".to_string(),
            version: 5,
            pds_cid: None,
            pds_uri: None,
            status: SyncStatus::Synced,
            last_synced_at: None,
        };
        let repo = MockSyncRepository::with_metadata(vec![metadata]);

        let new_version = repo.increment_version("note", "456").await.unwrap();
        assert_eq!(new_version, 6);

        let meta = repo.get_sync_metadata("note", "456").await.unwrap();
        assert_eq!(meta.version, 6);
    }

    #[tokio::test]
    async fn test_mock_sync_repo_log_operation() {
        let repo = MockSyncRepository::new();

        let log_id = repo
            .log_operation(LogOperationParams {
                owner_did: "did:plc:test",
                entity_type: "deck",
                entity_id: "123e4567-e89b-12d3-a456-426614174000",
                operation: "push",
                status: "pending",
                pds_cid: None,
                error_message: None,
            })
            .await
            .unwrap();

        assert!(!log_id.is_empty());

        repo.complete_log_entry(&log_id, "success", Some("bafycid"), None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_mock_sync_repo_get_pending() {
        let metadata = vec![
            SyncMetadata {
                entity_type: "deck".to_string(),
                entity_id: "1".to_string(),
                version: 1,
                pds_cid: None,
                pds_uri: None,
                status: SyncStatus::PendingPush,
                last_synced_at: None,
            },
            SyncMetadata {
                entity_type: "note".to_string(),
                entity_id: "2".to_string(),
                version: 1,
                pds_cid: None,
                pds_uri: None,
                status: SyncStatus::Synced,
                last_synced_at: None,
            },
        ];
        let repo = MockSyncRepository::with_metadata(metadata);

        let pending = repo.get_pending_items("did:plc:test").await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].entity_type, "deck");
    }

    #[test]
    fn test_sync_repo_error_display() {
        let err = SyncRepoError::DatabaseError("connection failed".to_string());
        assert!(err.to_string().contains("Database error"));

        let err = SyncRepoError::NotFound("deck:123".to_string());
        assert!(err.to_string().contains("Not found"));

        let err = SyncRepoError::InvalidArgument("bad uuid".to_string());
        assert!(err.to_string().contains("Invalid argument"));
    }
}
