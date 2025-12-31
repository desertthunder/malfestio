//! Firehose consumption via AT Protocol Jetstream.
//!
//! Provides WebSocket subscription to Jetstream for indexing public records.
//! Filters for `app.malfestio.*` collections and indexes them locally.

use crate::db::DbPool;
use async_trait::async_trait;
use atproto_jetstream::{Consumer, ConsumerTaskConfig, EventHandler, JetstreamEvent};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

/// Default Jetstream endpoint (Bluesky's public instance).
pub const DEFAULT_JETSTREAM_URL: &str = "wss://jetstream2.us-west.bsky.network/subscribe";

/// Collections we're interested in indexing.
pub const MALFESTIO_COLLECTIONS: &[&str] = &["app.malfestio.deck", "app.malfestio.card", "app.malfestio.note"];

/// Deck record structure matching the Lexicon schema.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckRecord {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub card_refs: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    pub created_at: String,
}

/// Card record structure matching the Lexicon schema.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardRecord {
    pub deck_ref: String,
    pub front: String,
    pub back: String,
    #[serde(default)]
    pub card_type: Option<String>,
    #[serde(default)]
    pub hints: Vec<String>,
    pub created_at: String,
}

/// Note record structure matching the Lexicon schema.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteRecord {
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub visibility: Option<String>,
    pub created_at: String,
}

