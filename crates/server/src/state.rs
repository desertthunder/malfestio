use crate::db::DbPool;
use crate::repository::card::{CardRepository, DbCardRepository};
use crate::repository::note::{DbNoteRepository, NoteRepository};
use std::sync::Arc;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub pool: DbPool,
    pub card_repo: Arc<dyn CardRepository>,
    pub note_repo: Arc<dyn NoteRepository>,
}

impl AppState {
    pub fn new(pool: DbPool) -> SharedState {
        let card_repo = Arc::new(DbCardRepository::new(pool.clone())) as Arc<dyn CardRepository>;
        let note_repo = Arc::new(DbNoteRepository::new(pool.clone())) as Arc<dyn NoteRepository>;

        Arc::new(Self { pool, card_repo, note_repo })
    }

    #[cfg(test)]
    pub fn new_with_repos(
        pool: DbPool, card_repo: Arc<dyn CardRepository>, note_repo: Arc<dyn NoteRepository>,
    ) -> SharedState {
        Arc::new(Self { pool, card_repo, note_repo })
    }
}
