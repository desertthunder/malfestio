//! Sync service for coordinating bi-directional PDS synchronization.
//!
//! Handles push/pull operations and conflict resolution.

use crate::middleware::auth::UserContext;
use crate::pds::client::{GetRecordResult, PdsClient, PdsError};
use crate::pds::records::{prepare_card_record, prepare_deck_record, prepare_note_record};
use crate::repository::card::CardRepository;
use crate::repository::deck::DeckRepository;
use crate::repository::note::NoteRepository;
use crate::repository::oauth::OAuthRepository;
use crate::repository::sync::{LogOperationParams, SyncRepoError, SyncRepository, SyncStatus};
use std::str::FromStr;
use std::sync::Arc;

/// Result of a sync operation.
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub entity_type: String,
    pub entity_id: String,
    pub pds_uri: Option<String>,
    pub pds_cid: Option<String>,
    pub new_version: i32,
    pub status: SyncStatus,
}

/// Conflict information for UI display.
#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub entity_type: String,
    pub entity_id: String,
    pub local_version: i32,
    pub remote_version: Option<i32>,
    pub local_updated_at: Option<String>,
    pub remote_updated_at: Option<String>,
}

/// Summary of sync status for a user.
#[derive(Debug, Clone)]
pub struct SyncStatusSummary {
    pub pending_count: usize,
    pub conflict_count: usize,
    pub pending_items: Vec<(String, String)>,
    pub conflicts: Vec<ConflictInfo>,
}

/// Conflict resolution strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Use the most recently modified version (default)
    LastWriteWins,
    /// Keep local version, overwrite remote
    KeepLocal,
    /// Keep remote version, overwrite local
    KeepRemote,
    // TODO: MergeUI - Show UI for manual merge
}

impl ConflictStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConflictStrategy::LastWriteWins => "last_write_wins",
            ConflictStrategy::KeepLocal => "keep_local",
            ConflictStrategy::KeepRemote => "keep_remote",
        }
    }
}

impl FromStr for ConflictStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "last_write_wins" => Ok(ConflictStrategy::LastWriteWins),
            "keep_local" => Ok(ConflictStrategy::KeepLocal),
            "keep_remote" => Ok(ConflictStrategy::KeepRemote),
            _ => Err(format!("Invalid conflict strategy: {}", s)),
        }
    }
}

/// Error type for sync operations.
#[derive(Debug)]
pub enum SyncError {
    /// Entity not found
    NotFound(String),
    /// Authentication required
    AuthRequired(String),
    /// No OAuth tokens available
    NoTokens(String),
    /// PDS operation failed
    PdsError(PdsError),
    /// Repository error
    RepoError(SyncRepoError),
    /// Invalid argument
    InvalidArgument(String),
    /// Conflict detected
    ConflictDetected(ConflictInfo),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::NotFound(e) => write!(f, "Not found: {}", e),
            SyncError::AuthRequired(e) => write!(f, "Authentication required: {}", e),
            SyncError::NoTokens(e) => write!(f, "No OAuth tokens: {}", e),
            SyncError::PdsError(e) => write!(f, "PDS error: {}", e),
            SyncError::RepoError(e) => write!(f, "Repository error: {}", e),
            SyncError::InvalidArgument(e) => write!(f, "Invalid argument: {}", e),
            SyncError::ConflictDetected(c) => {
                write!(f, "Conflict detected for {}:{}", c.entity_type, c.entity_id)
            }
        }
    }
}

impl std::error::Error for SyncError {}

impl From<SyncRepoError> for SyncError {
    fn from(e: SyncRepoError) -> Self {
        SyncError::RepoError(e)
    }
}

impl From<PdsError> for SyncError {
    fn from(e: PdsError) -> Self {
        SyncError::PdsError(e)
    }
}

/// Remote record data retrieved from PDS.
#[derive(Debug, Clone)]
pub struct RemoteRecord {
    pub uri: String,
    pub cid: String,
    pub value: serde_json::Value,
}

/// Sync service for coordinating sync operations.
pub struct SyncService {
    sync_repo: Arc<dyn SyncRepository>,
    deck_repo: Arc<dyn DeckRepository>,
    card_repo: Arc<dyn CardRepository>,
    note_repo: Arc<dyn NoteRepository>,
    oauth_repo: Arc<dyn OAuthRepository>,
}

