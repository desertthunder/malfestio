use async_trait::async_trait;
use malfestio_core::model::{Card, CardType};

#[derive(Debug)]
pub enum CardRepoError {
    DatabaseError(String),
    NotFound(String),
    InvalidArgument(String),
}

/// Parameters for creating a new card
#[derive(Debug)]
pub struct CreateCardParams {
    pub owner_did: String,
    pub deck_id: String,
    pub front: String,
    pub back: String,
    pub media_url: Option<String>,
    pub card_type: CardType,
    pub hints: Vec<String>,
}

#[async_trait]
pub trait CardRepository: Send + Sync {
    async fn create(&self, params: CreateCardParams) -> Result<Card, CardRepoError>;

    async fn list_by_deck(&self, deck_id: &str) -> Result<Vec<Card>, CardRepoError>;

    async fn verify_deck_ownership(&self, deck_id: &str, owner_did: &str) -> Result<bool, CardRepoError>;

    async fn update_at_uri(&self, card_id: &str, at_uri: &str) -> Result<(), CardRepoError>;
}

pub struct DbCardRepository {
    pool: crate::db::DbPool,
}

impl DbCardRepository {
    pub fn new(pool: crate::db::DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CardRepository for DbCardRepository {
    async fn create(&self, params: CreateCardParams) -> Result<Card, CardRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let deck_uuid = uuid::Uuid::parse_str(&params.deck_id)
            .map_err(|_| CardRepoError::InvalidArgument("Invalid deck ID".to_string()))?;

        let deck_row = client
            .query_opt("SELECT owner_did FROM decks WHERE id = $1", &[&deck_uuid])
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to query deck: {}", e)))?
            .ok_or_else(|| CardRepoError::NotFound("Deck not found".to_string()))?;

        let deck_owner: String = deck_row.get("owner_did");
        if deck_owner != params.owner_did {
            return Err(CardRepoError::InvalidArgument(
                "Only deck owner can add cards".to_string(),
            ));
        }

        let card_id = uuid::Uuid::new_v4();
        client
            .execute(
                "INSERT INTO cards (id, owner_did, deck_id, front, back, media_url, hints)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
                &[
                    &card_id,
                    &params.owner_did,
                    &deck_uuid,
                    &params.front,
                    &params.back,
                    &params.media_url,
                    &params.hints,
                ],
            )
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to insert card: {}", e)))?;

        Ok(Card {
            id: card_id.to_string(),
            owner_did: params.owner_did,
            deck_id: params.deck_id,
            front: params.front,
            back: params.back,
            media_url: params.media_url,
            card_type: params.card_type,
            hints: params.hints,
        })
    }

    async fn list_by_deck(&self, deck_id: &str) -> Result<Vec<Card>, CardRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let deck_uuid = uuid::Uuid::parse_str(deck_id)
            .map_err(|_| CardRepoError::InvalidArgument("Invalid deck ID".to_string()))?;

        let deck_exists = client
            .query_opt("SELECT id FROM decks WHERE id = $1", &[&deck_uuid])
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to query deck: {}", e)))?
            .is_some();

        if !deck_exists {
            return Err(CardRepoError::NotFound("Deck not found".to_string()));
        }

        let rows = client
            .query(
                "SELECT id, owner_did, deck_id, front, back, media_url, hints
                 FROM cards
                 WHERE deck_id = $1
                 ORDER BY created_at ASC",
                &[&deck_uuid],
            )
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to query cards: {}", e)))?;

        let mut cards = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.get("id");
            let card_deck_id: uuid::Uuid = row.get("deck_id");

            cards.push(Card {
                id: id.to_string(),
                owner_did: row.get("owner_did"),
                deck_id: card_deck_id.to_string(),
                front: row.get("front"),
                back: row.get("back"),
                media_url: row.get("media_url"),
                card_type: CardType::default(),
                hints: row.get("hints"),
            });
        }

        Ok(cards)
    }

    async fn verify_deck_ownership(&self, deck_id: &str, owner_did: &str) -> Result<bool, CardRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let deck_uuid = uuid::Uuid::parse_str(deck_id)
            .map_err(|_| CardRepoError::InvalidArgument("Invalid deck ID".to_string()))?;

        let row = client
            .query_opt("SELECT owner_did FROM decks WHERE id = $1", &[&deck_uuid])
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to query deck: {}", e)))?;

        match row {
            Some(row) => {
                let deck_owner: String = row.get("owner_did");
                Ok(deck_owner == owner_did)
            }
            None => Ok(false),
        }
    }

    async fn update_at_uri(&self, card_id: &str, at_uri: &str) -> Result<(), CardRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let card_uuid = uuid::Uuid::parse_str(card_id)
            .map_err(|_| CardRepoError::InvalidArgument("Invalid card ID".to_string()))?;

        client
            .execute("UPDATE cards SET at_uri = $1 WHERE id = $2", &[&at_uri, &card_uuid])
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to update card AT-URI: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct MockCardRepository {
        pub cards: Arc<Mutex<Vec<Card>>>,
        pub at_uris: Arc<Mutex<std::collections::HashMap<String, String>>>,
        pub should_fail: Arc<Mutex<bool>>,
    }

    impl MockCardRepository {
        pub fn new() -> Self {
            Self {
                cards: Arc::new(Mutex::new(Vec::new())),
                at_uris: Arc::new(Mutex::new(std::collections::HashMap::new())),
                should_fail: Arc::new(Mutex::new(false)),
            }
        }

        pub fn with_cards(cards: Vec<Card>) -> Self {
            Self {
                cards: Arc::new(Mutex::new(cards)),
                at_uris: Arc::new(Mutex::new(std::collections::HashMap::new())),
                should_fail: Arc::new(Mutex::new(false)),
            }
        }

        pub fn set_should_fail(&self, should_fail: bool) {
            *self.should_fail.lock().unwrap() = should_fail;
        }
    }

    impl Default for MockCardRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl CardRepository for MockCardRepository {
        async fn create(&self, params: CreateCardParams) -> Result<Card, CardRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(CardRepoError::DatabaseError("Mock failure".to_string()));
            }

            let card = Card {
                id: uuid::Uuid::new_v4().to_string(),
                owner_did: params.owner_did,
                deck_id: params.deck_id,
                front: params.front,
                back: params.back,
                media_url: params.media_url,
                card_type: params.card_type,
                hints: params.hints,
            };

            self.cards.lock().unwrap().push(card.clone());
            Ok(card)
        }

        async fn list_by_deck(&self, deck_id: &str) -> Result<Vec<Card>, CardRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(CardRepoError::DatabaseError("Mock failure".to_string()));
            }

            let cards = self.cards.lock().unwrap();
            Ok(cards.iter().filter(|c| c.deck_id == deck_id).cloned().collect())
        }

        async fn verify_deck_ownership(&self, _deck_id: &str, _owner_did: &str) -> Result<bool, CardRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(CardRepoError::DatabaseError("Mock failure".to_string()));
            }
            Ok(true)
        }

        async fn update_at_uri(&self, card_id: &str, at_uri: &str) -> Result<(), CardRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(CardRepoError::DatabaseError("Mock failure".to_string()));
            }

            let cards = self.cards.lock().unwrap();
            if cards.iter().any(|c| c.id == card_id) {
                let mut at_uris = self.at_uris.lock().unwrap();
                at_uris.insert(card_id.to_string(), at_uri.to_string());
                Ok(())
            } else {
                Err(CardRepoError::NotFound("Card not found".to_string()))
            }
        }
    }
}
