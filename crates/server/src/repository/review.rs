use async_trait::async_trait;
use chrono::{DateTime, Utc};
use malfestio_core::srs::{Grade, ReviewState, Sm2Config};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ReviewRepoError {
    DatabaseError(String),
    NotFound(String),
    InvalidArgument(String),
}

/// Card with review state for study sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewCard {
    pub review_id: String,
    pub card_id: String,
    pub deck_id: String,
    pub deck_title: String,
    pub front: String,
    pub back: String,
    pub media_url: Option<String>,
    pub hints: Vec<String>,
    pub due_at: DateTime<Utc>,
}

/// User study statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StudyStats {
    pub due_count: i64,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub reviewed_today: i64,
    pub total_reviews: i64,
}

#[async_trait]
pub trait ReviewRepository: Send + Sync {
    /// Get cards due for review, optionally filtered by deck
    async fn get_due_cards(
        &self, user_did: &str, deck_id: Option<&str>, limit: i64,
    ) -> Result<Vec<ReviewCard>, ReviewRepoError>;

    /// Submit a review grade for a card
    async fn submit_review(&self, user_did: &str, card_id: &str, grade: Grade) -> Result<ReviewState, ReviewRepoError>;

    /// Get study statistics for a user
    async fn get_stats(&self, user_did: &str) -> Result<StudyStats, ReviewRepoError>;
}

pub struct DbReviewRepository {
    pool: crate::db::DbPool,
    config: Sm2Config,
}

impl DbReviewRepository {
    pub fn new(pool: crate::db::DbPool) -> Self {
        Self { pool, config: Sm2Config::default() }
    }
}

#[async_trait]
impl ReviewRepository for DbReviewRepository {
    async fn get_due_cards(
        &self, user_did: &str, deck_id: Option<&str>, limit: i64,
    ) -> Result<Vec<ReviewCard>, ReviewRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| ReviewRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let now = Utc::now();

        let rows = if let Some(deck_id) = deck_id {
            let deck_uuid = uuid::Uuid::parse_str(deck_id)
                .map_err(|_| ReviewRepoError::InvalidArgument("Invalid deck ID".to_string()))?;

            client
                .query(
                    r#"
                    SELECT
                        cr.id as review_id,
                        c.id as card_id,
                        c.deck_id,
                        d.title as deck_title,
                        c.front,
                        c.back,
                        c.media_url,
                        cr.due_at
                    FROM cards c
                    JOIN decks d ON c.deck_id = d.id
                    LEFT JOIN card_reviews cr ON c.id = cr.card_id AND cr.user_did = $1
                    WHERE c.deck_id = $2
                      AND (cr.due_at IS NULL OR cr.due_at <= $3)
                    ORDER BY COALESCE(cr.due_at, '1970-01-01'::timestamptz) ASC
                    LIMIT $4
                    "#,
                    &[&user_did, &deck_uuid, &now, &limit],
                )
                .await
        } else {
            client
                .query(
                    r#"
                    SELECT
                        cr.id as review_id,
                        c.id as card_id,
                        c.deck_id,
                        d.title as deck_title,
                        c.front,
                        c.back,
                        c.media_url,
                        cr.due_at
                    FROM cards c
                    JOIN decks d ON c.deck_id = d.id
                    LEFT JOIN card_reviews cr ON c.id = cr.card_id AND cr.user_did = $1
                    WHERE d.owner_did = $1
                      AND (cr.due_at IS NULL OR cr.due_at <= $2)
                    ORDER BY COALESCE(cr.due_at, '1970-01-01'::timestamptz) ASC
                    LIMIT $3
                    "#,
                    &[&user_did, &now, &limit],
                )
                .await
        };

        let rows = rows.map_err(|e| ReviewRepoError::DatabaseError(format!("Failed to query cards: {}", e)))?;

