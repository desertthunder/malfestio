//! Firehose consumption via AT Protocol Jetstream.
//!
//! Provides WebSocket subscription to Jetstream for indexing public records.
//! Filters for `app.malfestio.*` collections and indexes them locally.

use crate::db::DbPool;
use async_trait::async_trait;
use atproto_jetstream::{Consumer, ConsumerTaskConfig, EventHandler, JetstreamEvent};
use tokio_util::sync::CancellationToken;

/// Default Jetstream endpoint (Bluesky's public instance).
pub const DEFAULT_JETSTREAM_URL: &str = "wss://jetstream2.us-west.bsky.network/subscribe";

/// Collections we're interested in indexing.
pub const MALFESTIO_COLLECTIONS: &[&str] = &["app.malfestio.deck", "app.malfestio.card", "app.malfestio.note"];

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

    /// Index a record into the database.
    async fn index_record(
        &self, did: &str, collection: &str, rkey: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let at_uri = format!("at://{}/{}/{}", did, collection, rkey);

        let client = self.pool.get().await?;

        // Upsert into indexed_records table
        client
            .execute(
                "INSERT INTO indexed_records (at_uri, did, collection, rkey, indexed_at)
                 VALUES ($1, $2, $3, $4, NOW())
                 ON CONFLICT (at_uri) DO UPDATE SET indexed_at = NOW()",
                &[&at_uri, &did, &collection, &rkey],
            )
            .await?;

        tracing::debug!("Indexed record: {}", at_uri);
        Ok(())
    }

    /// Update cursor position in database for reconnection.
    #[allow(dead_code)]
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
            JetstreamEvent::Commit { did, commit, .. } => {
                let collection = &commit.collection;

                // Only process our collections
                if !MALFESTIO_COLLECTIONS.iter().any(|c| collection == *c) {
                    return Ok(());
                }

                let rkey = &commit.rkey;

                tracing::info!("Received {} event: did={}, rkey={}", collection, did, rkey);

                // Index the record
                if let Err(e) = self.index_record(&did, collection, rkey).await {
                    tracing::warn!("Failed to index record: {}", e);
                }
            }
            JetstreamEvent::Identity { .. } | JetstreamEvent::Account { .. } | JetstreamEvent::Delete { .. } => {
                // Ignore identity, account, and delete events
            }
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

    // Build consumer config
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
}