/// Parse a datetime string from record into chrono DateTime.
fn parse_record_datetime(dt_str: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(dt_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

/// Event handler for Malfestio records from Jetstream.
pub struct MalfestioEventHandler {
    pool: DbPool,
    handler_id: String,
}

impl MalfestioEventHandler {
    /// Create a new event handler with database connection.
    pub fn new(pool: DbPool) -> Self {
        Self { pool, handler_id: "malfestio-indexer".to_string() }
    }

    /// Index a deck record with full content.
    async fn index_deck(
        &self, did: &str, rkey: &str, rev: &str, record: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let at_uri = format!("at://{}/app.malfestio.deck/{}", did, rkey);
        let deck: DeckRecord = serde_json::from_value(record.clone())?;
        let created_at = parse_record_datetime(&deck.created_at);

        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO indexed_decks (at_uri, did, rkey, title, description, tags, card_refs, source_refs, license, record_created_at, indexed_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
                 ON CONFLICT (at_uri) DO UPDATE SET
                    title = $4, description = $5, tags = $6, card_refs = $7, source_refs = $8,
                    license = $9, record_created_at = $10, indexed_at = NOW(), deleted_at = NULL",
                &[
                    &at_uri,
                    &did,
                    &rkey,
                    &deck.title,
                    &deck.description,
                    &deck.tags,
                    &deck.card_refs,
                    &deck.source_refs,
                    &deck.license,
                    &created_at,
                ],
            )
            .await?;

        self.update_repo_state(did, rev).await?;

        tracing::debug!("Indexed deck: {}", at_uri);
        Ok(())
    }

    /// Index a card record with full content.
    async fn index_card(
        &self, did: &str, rkey: &str, rev: &str, record: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let at_uri = format!("at://{}/app.malfestio.card/{}", did, rkey);
        let card: CardRecord = serde_json::from_value(record.clone())?;
        let created_at = parse_record_datetime(&card.created_at);
        let card_type = card.card_type.unwrap_or_else(|| "basic".to_string());

        let client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO indexed_cards (at_uri, did, rkey, deck_ref, front, back, card_type, hints, record_created_at, indexed_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
                 ON CONFLICT (at_uri) DO UPDATE SET
                    deck_ref = $4, front = $5, back = $6, card_type = $7, hints = $8,
                    record_created_at = $9, indexed_at = NOW(), deleted_at = NULL",
                &[
                    &at_uri,
                    &did,
                    &rkey,
                    &card.deck_ref,
                    &card.front,
                    &card.back,
                    &card_type,
                    &card.hints,
                    &created_at,
                ],
            )
            .await?;

        self.update_repo_state(did, rev).await?;

        tracing::debug!("Indexed card: {}", at_uri);
        Ok(())
    }

    /// Index a note record with full content.
    async fn index_note(
        &self, did: &str, rkey: &str, rev: &str, record: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let at_uri = format!("at://{}/app.malfestio.note/{}", did, rkey);
        let note: NoteRecord = serde_json::from_value(record.clone())?;
        let created_at = parse_record_datetime(&note.created_at);
        let visibility = note.visibility.unwrap_or_else(|| "public".to_string());

        let client = self.pool.get().await?;

        client
            .execute(
                "INSERT INTO indexed_notes (at_uri, did, rkey, title, body, tags, visibility, record_created_at, indexed_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
                 ON CONFLICT (at_uri) DO UPDATE SET
                    title = $4, body = $5, tags = $6, visibility = $7,
                    record_created_at = $8, indexed_at = NOW(), deleted_at = NULL",
                &[
                    &at_uri,
                    &did,
                    &rkey,
                    &note.title,
                    &note.body,
                    &note.tags,
                    &visibility,
                    &created_at,
                ],
            )
            .await?;

        self.update_repo_state(did, rev).await?;

        tracing::debug!("Indexed note: {}", at_uri);
        Ok(())
    }

    /// Handle deletion of a record (soft delete by setting deleted_at).
    async fn handle_delete(
        &self, did: &str, collection: &str, rkey: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let at_uri = format!("at://{}/{}/{}", did, collection, rkey);
        let client = self.pool.get().await?;

        let table = match collection {
            "app.malfestio.deck" => "indexed_decks",
            "app.malfestio.card" => "indexed_cards",
            "app.malfestio.note" => "indexed_notes",
            _ => return Ok(()),
        };

        let query = format!(
            "UPDATE {} SET deleted_at = NOW() WHERE at_uri = $1 AND deleted_at IS NULL",
            table
        );
        client.execute(&query, &[&at_uri]).await?;

        tracing::info!("Soft-deleted record: {}", at_uri);
        Ok(())
    }

    /// Update the repo sync state with the latest processed revision.
    async fn update_repo_state(&self, did: &str, rev: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO repo_sync_state (did, latest_rev, indexed_at)
                 VALUES ($1, $2, NOW())
                 ON CONFLICT (did) DO UPDATE SET latest_rev = $2, indexed_at = NOW()
                 WHERE repo_sync_state.latest_rev < $2 OR repo_sync_state.latest_rev IS NULL",
                &[&did, &rev],
            )
            .await?;
        Ok(())
    }

    /// Update cursor position in database for reconnection.
    pub async fn save_cursor(&self, cursor_us: i64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.pool.get().await?;
        client
            .execute(
                "INSERT INTO firehose_cursors (endpoint, cursor_us)
                 VALUES ($1, $2)
                 ON CONFLICT (endpoint) DO UPDATE SET cursor_us = $2, updated_at = NOW()",
                &[&DEFAULT_JETSTREAM_URL, &cursor_us],
            )
            .await?;
        Ok(())
    }

    /// Get saved cursor position for reconnection.
    pub async fn get_cursor(&self) -> Option<i64> {
        let client = self.pool.get().await.ok()?;
        let row = client
            .query_opt(
                "SELECT cursor_us FROM firehose_cursors WHERE endpoint = $1",
                &[&DEFAULT_JETSTREAM_URL],
            )
            .await
            .ok()??;
        row.get("cursor_us")
    }
}

#[async_trait]
impl EventHandler for MalfestioEventHandler {
    fn handler_id(&self) -> String {
        self.handler_id.clone()
    }

    async fn handle_event(&self, event: JetstreamEvent) -> Result<(), anyhow::Error> {
        match event {
            JetstreamEvent::Commit { did, time_us, commit, .. } => {
                let collection = &commit.collection;
                if !MALFESTIO_COLLECTIONS.iter().any(|c| collection == *c) {
                    return Ok(());
                }

                let rkey = &commit.rkey;
                let rev = &commit.rev;
                let operation = &commit.operation;

                tracing::info!(
                    "Received {} {} event: did={}, rkey={}",
                    collection,
                    operation,
                    did,
                    rkey
                );

                match operation.as_str() {
                    "create" | "update" => {
                        let result = match collection.as_str() {
                            "app.malfestio.deck" => self.index_deck(&did, rkey, rev, &commit.record).await,
                            "app.malfestio.card" => self.index_card(&did, rkey, rev, &commit.record).await,
                            "app.malfestio.note" => self.index_note(&did, rkey, rev, &commit.record).await,
                            _ => Ok(()),
                        };

                        if let Err(e) = result {
                            tracing::warn!("Failed to index record: {}", e);
                        }
                    }
                    "delete" => {
                        if let Err(e) = self.handle_delete(&did, collection, rkey).await {
                            tracing::warn!("Failed to handle delete: {}", e);
                        }
                    }
                    _ => tracing::debug!("Unknown operation type: {}", operation),
                }

                if let Err(e) = self.save_cursor(time_us as i64).await {
                    tracing::warn!("Failed to save cursor: {}", e);
                }
            }
            JetstreamEvent::Delete { did, commit, .. } => {
                let collection = &commit.collection;

                if MALFESTIO_COLLECTIONS.iter().any(|c| collection == *c) {
                    let rkey = &commit.rkey;
                    tracing::info!(
                        "Received delete event: did={}, collection={}, rkey={}",
                        did,
                        collection,
                        rkey
                    );

                    if let Err(e) = self.handle_delete(&did, collection, rkey).await {
                        tracing::warn!("Failed to handle delete: {}", e);
                    }
                }
            }
            JetstreamEvent::Identity { .. } | JetstreamEvent::Account { .. } => (),
        }
        Ok(())
    }
}

/// Configuration for the firehose consumer.
pub struct FirehoseConfig {
    /// Jetstream WebSocket URL
    pub jetstream_url: String,
    /// Collections to filter for
    pub collections: Vec<String>,
    /// Enable zstd compression
    pub compress: bool,
}

impl Default for FirehoseConfig {
    fn default() -> Self {
        Self {
            jetstream_url: DEFAULT_JETSTREAM_URL.to_string(),
            collections: MALFESTIO_COLLECTIONS.iter().map(|s| s.to_string()).collect(),
            compress: true,
        }
    }
}

/// Start the firehose consumer as a background task.
///
/// Returns a `CancellationToken` that can be used to stop the consumer.
pub async fn start_firehose(pool: DbPool, config: FirehoseConfig) -> CancellationToken {
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    let handler = MalfestioEventHandler::new(pool);

    let task_config = ConsumerTaskConfig {
        user_agent: "malfestio-indexer/0.1.0".to_string(),
        compression: config.compress,
        zstd_dictionary_location: String::new(),
        jetstream_hostname: config.jetstream_url.replace("wss://", "").replace("/subscribe", ""),
        collections: config.collections,
        dids: vec![],
        max_message_size_bytes: None,
        cursor: None,
        require_hello: false,
    };

    tokio::spawn(async move {
        tracing::info!("Starting Jetstream firehose consumer...");

        if let Some(cursor) = handler.get_cursor().await {
            tracing::info!("Resuming from cursor: {}", cursor);
        }

        let consumer = Consumer::new(task_config);
        if let Err(e) = consumer.register_handler(std::sync::Arc::new(handler)).await {
            tracing::error!("Failed to register handler: {}", e);
            return;
        }

        if let Err(e) = consumer.run_background(cancel_clone).await {
            tracing::error!("Firehose consumer error: {}", e);
        }

        tracing::info!("Firehose consumer stopped");
    });

    cancel
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::*;

    #[test]
    fn test_default_firehose_config() {
        let config = FirehoseConfig::default();
        assert_eq!(config.jetstream_url, DEFAULT_JETSTREAM_URL);
        assert_eq!(config.collections.len(), 3);
        assert!(config.compress);
    }

    #[test]
    fn test_malfestio_collections() {
        assert!(MALFESTIO_COLLECTIONS.contains(&"app.malfestio.deck"));
        assert!(MALFESTIO_COLLECTIONS.contains(&"app.malfestio.card"));
        assert!(MALFESTIO_COLLECTIONS.contains(&"app.malfestio.note"));
    }

    #[test]
    fn test_parse_deck_record() {
        let json = serde_json::json!({
            "title": "Test Deck",
            "description": "A test deck",
            "tags": ["rust", "learning"],
            "cardRefs": ["at://did:plc:abc/app.malfestio.card/123"],
            "sourceRefs": [],
            "license": "CC-BY-4.0",
            "createdAt": "2024-01-01T00:00:00Z"
        });

        let deck: DeckRecord = serde_json::from_value(json).unwrap();
        assert_eq!(deck.title, "Test Deck");
        assert_eq!(deck.description, Some("A test deck".to_string()));
        assert_eq!(deck.tags, vec!["rust", "learning"]);
        assert_eq!(deck.card_refs.len(), 1);
        assert_eq!(deck.license, Some("CC-BY-4.0".to_string()));
    }

    #[test]
    fn test_parse_card_record() {
        let json = serde_json::json!({
            "deckRef": "at://did:plc:abc/app.malfestio.deck/123",
            "front": "What is Rust?",
            "back": "A systems programming language",
            "cardType": "basic",
            "hints": ["Think about memory safety"],
            "createdAt": "2024-01-01T00:00:00Z"
        });

        let card: CardRecord = serde_json::from_value(json).unwrap();
        assert_eq!(card.deck_ref, "at://did:plc:abc/app.malfestio.deck/123");
        assert_eq!(card.front, "What is Rust?");
        assert_eq!(card.back, "A systems programming language");
        assert_eq!(card.card_type, Some("basic".to_string()));
        assert_eq!(card.hints, vec!["Think about memory safety"]);
    }

    #[test]
    fn test_parse_note_record() {
        let json = serde_json::json!({
            "title": "Study Notes",
            "body": "# Chapter 1\n\nSome content here.",
            "tags": ["chapter1"],
            "visibility": "public",
            "createdAt": "2024-01-01T00:00:00Z"
        });

        let note: NoteRecord = serde_json::from_value(json).unwrap();
        assert_eq!(note.title, "Study Notes");
        assert_eq!(note.body, "# Chapter 1\n\nSome content here.");
        assert_eq!(note.tags, vec!["chapter1"]);
        assert_eq!(note.visibility, Some("public".to_string()));
    }

    #[test]
    fn test_parse_record_datetime() {
        let dt = parse_record_datetime("2024-01-15T10:30:00Z");
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_parse_record_datetime_invalid() {
        let dt = parse_record_datetime("invalid");
        assert!(dt.year() >= 2024);
    }
}
