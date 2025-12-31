use async_trait::async_trait;
use malfestio_core::model::{Deck, Visibility};
use uuid::Uuid;

#[derive(Debug)]
pub enum DeckRepoError {
    DatabaseError(String),
    NotFound(String),
    AccessDenied(String),
    InvalidArgument(String),
}

#[derive(Debug)]
pub struct CreateDeckParams {
    pub owner_did: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub visibility: Visibility,
}

#[derive(Debug)]
pub struct UpdateDeckParams {
    pub deck_id: String,
    pub visibility: Option<Visibility>,
    pub published_at: Option<String>,
    pub at_uri: Option<String>,
}

#[async_trait]
pub trait DeckRepository: Send + Sync {
    async fn create(&self, params: CreateDeckParams) -> Result<Deck, DeckRepoError>;
    async fn get(&self, id: &str) -> Result<Deck, DeckRepoError>;
    async fn list_visible(&self, viewer_did: Option<&str>) -> Result<Vec<Deck>, DeckRepoError>;
    async fn update(&self, params: UpdateDeckParams) -> Result<Deck, DeckRepoError>;
    async fn fork(&self, original_deck_id: &str, user_did: &str) -> Result<Deck, DeckRepoError>;
    async fn get_decks_by_user(&self, owner_did: &str) -> Result<Vec<Deck>, DeckRepoError>;
    async fn get_remote_deck(&self, at_uri: &str) -> Result<(Deck, Vec<malfestio_core::model::Card>), DeckRepoError>;
}

pub struct DbDeckRepository {
    pool: crate::db::DbPool,
}

impl DbDeckRepository {
    pub fn new(pool: crate::db::DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeckRepository for DbDeckRepository {
    async fn create(&self, params: CreateDeckParams) -> Result<Deck, DeckRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let deck_id = Uuid::new_v4();
        let visibility_json = serde_json::to_value(&params.visibility)
            .map_err(|e| DeckRepoError::InvalidArgument(format!("Failed to serialize visibility: {}", e)))?;

        client
            .execute(
                "INSERT INTO decks (id, owner_did, title, description, tags, visibility)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                &[
                    &deck_id,
                    &params.owner_did,
                    &params.title,
                    &params.description,
                    &params.tags,
                    &visibility_json,
                ],
            )
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to insert deck: {}", e)))?;

        Ok(Deck {
            id: deck_id.to_string(),
            owner_did: params.owner_did,
            title: params.title,
            description: params.description,
            tags: params.tags,
            visibility: params.visibility,
            published_at: None,
            fork_of: None,
        })
    }

    async fn get(&self, id: &str) -> Result<Deck, DeckRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let deck_uuid =
            Uuid::parse_str(id).map_err(|_| DeckRepoError::InvalidArgument("Invalid deck ID".to_string()))?;

        let row = client
            .query_opt(
                "SELECT id, owner_did, title, description, tags, visibility, published_at, fork_of
                 FROM decks WHERE id = $1",
                &[&deck_uuid],
            )
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to query deck: {}", e)))?
            .ok_or_else(|| DeckRepoError::NotFound("Deck not found".to_string()))?;

        let visibility_json: serde_json::Value = row.get("visibility");
        let visibility: Visibility = serde_json::from_value(visibility_json)
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to parse deck visibility: {}", e)))?;

        let fork_of: Option<Uuid> = row.get("fork_of");