        let mut cards = Vec::new();
        for row in rows {
            let review_id: Option<uuid::Uuid> = row.get("review_id");
            let card_id: uuid::Uuid = row.get("card_id");
            let deck_id: uuid::Uuid = row.get("deck_id");
            let due_at: Option<DateTime<Utc>> = row.get("due_at");

            cards.push(ReviewCard {
                review_id: review_id.map(|id| id.to_string()).unwrap_or_default(),
                card_id: card_id.to_string(),
                deck_id: deck_id.to_string(),
                deck_title: row.get("deck_title"),
                front: row.get("front"),
                back: row.get("back"),
                media_url: row.get("media_url"),
                // TODO: Load hints when stored in DB
                hints: vec![],
                due_at: due_at.unwrap_or_else(Utc::now),
            });
        }

        Ok(cards)
    }

    async fn submit_review(&self, user_did: &str, card_id: &str, grade: Grade) -> Result<ReviewState, ReviewRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| ReviewRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let card_uuid = uuid::Uuid::parse_str(card_id)
            .map_err(|_| ReviewRepoError::InvalidArgument("Invalid card ID".to_string()))?;

        let existing = client
            .query_opt(
                "SELECT id, ease_factor, interval_days, repetitions, due_at FROM card_reviews WHERE card_id = $1 AND user_did = $2",
                &[&card_uuid, &user_did],
            )
            .await
            .map_err(|e| ReviewRepoError::DatabaseError(format!("Failed to query review: {}", e)))?;

        let current_state = existing
            .map(|row| ReviewState {
                ease_factor: row.get::<_, f32>("ease_factor"),
                interval_days: row.get::<_, i32>("interval_days"),
                repetitions: row.get::<_, i32>("repetitions"),
                due_at: row.get("due_at"),
            })
            .unwrap_or_default();

        let new_state = current_state.schedule(grade, &self.config);
        let now = Utc::now();

        client
            .execute(
                r#"
                INSERT INTO card_reviews (id, card_id, user_did, ease_factor, interval_days, repetitions, due_at, last_reviewed_at, total_reviews)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1)
                ON CONFLICT (card_id, user_did) DO UPDATE SET
                    ease_factor = $4,
                    interval_days = $5,
                    repetitions = $6,
                    due_at = $7,
                    last_reviewed_at = $8,
                    total_reviews = card_reviews.total_reviews + 1
                "#,
                &[
                    &uuid::Uuid::new_v4(),
                    &card_uuid,
                    &user_did,
                    &new_state.ease_factor,
                    &new_state.interval_days,
                    &new_state.repetitions,
                    &new_state.due_at,
                    &now,
                ],
            )
            .await
            .map_err(|e| ReviewRepoError::DatabaseError(format!("Failed to update review: {}", e)))?;

        let today = now.date_naive();
        client
            .execute(
                r#"
                INSERT INTO user_study_stats (id, user_did, current_streak, longest_streak, last_study_date, total_cards_reviewed)
                VALUES ($1, $2, 1, 1, $3, 1)
                ON CONFLICT (user_did) DO UPDATE SET
                    current_streak = CASE
                        WHEN user_study_stats.last_study_date = $3 THEN user_study_stats.current_streak
                        WHEN user_study_stats.last_study_date = $3 - INTERVAL '1 day' THEN user_study_stats.current_streak + 1
                        ELSE 1
                    END,
                    longest_streak = GREATEST(user_study_stats.longest_streak,
                        CASE
                            WHEN user_study_stats.last_study_date = $3 THEN user_study_stats.current_streak
                            WHEN user_study_stats.last_study_date = $3 - INTERVAL '1 day' THEN user_study_stats.current_streak + 1
                            ELSE 1
                        END
                    ),
                    last_study_date = $3,
                    total_cards_reviewed = user_study_stats.total_cards_reviewed + 1
                "#,
                &[&uuid::Uuid::new_v4(), &user_did, &today],
            )
            .await
            .map_err(|e| ReviewRepoError::DatabaseError(format!("Failed to update stats: {}", e)))?;

        Ok(new_state)
    }

    async fn get_stats(&self, user_did: &str) -> Result<StudyStats, ReviewRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| ReviewRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let now = Utc::now();
        let today = now.date_naive();

        let due_row = client
            .query_one(
                r#"
                SELECT COUNT(*) as due_count FROM cards c
                JOIN decks d ON c.deck_id = d.id
                LEFT JOIN card_reviews cr ON c.id = cr.card_id AND cr.user_did = $1
                WHERE d.owner_did = $1
                  AND (cr.due_at IS NULL OR cr.due_at <= $2)
                "#,
                &[&user_did, &now],
            )
            .await
            .map_err(|e| ReviewRepoError::DatabaseError(format!("Failed to count due cards: {}", e)))?;

        let due_count: i64 = due_row.get("due_count");

        let reviewed_row = client
            .query_one(
                r#"
                SELECT COUNT(*) as reviewed_count FROM card_reviews
                WHERE user_did = $1 AND DATE(last_reviewed_at) = $2
                "#,
                &[&user_did, &today],
            )
            .await
            .map_err(|e| ReviewRepoError::DatabaseError(format!("Failed to count reviews: {}", e)))?;

        let reviewed_today: i64 = reviewed_row.get("reviewed_count");

        let stats_row = client
            .query_opt(
                "SELECT current_streak, longest_streak, total_cards_reviewed FROM user_study_stats WHERE user_did = $1",
                &[&user_did],
            )
            .await
            .map_err(|e| ReviewRepoError::DatabaseError(format!("Failed to get stats: {}", e)))?;

        let (current_streak, longest_streak, total_reviews) = stats_row
            .map(|row| {
                (
                    row.get::<_, i32>("current_streak"),
                    row.get::<_, i32>("longest_streak"),
                    row.get::<_, i32>("total_cards_reviewed") as i64,
                )
            })
            .unwrap_or((0, 0, 0));

        Ok(StudyStats { due_count, current_streak, longest_streak, reviewed_today, total_reviews })
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct MockReviewRepository {
        pub cards: Arc<Mutex<Vec<ReviewCard>>>,
        pub should_fail: Arc<Mutex<bool>>,
    }

    impl MockReviewRepository {
        pub fn new() -> Self {
            Self { cards: Arc::new(Mutex::new(Vec::new())), should_fail: Arc::new(Mutex::new(false)) }
        }

        pub fn with_cards(cards: Vec<ReviewCard>) -> Self {
            Self { cards: Arc::new(Mutex::new(cards)), should_fail: Arc::new(Mutex::new(false)) }
        }

        #[allow(dead_code)]
        pub fn set_should_fail(&self, should_fail: bool) {
            *self.should_fail.lock().unwrap() = should_fail;
        }
    }

    impl Default for MockReviewRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl ReviewRepository for MockReviewRepository {
        async fn get_due_cards(
            &self, _user_did: &str, deck_id: Option<&str>, limit: i64,
        ) -> Result<Vec<ReviewCard>, ReviewRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(ReviewRepoError::DatabaseError("Mock failure".to_string()));
            }

            let cards = self.cards.lock().unwrap();
            let filtered: Vec<_> = cards
                .iter()
                .filter(|c| deck_id.is_none_or(|id| c.deck_id == id))
                .take(limit as usize)
                .cloned()
                .collect();
            Ok(filtered)
        }

        async fn submit_review(
            &self, _user_did: &str, _card_id: &str, grade: Grade,
        ) -> Result<ReviewState, ReviewRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(ReviewRepoError::DatabaseError("Mock failure".to_string()));
            }

            let state = ReviewState::default();
            Ok(state.schedule(grade, &Sm2Config::default()))
        }

        async fn get_stats(&self, _user_did: &str) -> Result<StudyStats, ReviewRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(ReviewRepoError::DatabaseError("Mock failure".to_string()));
            }

            Ok(StudyStats {
                due_count: self.cards.lock().unwrap().len() as i64,
                current_streak: 5,
                longest_streak: 10,
                reviewed_today: 3,
                total_reviews: 100,
            })
        }
    }
}
