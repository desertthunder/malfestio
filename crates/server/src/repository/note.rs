use async_trait::async_trait;
use malfestio_core::model::{Note, Visibility};

#[derive(Debug)]
pub enum NoteRepoError {
    DatabaseError(String),
    NotFound(String),
    InvalidArgument(String),
    SerializationError(String),
}

#[async_trait]
pub trait NoteRepository: Send + Sync {
    async fn create(
        &self, owner_did: &str, title: &str, body: &str, tags: Vec<String>, visibility: Visibility,
    ) -> Result<Note, NoteRepoError>;
    async fn list(&self, viewer_did: Option<&str>) -> Result<Vec<Note>, NoteRepoError>;
    async fn get(&self, id: &str, viewer_did: Option<&str>) -> Result<Note, NoteRepoError>;
    async fn get_notes_by_user(&self, owner_did: &str) -> Result<Vec<Note>, NoteRepoError>;
}

pub struct DbNoteRepository {
    pool: crate::db::DbPool,
}

impl DbNoteRepository {
    pub fn new(pool: crate::db::DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NoteRepository for DbNoteRepository {
    async fn create(
        &self, owner_did: &str, title: &str, body: &str, tags: Vec<String>, visibility: Visibility,
    ) -> Result<Note, NoteRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| NoteRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let note_id = uuid::Uuid::new_v4();
        let visibility_json = serde_json::to_value(&visibility)
            .map_err(|e| NoteRepoError::SerializationError(format!("Failed to serialize visibility: {}", e)))?;

        client
            .execute(
                "INSERT INTO notes (id, owner_did, title, body, tags, visibility)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                &[&note_id, &owner_did, &title, &body, &tags, &visibility_json],
            )
            .await
            .map_err(|e| NoteRepoError::DatabaseError(format!("Failed to insert note: {}", e)))?;

        Ok(Note {
            id: note_id.to_string(),
            owner_did: owner_did.to_string(),
            title: title.to_string(),
            body: body.to_string(),
            tags,
            visibility,
            published_at: None,
            links: Vec::new(),
        })
    }

    async fn list(&self, viewer_did: Option<&str>) -> Result<Vec<Note>, NoteRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| NoteRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let query = if viewer_did.is_some() {
            "SELECT id, owner_did, title, body, tags, visibility, published_at, links, created_at, updated_at
             FROM notes
             WHERE owner_did = $1
                OR visibility->>'type' = 'Public'
                OR visibility->>'type' = 'Unlisted'
                OR (visibility->>'type' = 'SharedWith' AND visibility->'content' ? $1)
             ORDER BY created_at DESC"
        } else {
            "SELECT id, owner_did, title, body, tags, visibility, published_at, links, created_at, updated_at
             FROM notes
             WHERE visibility->>'type' IN ('Public', 'Unlisted')
             ORDER BY created_at DESC"
        };

        let rows = if let Some(did) = viewer_did {
            client.query(query, &[&did]).await
        } else {
            client.query(query, &[]).await
        };

        let rows = rows.map_err(|e| NoteRepoError::DatabaseError(format!("Failed to query notes: {}", e)))?;

        let mut notes = Vec::new();
        for row in rows {
            let visibility_json: serde_json::Value = row.get("visibility");
            let visibility: Visibility = serde_json::from_value(visibility_json)
                .map_err(|e| NoteRepoError::SerializationError(format!("Failed to deserialize visibility: {}", e)))?;

            let id: uuid::Uuid = row.get("id");
            let links: Vec<String> = row.get("links");

            notes.push(Note {
                id: id.to_string(),
                owner_did: row.get("owner_did"),
                title: row.get("title"),
                body: row.get("body"),
                tags: row.get("tags"),
                visibility,
                published_at: row
                    .get::<_, Option<chrono::DateTime<chrono::Utc>>>("published_at")
                    .map(|dt| dt.to_rfc3339()),
                links,
            });
        }

        Ok(notes)
    }

    async fn get(&self, id: &str, viewer_did: Option<&str>) -> Result<Note, NoteRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| NoteRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let note_id =
            uuid::Uuid::parse_str(id).map_err(|_| NoteRepoError::InvalidArgument("Invalid note ID".to_string()))?;

        let row = client
            .query_opt(
                "SELECT id, owner_did, title, body, tags, visibility, published_at, links, created_at, updated_at
                 FROM notes WHERE id = $1",
                &[&note_id],
            )
            .await
            .map_err(|e| NoteRepoError::DatabaseError(format!("Failed to query note: {}", e)))?
            .ok_or_else(|| NoteRepoError::NotFound("Note not found".to_string()))?;

        let visibility_json: serde_json::Value = row.get("visibility");
        let visibility: Visibility = serde_json::from_value(visibility_json)
            .map_err(|e| NoteRepoError::SerializationError(format!("Failed to deserialize visibility: {}", e)))?;

        let owner_did: String = row.get("owner_did");
        let is_owner = viewer_did == Some(owner_did.as_str());

        let has_access = match &visibility {
            Visibility::Public | Visibility::Unlisted => true,
            Visibility::Private => is_owner,
            Visibility::SharedWith(dids) => {
                is_owner || viewer_did.map(|did| dids.contains(&did.to_string())).unwrap_or(false)
            }
        };

