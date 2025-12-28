use async_trait::async_trait;
use malfestio_core::model::Card;

#[derive(Debug)]
pub enum CardRepoError {
    DatabaseError(String),
    NotFound(String),
    InvalidArgument(String),
}

#[async_trait]
pub trait CardRepository: Send + Sync {
    async fn create(
        &self, owner_did: &str, deck_id: &str, front: &str, back: &str, media_url: Option<&str>,
    ) -> Result<Card, CardRepoError>;

    async fn list_by_deck(&self, deck_id: &str) -> Result<Vec<Card>, CardRepoError>;

    async fn verify_deck_ownership(&self, deck_id: &str, owner_did: &str) -> Result<bool, CardRepoError>;
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
    async fn create(
        &self, owner_did: &str, deck_id: &str, front: &str, back: &str, media_url: Option<&str>,
    ) -> Result<Card, CardRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let deck_uuid = uuid::Uuid::parse_str(deck_id)
            .map_err(|_| CardRepoError::InvalidArgument("Invalid deck ID".to_string()))?;

        // Verify deck exists and user owns it
        let deck_row = client
            .query_opt("SELECT owner_did FROM decks WHERE id = $1", &[&deck_uuid])
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to query deck: {}", e)))?
            .ok_or_else(|| CardRepoError::NotFound("Deck not found".to_string()))?;

        let deck_owner: String = deck_row.get("owner_did");
        if deck_owner != owner_did {
            return Err(CardRepoError::InvalidArgument(
                "Only deck owner can add cards".to_string(),
            ));
        }

        let card_id = uuid::Uuid::new_v4();
        client
            .execute(
                "INSERT INTO cards (id, owner_did, deck_id, front, back, media_url)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                &[&card_id, &owner_did, &deck_uuid, &front, &back, &media_url],
            )
            .await
            .map_err(|e| CardRepoError::DatabaseError(format!("Failed to insert card: {}", e)))?;

        Ok(Card {
            id: card_id.to_string(),
            owner_did: owner_did.to_string(),
            deck_id: deck_id.to_string(),
            front: front.to_string(),
            back: back.to_string(),
            media_url: media_url.map(String::from),
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

        // Verify deck exists
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
                "SELECT id, owner_did, deck_id, front, back, media_url
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
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct MockCardRepository {
        pub cards: Arc<Mutex<Vec<Card>>>,
        pub should_fail: Arc<Mutex<bool>>,
    }

    impl MockCardRepository {
        pub fn new() -> Self {
            Self { cards: Arc::new(Mutex::new(Vec::new())), should_fail: Arc::new(Mutex::new(false)) }
        }

        pub fn with_cards(cards: Vec<Card>) -> Self {
            Self { cards: Arc::new(Mutex::new(cards)), should_fail: Arc::new(Mutex::new(false)) }
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
        async fn create(
            &self, owner_did: &str, deck_id: &str, front: &str, back: &str, media_url: Option<&str>,
        ) -> Result<Card, CardRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(CardRepoError::DatabaseError("Mock failure".to_string()));
            }

            let card = Card {
                id: uuid::Uuid::new_v4().to_string(),
                owner_did: owner_did.to_string(),
                deck_id: deck_id.to_string(),
                front: front.to_string(),
                back: back.to_string(),
                media_url: media_url.map(String::from),
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
    }
}