impl SyncService {
    pub fn new(
        sync_repo: Arc<dyn SyncRepository>, deck_repo: Arc<dyn DeckRepository>, card_repo: Arc<dyn CardRepository>,
        note_repo: Arc<dyn NoteRepository>, oauth_repo: Arc<dyn OAuthRepository>,
    ) -> Self {
        Self { sync_repo, deck_repo, card_repo, note_repo, oauth_repo }
    }

    /// Push a local deck to the user's PDS.
    pub async fn push_deck(&self, deck_id: &str, user_ctx: &UserContext) -> Result<SyncResult, SyncError> {
        // Log the operation
        let log_id = self
            .sync_repo
            .log_operation(LogOperationParams {
                owner_did: &user_ctx.did,
                entity_type: "deck",
                entity_id: deck_id,
                operation: "push",
                status: "pending",
                pds_cid: None,
                error_message: None,
            })
            .await?;

        // Get PDS client
        let pds_client = self.get_pds_client(user_ctx).await?;

        // Get deck from repository
        let deck = self
            .deck_repo
            .get(deck_id)
            .await
            .map_err(|e| SyncError::NotFound(format!("Deck not found: {:?}", e)))?;

        // Get cards for the deck
        let cards = self
            .card_repo
            .list_by_deck(deck_id)
            .await
            .map_err(|e| SyncError::RepoError(SyncRepoError::DatabaseError(format!("{:?}", e))))?;

        // Push cards first, collect AT-URIs
        let mut card_at_uris = Vec::with_capacity(cards.len());
        for card in &cards {
            let prepared = prepare_card_record(card, ""); // deck_ref filled later
            let at_uri = pds_client
                .put_record(&user_ctx.did, &prepared.collection, &prepared.rkey, prepared.record)
                .await?;
            card_at_uris.push(at_uri.to_string());

            // Mark card as synced
            self.sync_repo
                .mark_synced("card", &card.id, "", &at_uri.to_string())
                .await?;
        }

        // Push deck with card refs
        let prepared = prepare_deck_record(&deck, card_at_uris);
        let at_uri = pds_client
            .put_record(&user_ctx.did, &prepared.collection, &prepared.rkey, prepared.record)
            .await?;

        // Mark deck as synced
        self.sync_repo
            .mark_synced("deck", deck_id, "", &at_uri.to_string())
            .await?;

        let metadata = self.sync_repo.get_sync_metadata("deck", deck_id).await?;

        // Complete log entry
        self.sync_repo
            .complete_log_entry(&log_id, "success", metadata.pds_cid.as_deref(), None)
            .await?;

        Ok(SyncResult {
            entity_type: "deck".to_string(),
            entity_id: deck_id.to_string(),
            pds_uri: Some(at_uri.to_string()),
            pds_cid: metadata.pds_cid,
            new_version: metadata.version,
            status: SyncStatus::Synced,
        })
    }

    /// Push a local note to the user's PDS.
    pub async fn push_note(&self, note_id: &str, user_ctx: &UserContext) -> Result<SyncResult, SyncError> {
        // Log the operation
        let log_id = self
            .sync_repo
            .log_operation(LogOperationParams {
                owner_did: &user_ctx.did,
                entity_type: "note",
                entity_id: note_id,
                operation: "push",
                status: "pending",
                pds_cid: None,
                error_message: None,
            })
            .await?;

        // Get PDS client
        let pds_client = self.get_pds_client(user_ctx).await?;

        // Get note from repository
        let note = self
            .note_repo
            .get(note_id, Some(&user_ctx.did))
            .await
            .map_err(|e| SyncError::NotFound(format!("Note not found: {:?}", e)))?;

        let prepared = prepare_note_record(&note);
        let at_uri = pds_client
            .put_record(&user_ctx.did, &prepared.collection, &prepared.rkey, prepared.record)
            .await?;

        self.sync_repo
            .mark_synced("note", note_id, "", &at_uri.to_string())
            .await?;

        let metadata = self.sync_repo.get_sync_metadata("note", note_id).await?;

        // Complete log entry
        self.sync_repo
            .complete_log_entry(&log_id, "success", metadata.pds_cid.as_deref(), None)
            .await?;

        Ok(SyncResult {
            entity_type: "note".to_string(),
            entity_id: note_id.to_string(),
            pds_uri: Some(at_uri.to_string()),
            pds_cid: metadata.pds_cid,
            new_version: metadata.version,
            status: SyncStatus::Synced,
        })
    }