        Ok(Deck {
            id: row.get::<_, Uuid>("id").to_string(),
            owner_did: row.get("owner_did"),
            title: row.get("title"),
            description: row.get("description"),
            tags: row.get("tags"),
            visibility,
            published_at: row
                .get::<_, Option<chrono::DateTime<chrono::Utc>>>("published_at")
                .map(|dt| dt.to_rfc3339()),
            fork_of: fork_of.map(|u| u.to_string()),
        })
    }

    async fn list_visible(&self, viewer_did: Option<&str>) -> Result<Vec<Deck>, DeckRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = if viewer_did.is_some() {
            "SELECT id, owner_did, title, description, tags, visibility, published_at, fork_of, created_at
             FROM decks
             WHERE owner_did = $1
                OR visibility->>'type' = 'Public'
                OR visibility->>'type' = 'Unlisted'
                OR (visibility->>'type' = 'SharedWith' AND visibility->'content' ? $1)
             ORDER BY created_at DESC"
        } else {
            "SELECT id, owner_did, title, description, tags, visibility, published_at, fork_of, created_at
             FROM decks
             WHERE visibility->>'type' IN ('Public', 'Unlisted')
             ORDER BY created_at DESC"
        };

        let rows = if let Some(did) = viewer_did {
            client.query(query, &[&did]).await
        } else {
            client.query(query, &[]).await
        }
        .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to retrieve decks: {}", e)))?;

        let mut decks = Vec::new();
        for row in rows {
            let visibility_json: serde_json::Value = row.get("visibility");
            let visibility: Visibility = serde_json::from_value(visibility_json).unwrap_or(Visibility::Private);
            let fork_of: Option<Uuid> = row.get("fork_of");

            decks.push(Deck {
                id: row.get::<_, Uuid>("id").to_string(),
                owner_did: row.get("owner_did"),
                title: row.get("title"),
                description: row.get("description"),
                tags: row.get("tags"),
                visibility,
                published_at: row
                    .get::<_, Option<chrono::DateTime<chrono::Utc>>>("published_at")
                    .map(|dt| dt.to_rfc3339()),
                fork_of: fork_of.map(|u| u.to_string()),
            });
        }
        Ok(decks)
    }

    async fn update(&self, params: UpdateDeckParams) -> Result<Deck, DeckRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let deck_uuid = Uuid::parse_str(&params.deck_id)
            .map_err(|_| DeckRepoError::InvalidArgument("Invalid deck ID".to_string()))?;

        // TODO: build the query dynamically.
        let current = self.get(&params.deck_id).await?;

        let new_visibility = params.visibility.unwrap_or(current.visibility);
        let vis_json = serde_json::to_value(&new_visibility).unwrap();

        let new_published_at = if let Some(ts) = params.published_at {
            Some(
                chrono::DateTime::parse_from_rfc3339(&ts)
                    .map_err(|_| DeckRepoError::InvalidArgument("Invalid timestamp".to_string()))?
                    .with_timezone(&chrono::Utc),
            )
        } else {
            None
        };

        if let Some(at_uri) = params.at_uri {
            client
                .execute(
                    "UPDATE decks SET visibility = $1, published_at = $2, at_uri = $3 WHERE id = $4",
                    &[&vis_json, &new_published_at, &at_uri, &deck_uuid],
                )
                .await
        } else {
            client
                .execute(
                    "UPDATE decks SET visibility = $1, published_at = $2 WHERE id = $3",
                    &[&vis_json, &new_published_at, &deck_uuid],
                )
                .await
        }
        .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to update deck: {}", e)))?;

        self.get(&params.deck_id).await
    }

    async fn fork(&self, original_deck_id: &str, user_did: &str) -> Result<Deck, DeckRepoError> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let original_uuid = Uuid::parse_str(original_deck_id)
            .map_err(|_| DeckRepoError::InvalidArgument("Invalid deck ID".to_string()))?;

        // TODO: execute multiple queries with a Transaction object that lives across boundaries or use a closure
        let tx = client
            .transaction()
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

        let original_deck_row = tx
            .query_opt(
                "SELECT title, description, tags FROM decks WHERE id = $1",
                &[&original_uuid],
            )
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to query deck: {}", e)))?
            .ok_or_else(|| DeckRepoError::NotFound("Original deck not found".to_string()))?;

        let new_deck_id = Uuid::new_v4();
        let title: String = original_deck_row.get("title");
        let description: String = original_deck_row.get("description");
        let tags: Vec<String> = original_deck_row.get("tags");
        let new_title = format!("Fork of {}", title);
        let visibility = Visibility::Private;
        let vis_json = serde_json::to_value(&visibility).unwrap();

        tx.execute(
            "INSERT INTO decks (id, owner_did, title, description, tags, visibility, fork_of)
              VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &new_deck_id,
                &user_did,
                &new_title,
                &description,
                &tags,
                &vis_json,
                &original_uuid,
            ],
        )
        .await
        .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to create deck: {}", e)))?;

        let rows = tx
            .query(
                "SELECT front, back, media_url, hints FROM cards WHERE deck_id = $1",
                &[&original_uuid],
            )
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to fetch cards: {}", e)))?;

        for row in rows {
            let card_id = Uuid::new_v4();
            let front: String = row.get("front");
            let back: String = row.get("back");
            let media_url: Option<String> = row.get("media_url");
            let hints: Vec<String> = row.get("hints"); // Added hints support here too

            tx.execute(
                "INSERT INTO cards (id, owner_did, deck_id, front, back, media_url, hints)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
                &[&card_id, &user_did, &new_deck_id, &front, &back, &media_url, &hints],
            )
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to copy card: {}", e)))?;
        }

        tx.commit()
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(Deck {
            id: new_deck_id.to_string(),
            owner_did: user_did.to_string(),
            title: new_title,
            description,
            tags,
            visibility,
            published_at: None,
            fork_of: Some(original_deck_id.to_string()),
        })
    }

    async fn get_decks_by_user(&self, owner_did: &str) -> Result<Vec<Deck>, DeckRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let rows = client
            .query(
                "SELECT id, owner_did, title, description, tags, visibility, published_at, fork_of
                 FROM decks
                 WHERE owner_did = $1",
                &[&owner_did],
            )
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to retrieve decks: {}", e)))?;

        let mut decks = Vec::new();
        for row in rows {
            let visibility_json: serde_json::Value = row.get("visibility");
            let visibility: Visibility = serde_json::from_value(visibility_json).unwrap_or(Visibility::Private);
            let fork_of: Option<Uuid> = row.get("fork_of");

            decks.push(Deck {
                id: row.get::<_, Uuid>("id").to_string(),
                owner_did: row.get("owner_did"),
                title: row.get("title"),
                description: row.get("description"),
                tags: row.get("tags"),
                visibility,
                published_at: row
                    .get::<_, Option<chrono::DateTime<chrono::Utc>>>("published_at")
                    .map(|dt| dt.to_rfc3339()),
                fork_of: fork_of.map(|u| u.to_string()),
            });
        }
        Ok(decks)
    }

    async fn get_remote_deck(&self, at_uri: &str) -> Result<(Deck, Vec<malfestio_core::model::Card>), DeckRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let deck_row = client
            .query_opt(
                "SELECT did, title, description, tags FROM indexed_decks WHERE at_uri = $1 AND deleted_at IS NULL",
                &[&at_uri],
            )
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to query remote deck: {}", e)))?
            .ok_or_else(|| DeckRepoError::NotFound("Remote deck not found".to_string()))?;

        let deck = Deck {
            id: at_uri.to_string(),
            owner_did: deck_row.get("did"),
            title: deck_row.get("title"),
            description: deck_row.get("description"),
            tags: deck_row.get("tags"),
            visibility: Visibility::Public,
            published_at: None,
            fork_of: None,
        };

        let card_rows = client
            .query(
                "SELECT at_uri, did, front, back, media_url, hints FROM indexed_cards WHERE deck_ref = $1 AND deleted_at IS NULL",
                &[&at_uri],
            )
            .await
            .map_err(|e| DeckRepoError::DatabaseError(format!("Failed to query remote cards: {}", e)))?;

        let mut cards = Vec::new();
        for row in card_rows {
            let hints: Vec<String> = row.get("hints");
            cards.push(malfestio_core::model::Card {
                id: row.get("at_uri"),
                owner_did: row.get("did"),
                deck_id: at_uri.to_string(),
                front: row.get("front"),
                back: row.get("back"),
                media_url: row.get("media_url"),
                hints,
                // TODO: support other card types
                card_type: malfestio_core::model::CardType::Basic,
            });
        }

        Ok((deck, cards))
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct MockDeckRepository {
        pub decks: Arc<Mutex<Vec<Deck>>>,
    }

    impl MockDeckRepository {
        pub fn new() -> Self {
            Self { decks: Arc::new(Mutex::new(Vec::new())) }
        }
    }

    impl Default for MockDeckRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl DeckRepository for MockDeckRepository {
        async fn create(&self, params: CreateDeckParams) -> Result<Deck, DeckRepoError> {
            let deck = Deck {
                id: Uuid::new_v4().to_string(),
                owner_did: params.owner_did,
                title: params.title,
                description: params.description,
                tags: params.tags,
                visibility: params.visibility,
                published_at: None,
                fork_of: None,
            };
            self.decks.lock().unwrap().push(deck.clone());
            Ok(deck)
        }

        async fn get(&self, id: &str) -> Result<Deck, DeckRepoError> {
            let decks = self.decks.lock().unwrap();
            decks
                .iter()
                .find(|d| d.id == id)
                .cloned()
                .ok_or_else(|| DeckRepoError::NotFound("Deck not found".to_string()))
        }

        async fn list_visible(&self, _viewer_did: Option<&str>) -> Result<Vec<Deck>, DeckRepoError> {
            let decks = self.decks.lock().unwrap();
            Ok(decks.clone())
        }

        async fn update(&self, params: UpdateDeckParams) -> Result<Deck, DeckRepoError> {
            let mut decks = self.decks.lock().unwrap();
            let deck = decks
                .iter_mut()
                .find(|d| d.id == params.deck_id)
                .ok_or_else(|| DeckRepoError::NotFound("Deck not found".to_string()))?;

            if let Some(v) = params.visibility {
                deck.visibility = v;
            }
            if let Some(p) = params.published_at {
                deck.published_at = Some(p);
            }
            Ok(deck.clone())
        }

        async fn fork(&self, original_deck_id: &str, user_did: &str) -> Result<Deck, DeckRepoError> {
            let mut decks = self.decks.lock().unwrap();
            let original = decks
                .iter()
                .find(|d| d.id == original_deck_id)
                .ok_or_else(|| DeckRepoError::NotFound("Deck not found".to_string()))?
                .clone();

            let deck = Deck {
                id: Uuid::new_v4().to_string(),
                owner_did: user_did.to_string(),
                title: format!("Fork of {}", original.title),
                description: original.description,
                tags: original.tags,
                visibility: Visibility::Private,
                published_at: None,
                fork_of: Some(original_deck_id.to_string()),
            };
            decks.push(deck.clone());
            decks.push(deck.clone());
            Ok(deck)
        }

        async fn get_decks_by_user(&self, owner_did: &str) -> Result<Vec<Deck>, DeckRepoError> {
            let decks = self.decks.lock().unwrap();
            let user_decks = decks.iter().filter(|d| d.owner_did == owner_did).cloned().collect();
            Ok(user_decks)
        }

        async fn get_remote_deck(
            &self, at_uri: &str,
        ) -> Result<(Deck, Vec<malfestio_core::model::Card>), DeckRepoError> {
            let decks = self.decks.lock().unwrap();
            let deck = decks
                .iter()
                .find(|d| d.id == at_uri)
                .cloned()
                .ok_or_else(|| DeckRepoError::NotFound("Deck not found".to_string()))?;
            Ok((deck, vec![]))
        }
    }
}
