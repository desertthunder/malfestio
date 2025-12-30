use async_trait::async_trait;
use chrono::Utc;
use malfestio_core::error::Error;
use malfestio_core::model::{Comment, Deck, Visibility};

use crate::db;

#[async_trait]
pub trait SocialRepository: Send + Sync {
    async fn follow(&self, follower: &str, subject: &str) -> Result<(), Error>;
    async fn unfollow(&self, follower: &str, subject: &str) -> Result<(), Error>;
    async fn get_followers(&self, did: &str) -> Result<Vec<String>, Error>;
    async fn get_following(&self, did: &str) -> Result<Vec<String>, Error>;
    async fn add_comment(
        &self, deck_id: &str, author_did: &str, content: &str, parent_id: Option<&str>,
    ) -> Result<Comment, Error>;
    async fn get_comments(&self, deck_id: &str) -> Result<Vec<Comment>, Error>;
    async fn get_feed_follows(&self, user_did: &str) -> Result<Vec<Deck>, Error>;
    async fn get_feed_trending(&self) -> Result<Vec<Deck>, Error>;
}

pub struct DbSocialRepository {
    pool: db::DbPool,
}

impl DbSocialRepository {
    pub fn new(pool: db::DbPool) -> Self {
        Self { pool }
    }

    fn parse_deck_rows(rows: Vec<tokio_postgres::Row>) -> Vec<Deck> {
        let mut decks = Vec::new();
        for row in rows {
            let visibility_json: serde_json::Value = row.get("visibility");
            let visibility: Visibility = match serde_json::from_value(visibility_json) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Failed to deserialize visibility: {}", e);
                    continue;
                }
            };

            let id: uuid::Uuid = row.get("id");
            let fork_of: Option<uuid::Uuid> = row.get("fork_of");

            decks.push(Deck {
                id: id.to_string(),
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
        decks
    }
}

#[async_trait]
impl SocialRepository for DbSocialRepository {
    async fn follow(&self, follower: &str, subject: &str) -> Result<(), Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(format!("Failed to get connection: {}", e)))?;