        if !has_access {
            return Err(NoteRepoError::InvalidArgument("Access denied".to_string()));
        }

        let uuid_id: uuid::Uuid = row.get("id");
        let links: Vec<String> = row.get("links");

        Ok(Note {
            id: uuid_id.to_string(),
            owner_did,
            title: row.get("title"),
            body: row.get("body"),
            tags: row.get("tags"),
            visibility,
            published_at: row
                .get::<_, Option<chrono::DateTime<chrono::Utc>>>("published_at")
                .map(|dt| dt.to_rfc3339()),
            links,
        })
    }

    async fn get_notes_by_user(&self, owner_did: &str) -> Result<Vec<Note>, NoteRepoError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| NoteRepoError::DatabaseError(format!("Failed to get connection: {}", e)))?;

        let rows = client
            .query(
                "SELECT id, owner_did, title, body, tags, visibility, published_at, links, created_at, updated_at
                 FROM notes
                 WHERE owner_did = $1",
                &[&owner_did],
            )
            .await
            .map_err(|e| NoteRepoError::DatabaseError(format!("Failed to retrieve notes: {}", e)))?;

        let mut notes = Vec::new();
        for row in rows {
            let visibility_json: serde_json::Value = row.get("visibility");
            let visibility: Visibility = serde_json::from_value(visibility_json)
                .map_err(|e| NoteRepoError::SerializationError(format!("Failed to deserialize visibility: {}", e)))?;
            let id: uuid::Uuid = row.get("id");
            let links: Vec<String> = row.get("links");

            notes.push(Note {
                id: id.to_string(),
                owner_did: row.get("owner_did"),
                title: row.get("title"),
                body: row.get("body"),
                tags: row.get("tags"),
                visibility,
                published_at: row
                    .get::<_, Option<chrono::DateTime<chrono::Utc>>>("published_at")
                    .map(|dt| dt.to_rfc3339()),
                links,
            });
        }
        Ok(notes)
    }
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    pub struct MockNoteRepository {
        pub notes: Arc<Mutex<Vec<Note>>>,
        pub should_fail: Arc<Mutex<bool>>,
    }

    impl MockNoteRepository {
        pub fn new() -> Self {
            Self { notes: Arc::new(Mutex::new(Vec::new())), should_fail: Arc::new(Mutex::new(false)) }
        }

        pub fn with_notes(notes: Vec<Note>) -> Self {
            Self { notes: Arc::new(Mutex::new(notes)), should_fail: Arc::new(Mutex::new(false)) }
        }

        pub fn set_should_fail(&self, should_fail: bool) {
            *self.should_fail.lock().unwrap() = should_fail;
        }
    }

    impl Default for MockNoteRepository {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl NoteRepository for MockNoteRepository {
        async fn create(
            &self, owner_did: &str, title: &str, body: &str, tags: Vec<String>, visibility: Visibility,
        ) -> Result<Note, NoteRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(NoteRepoError::DatabaseError("Mock failure".to_string()));
            }

            let note = Note {
                id: uuid::Uuid::new_v4().to_string(),
                owner_did: owner_did.to_string(),
                title: title.to_string(),
                body: body.to_string(),
                tags,
                visibility,
                published_at: None,
                links: Vec::new(),
            };

            self.notes.lock().unwrap().push(note.clone());
            Ok(note)
        }

        async fn list(&self, viewer_did: Option<&str>) -> Result<Vec<Note>, NoteRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(NoteRepoError::DatabaseError("Mock failure".to_string()));
            }

            let notes = self.notes.lock().unwrap();
            let filtered: Vec<Note> = notes
                .iter()
                .filter(|note| {
                    let is_owner = viewer_did == Some(note.owner_did.as_str());
                    match &note.visibility {
                        Visibility::Public | Visibility::Unlisted => true,
                        Visibility::Private => is_owner,
                        Visibility::SharedWith(dids) => {
                            is_owner || viewer_did.map(|did| dids.contains(&did.to_string())).unwrap_or(false)
                        }
                    }
                })
                .cloned()
                .collect();
            Ok(filtered)
        }

        async fn get(&self, id: &str, viewer_did: Option<&str>) -> Result<Note, NoteRepoError> {
            if *self.should_fail.lock().unwrap() {
                return Err(NoteRepoError::DatabaseError("Mock failure".to_string()));
            }

            let notes = self.notes.lock().unwrap();
            let note = notes
                .iter()
                .find(|n| n.id == id)
                .ok_or_else(|| NoteRepoError::NotFound("Note not found".to_string()))?;

            let is_owner = viewer_did == Some(note.owner_did.as_str());
            let has_access = match &note.visibility {
                Visibility::Public | Visibility::Unlisted => true,
                Visibility::Private => is_owner,
                Visibility::SharedWith(dids) => {
                    is_owner || viewer_did.map(|did| dids.contains(&did.to_string())).unwrap_or(false)
                }
            };

            if !has_access {
                return Err(NoteRepoError::InvalidArgument("Access denied".to_string()));
            }

            Ok(note.clone())
        }

        async fn get_notes_by_user(&self, owner_did: &str) -> Result<Vec<Note>, NoteRepoError> {
            let notes = self.notes.lock().unwrap();
            let user_notes = notes.iter().filter(|n| n.owner_did == owner_did).cloned().collect();
            Ok(user_notes)
        }
    }
}
