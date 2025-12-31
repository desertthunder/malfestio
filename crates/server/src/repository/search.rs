use crate::db::DbPool;
use malfestio_core::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub item_type: String,
    pub item_id: String,
    pub creator_did: String,
    pub data: serde_json::Value,
    pub rank: f32,
    pub source: String,
}

#[async_trait::async_trait]
pub trait SearchRepository: Send + Sync {
    async fn search(&self, query: &str, limit: i64, offset: i64, viewer_did: Option<&str>)
    -> Result<Vec<SearchResult>>;
    async fn get_top_tags(&self, limit: i64) -> Result<Vec<(String, i64)>>;
}

pub struct DbSearchRepository {
    pool: DbPool,
}

impl DbSearchRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SearchRepository for DbSearchRepository {
    async fn search(
        &self, query: &str, limit: i64, offset: i64, viewer_did: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| malfestio_core::Error::Database(e.to_string()))?;

        // TODO: implement shared-with logic.
        let sql = "
            SELECT
                item_type,
                item_id,
                creator_did,
                data,
                ts_rank(tsv_content, websearch_to_tsquery('english', $1)) as rank,
                source
            FROM search_items
            WHERE tsv_content @@ websearch_to_tsquery('english', $1)
            AND (
                visibility->>'type' = 'Public'
                OR (creator_did = $4)
            )
            ORDER BY rank DESC
            LIMIT $2 OFFSET $3
        ";

        let rows = client
            .query(sql, &[&query, &limit, &offset, &viewer_did])
            .await
            .map_err(|e| malfestio_core::Error::Database(e.to_string()))?;

        let results = rows
            .iter()
            .map(|row| SearchResult {
                item_type: row.get("item_type"),
                item_id: row.get("item_id"),
                creator_did: row.get("creator_did"),
                data: row.get("data"),
                rank: row.get("rank"),
                source: row.get("source"),
            })
            .collect();

        Ok(results)
    }

    async fn get_top_tags(&self, limit: i64) -> Result<Vec<(String, i64)>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| malfestio_core::Error::Database(e.to_string()))?;

        let sql = "
            SELECT tag, count(*) as count
            FROM (
                SELECT unnest(tags) as tag FROM decks WHERE visibility->>'type' = 'Public'
                UNION ALL
                SELECT unnest(tags) as tag FROM notes WHERE visibility->>'type' = 'Public'
            ) as all_tags
            GROUP BY tag
            ORDER BY count DESC
            LIMIT $1
         ";

        let rows = client
            .query(sql, &[&limit])
            .await
            .map_err(|e| malfestio_core::Error::Database(e.to_string()))?;

        let results = rows.iter().map(|row| (row.get("tag"), row.get("count"))).collect();

        Ok(results)
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone)]
    pub struct MockSearchRepository {
        pub search_results: Arc<Mutex<Vec<SearchResult>>>,
    }

    impl MockSearchRepository {
        pub fn new() -> Self {
            Self { search_results: Arc::new(Mutex::new(vec![])) }
        }

        pub async fn add_result(&self, result: SearchResult) {
            let mut results = self.search_results.lock().await;
            results.push(result);
        }
    }

    impl Default for MockSearchRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait::async_trait]
    impl SearchRepository for MockSearchRepository {
        async fn search(
            &self, query: &str, limit: i64, offset: i64, viewer_did: Option<&str>,
        ) -> Result<Vec<SearchResult>> {
            let results = self.search_results.lock().await;

            let filtered: Vec<SearchResult> = results
                .iter()
                .filter(|r| {
                    let matches_query = r.item_id.to_lowercase().contains(&query.to_lowercase());

                    let is_public = r
                        .data
                        .get("visibility")
                        .and_then(|v| v.get("type"))
                        .and_then(|t| t.as_str())
                        == Some("Public");

                    let matches_auth = viewer_did.map_or(is_public, |did| r.creator_did == did || is_public);

                    matches_query && matches_auth
                })
                .skip(offset as usize)
                .take(limit as usize)
                .cloned()
                .collect();

            Ok(filtered)
        }

        async fn get_top_tags(&self, _limit: i64) -> Result<Vec<(String, i64)>> {
            Ok(vec![])
        }
    }
}