        client
            .execute(
                "INSERT INTO follows (follower_did, subject_did) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                &[&follower, &subject],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to follow: {}", e)))?;

        Ok(())
    }

    async fn unfollow(&self, follower: &str, subject: &str) -> Result<(), Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(format!("Failed to get connection: {}", e)))?;

        client
            .execute(
                "DELETE FROM follows WHERE follower_did = $1 AND subject_did = $2",
                &[&follower, &subject],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to unfollow: {}", e)))?;

        Ok(())
    }

    async fn get_followers(&self, did: &str) -> Result<Vec<String>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(format!("Failed to get connection: {}", e)))?;

        let rows = client
            .query("SELECT follower_did FROM follows WHERE subject_did = $1", &[&did])
            .await
            .map_err(|e| Error::Database(format!("Failed to get followers: {}", e)))?;

        Ok(rows.iter().map(|row| row.get("follower_did")).collect())
    }

    async fn get_following(&self, did: &str) -> Result<Vec<String>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(format!("Failed to get connection: {}", e)))?;

        let rows = client
            .query("SELECT subject_did FROM follows WHERE follower_did = $1", &[&did])
            .await
            .map_err(|e| Error::Database(format!("Failed to get following: {}", e)))?;

        Ok(rows.iter().map(|row| row.get("subject_did")).collect())
    }

    async fn add_comment(
        &self, deck_id: &str, author_did: &str, content: &str, parent_id: Option<&str>,
    ) -> Result<Comment, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(format!("Failed to get connection: {}", e)))?;

        let deck_uuid = uuid::Uuid::parse_str(deck_id).map_err(|_| Error::Database("Invalid deck ID".to_string()))?;

        let parent_uuid = parent_id
            .map(uuid::Uuid::parse_str)
            .transpose()
            .map_err(|_| Error::Database("Invalid parent ID".to_string()))?;

        let comment_id = uuid::Uuid::new_v4();
        let now = Utc::now();

        client
            .execute(
                "INSERT INTO comments (id, deck_id, author_did, content, parent_id, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                &[&comment_id, &deck_uuid, &author_did, &content, &parent_uuid, &now],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to add comment: {}", e)))?;

        Ok(Comment {
            id: comment_id.to_string(),
            deck_id: deck_id.to_string(),
            author_did: author_did.to_string(),
            content: content.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
            created_at: now.to_rfc3339(),
        })
    }

    async fn get_comments(&self, deck_id: &str) -> Result<Vec<Comment>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(format!("Failed to get connection: {}", e)))?;

        let deck_uuid = uuid::Uuid::parse_str(deck_id).map_err(|_| Error::Database("Invalid deck ID".to_string()))?;

        let rows = client
            .query(
                "SELECT id, deck_id, author_did, content, parent_id, created_at
                 FROM comments
                 WHERE deck_id = $1
                 ORDER BY created_at ASC",
                &[&deck_uuid],
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to get comments: {}", e)))?;

        let mut comments = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.get("id");
            let row_deck_id: uuid::Uuid = row.get("deck_id");
            let parent_id: Option<uuid::Uuid> = row.get("parent_id");
            let created_at: chrono::DateTime<Utc> = row.get("created_at");

            comments.push(Comment {
                id: id.to_string(),
                deck_id: row_deck_id.to_string(),
                author_did: row.get("author_did"),
                content: row.get("content"),
                parent_id: parent_id.map(|u| u.to_string()),
                created_at: created_at.to_rfc3339(),
            });
        }

        Ok(comments)
    }

    async fn get_feed_follows(&self, user_did: &str) -> Result<Vec<Deck>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(format!("Failed to get connection: {}", e)))?;

        let query = "
            SELECT d.id, d.owner_did, d.title, d.description, d.tags, d.visibility, d.published_at, d.fork_of
            FROM decks d
            JOIN follows f ON d.owner_did = f.subject_did
            WHERE f.follower_did = $1
              AND d.published_at IS NOT NULL
              AND d.visibility->>'type' != 'Private'
            ORDER BY d.published_at DESC
            LIMIT 50
        ";

        let rows = client
            .query(query, &[&user_did])
            .await
            .map_err(|e| Error::Database(format!("Failed to get feed: {}", e)))?;

        Ok(Self::parse_deck_rows(rows))
    }

    async fn get_feed_trending(&self) -> Result<Vec<Deck>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Database(format!("Failed to get connection: {}", e)))?;

        let query = "
            SELECT id, owner_did, title, description, tags, visibility, published_at, fork_of
            FROM decks
            WHERE published_at IS NOT NULL
              AND visibility->>'type' = 'Public'
            ORDER BY published_at DESC
            LIMIT 50
        ";

        let rows = client
            .query(query, &[])
            .await
            .map_err(|e| Error::Database(format!("Failed to get trending: {}", e)))?;

        Ok(Self::parse_deck_rows(rows))
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    pub struct MockSocialRepository {
        /// (follower, subject)
        pub followers: Arc<Mutex<Vec<(String, String)>>>,
        pub comments: Arc<Mutex<Vec<Comment>>>,
    }

    impl MockSocialRepository {
        pub fn new() -> Self {
            Self { followers: Arc::new(Mutex::new(Vec::new())), comments: Arc::new(Mutex::new(Vec::new())) }
        }
    }

    impl Default for MockSocialRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl SocialRepository for MockSocialRepository {
        async fn follow(&self, follower: &str, subject: &str) -> Result<(), Error> {
            let mut followers = self.followers.lock().unwrap();
            if !followers.contains(&(follower.to_string(), subject.to_string())) {
                followers.push((follower.to_string(), subject.to_string()));
            }
            Ok(())
        }

        async fn unfollow(&self, follower: &str, subject: &str) -> Result<(), Error> {
            let mut followers = self.followers.lock().unwrap();
            followers.retain(|(f, s)| f != follower || s != subject);
            Ok(())
        }

        async fn get_followers(&self, did: &str) -> Result<Vec<String>, Error> {
            let followers = self.followers.lock().unwrap();
            Ok(followers
                .iter()
                .filter(|(_, s)| s == did)
                .map(|(f, _)| f.clone())
                .collect())
        }

        async fn get_following(&self, did: &str) -> Result<Vec<String>, Error> {
            let followers = self.followers.lock().unwrap();
            Ok(followers
                .iter()
                .filter(|(f, _)| f == did)
                .map(|(_, s)| s.clone())
                .collect())
        }

        async fn add_comment(
            &self, deck_id: &str, author_did: &str, content: &str, parent_id: Option<&str>,
        ) -> Result<Comment, Error> {
            let comment = Comment {
                id: uuid::Uuid::new_v4().to_string(),
                deck_id: deck_id.to_string(),
                author_did: author_did.to_string(),
                content: content.to_string(),
                parent_id: parent_id.map(|s| s.to_string()),
                created_at: Utc::now().to_rfc3339(),
            };
            self.comments.lock().unwrap().push(comment.clone());
            Ok(comment)
        }

        async fn get_comments(&self, deck_id: &str) -> Result<Vec<Comment>, Error> {
            let comments = self.comments.lock().unwrap();
            Ok(comments.iter().filter(|c| c.deck_id == deck_id).cloned().collect())
        }

        /// Mock empty or predefined
        async fn get_feed_follows(&self, _user_did: &str) -> Result<Vec<Deck>, Error> {
            Ok(vec![])
        }

        async fn get_feed_trending(&self) -> Result<Vec<Deck>, Error> {
            Ok(vec![])
        }
    }
}