    /// Pull a record from the user's PDS.
    pub async fn pull_record(
        &self, entity_type: &str, at_uri: &str, user_ctx: &UserContext,
    ) -> Result<RemoteRecord, SyncError> {
        let parsed = malfestio_core::at_uri::AtUri::parse(at_uri)
            .map_err(|e| SyncError::InvalidArgument(format!("Invalid AT-URI: {}", e)))?;

        let log_id = self
            .sync_repo
            .log_operation(LogOperationParams {
                owner_did: &user_ctx.did,
                entity_type,
                entity_id: at_uri,
                operation: "pull",
                status: "pending",
                pds_cid: None,
                error_message: None,
            })
            .await?;

        let pds_client = self.get_pds_client(user_ctx).await?;
        let result: GetRecordResult = pds_client
            .get_record(&parsed.authority, &parsed.collection, &parsed.rkey)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = ?e,
                    at_uri = %at_uri,
                    "Failed to pull record from PDS"
                );
                SyncError::PdsError(e)
            })?;

        self.sync_repo
            .complete_log_entry(&log_id, "success", Some(&result.cid), None)
            .await?;

        // TODO: Offline queue - Store pulled record in IndexedDB for offline access

        Ok(RemoteRecord { uri: result.uri, cid: result.cid, value: result.value })
    }

    /// Check if there's a conflict between local and remote versions.
    pub async fn check_conflict(
        &self, entity_type: &str, entity_id: &str, remote_cid: &str,
    ) -> Result<bool, SyncError> {
        let metadata = self.sync_repo.get_sync_metadata(entity_type, entity_id).await?;

        let has_conflict =
            metadata.status == SyncStatus::PendingPush && metadata.pds_cid.as_deref() != Some(remote_cid);

        if has_conflict {
            self.sync_repo.mark_conflict(entity_type, entity_id).await?;
        }

        Ok(has_conflict)
    }

    /// Get sync status for a user.
    pub async fn get_sync_status(&self, user_ctx: &UserContext) -> Result<SyncStatusSummary, SyncError> {
        let pending = self.sync_repo.get_pending_items(&user_ctx.did).await?;
        let conflicts = self.sync_repo.get_conflicts(&user_ctx.did).await?;

        Ok(SyncStatusSummary {
            pending_count: pending.len(),
            conflict_count: conflicts.len(),
            pending_items: pending.into_iter().map(|p| (p.entity_type, p.entity_id)).collect(),
            conflicts: conflicts
                .into_iter()
                .map(|c| ConflictInfo {
                    entity_type: c.entity_type,
                    entity_id: c.entity_id,
                    local_version: c.version,
                    remote_version: None,
                    local_updated_at: None,
                    remote_updated_at: None,
                })
                .collect(),
        })
    }

    /// Resolve a conflict using the specified strategy.
    pub async fn resolve_conflict(
        &self, entity_type: &str, id: &str, strategy: ConflictStrategy, user_ctx: &UserContext,
    ) -> Result<SyncResult, SyncError> {
        let metadata = self.sync_repo.get_sync_metadata(entity_type, id).await?;

        if metadata.status != SyncStatus::Conflict {
            return Err(SyncError::InvalidArgument(format!(
                "Entity is not in conflict state: {}:{}",
                entity_type, id
            )));
        }

        match strategy {
            ConflictStrategy::LastWriteWins | ConflictStrategy::KeepLocal => match entity_type {
                "deck" => self.push_deck(id, user_ctx).await,
                "note" => self.push_note(id, user_ctx).await,
                _ => Err(SyncError::InvalidArgument(format!(
                    "Unknown entity type: {}",
                    entity_type
                ))),
            },
            ConflictStrategy::KeepRemote => {
                if let Some(pds_uri) = &metadata.pds_uri {
                    let remote = self.pull_record(entity_type, pds_uri, user_ctx).await?;

                    self.sync_repo
                        .mark_synced(entity_type, id, &remote.cid, &remote.uri)
                        .await?;

                    let new_metadata = self.sync_repo.get_sync_metadata(entity_type, id).await?;

                    Ok(SyncResult {
                        entity_type: entity_type.to_string(),
                        entity_id: id.to_string(),
                        pds_uri: Some(remote.uri),
                        pds_cid: Some(remote.cid),
                        new_version: new_metadata.version,
                        status: SyncStatus::Synced,
                    })
                } else {
                    Err(SyncError::InvalidArgument("No PDS URI for remote record".to_string()))
                }
            }
        }
    }

    async fn get_pds_client(&self, user_ctx: &UserContext) -> Result<PdsClient, SyncError> {
        if user_ctx.has_dpop
            && let Ok(stored_token) = self.oauth_repo.get_tokens(&user_ctx.did).await
            && let Some(dpop_keypair) = stored_token.dpop_keypair()
        {
            Ok(PdsClient::new_with_dpop(
                stored_token.pds_url.clone(),
                stored_token.access_token.clone(),
                dpop_keypair,
            ))
        } else {
            Ok(PdsClient::new_bearer(
                user_ctx.pds_url.clone(),
                user_ctx.access_token.clone(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_strategy_from_str() {
        assert_eq!(
            ConflictStrategy::from_str("last_write_wins"),
            Ok(ConflictStrategy::LastWriteWins)
        );
        assert_eq!(
            ConflictStrategy::from_str("keep_local"),
            Ok(ConflictStrategy::KeepLocal)
        );
        assert_eq!(
            ConflictStrategy::from_str("keep_remote"),
            Ok(ConflictStrategy::KeepRemote)
        );
        assert!(ConflictStrategy::from_str("unknown").is_err());
    }

    #[test]
    fn test_conflict_strategy_as_str() {
        assert_eq!(ConflictStrategy::LastWriteWins.as_str(), "last_write_wins");
        assert_eq!(ConflictStrategy::KeepLocal.as_str(), "keep_local");
        assert_eq!(ConflictStrategy::KeepRemote.as_str(), "keep_remote");
    }

    #[test]
    fn test_sync_error_display() {
        let err = SyncError::NotFound("deck:123".to_string());
        assert!(err.to_string().contains("Not found"));

        let err = SyncError::AuthRequired("missing token".to_string());
        assert!(err.to_string().contains("Authentication required"));

        let err = SyncError::InvalidArgument("bad type".to_string());
        assert!(err.to_string().contains("Invalid argument"));
    }

    #[test]
    fn test_sync_result_creation() {
        let result = SyncResult {
            entity_type: "deck".to_string(),
            entity_id: "123".to_string(),
            pds_uri: Some("at://did:plc:test/deck/tid".to_string()),
            pds_cid: Some("bafycid".to_string()),
            new_version: 2,
            status: SyncStatus::Synced,
        };

        assert_eq!(result.entity_type, "deck");
        assert_eq!(result.new_version, 2);
        assert_eq!(result.status, SyncStatus::Synced);
    }

    #[test]
    fn test_sync_status_summary() {
        let summary = SyncStatusSummary {
            pending_count: 3,
            conflict_count: 1,
            pending_items: vec![
                ("deck".to_string(), "1".to_string()),
                ("note".to_string(), "2".to_string()),
            ],
            conflicts: vec![ConflictInfo {
                entity_type: "deck".to_string(),
                entity_id: "3".to_string(),
                local_version: 5,
                remote_version: Some(6),
                local_updated_at: None,
                remote_updated_at: None,
            }],
        };

        assert_eq!(summary.pending_count, 3);
        assert_eq!(summary.conflict_count, 1);
        assert_eq!(summary.pending_items.len(), 2);
    }

    #[test]
    fn test_remote_record_creation() {
        let record = RemoteRecord {
            uri: "at://did:plc:test/deck/tid".to_string(),
            cid: "bafycid123".to_string(),
            value: serde_json::json!({"title": "Test"}),
        };

        assert_eq!(record.uri, "at://did:plc:test/deck/tid");
        assert!(record.value.get("title").is_some());
    }
}
