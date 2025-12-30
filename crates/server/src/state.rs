use crate::db::DbPool;
use crate::repository::card::{CardRepository, DbCardRepository};
use crate::repository::note::{DbNoteRepository, NoteRepository};
use crate::repository::oauth::{DbOAuthRepository, OAuthRepository};
use crate::repository::review::{DbReviewRepository, ReviewRepository};
use crate::repository::social::{DbSocialRepository, SocialRepository};

use std::sync::Arc;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub pool: DbPool,
    pub card_repo: Arc<dyn CardRepository>,
    pub note_repo: Arc<dyn NoteRepository>,
    pub oauth_repo: Arc<dyn OAuthRepository>,
    pub review_repo: Arc<dyn ReviewRepository>,
    pub social_repo: Arc<dyn SocialRepository>,
}

impl AppState {
    pub fn new(pool: DbPool) -> SharedState {
        let card_repo = Arc::new(DbCardRepository::new(pool.clone())) as Arc<dyn CardRepository>;
        let note_repo = Arc::new(DbNoteRepository::new(pool.clone())) as Arc<dyn NoteRepository>;
        let oauth_repo = Arc::new(DbOAuthRepository::new(pool.clone())) as Arc<dyn OAuthRepository>;
        let review_repo = Arc::new(DbReviewRepository::new(pool.clone())) as Arc<dyn ReviewRepository>;
        let social_repo = Arc::new(DbSocialRepository::new(pool.clone())) as Arc<dyn SocialRepository>;

        Arc::new(Self { pool, card_repo, note_repo, oauth_repo, review_repo, social_repo })
    }

    #[cfg(test)]
    pub fn new_with_repos(
        pool: DbPool, card_repo: Arc<dyn CardRepository>, note_repo: Arc<dyn NoteRepository>,
        oauth_repo: Arc<dyn OAuthRepository>,
    ) -> SharedState {
        use crate::repository;
        let review_repo = Arc::new(repository::review::mock::MockReviewRepository::new()) as Arc<dyn ReviewRepository>;
        let social_repo = Arc::new(repository::social::mock::MockSocialRepository::new()) as Arc<dyn SocialRepository>;
        Arc::new(Self { pool, card_repo, note_repo, oauth_repo, review_repo, social_repo })
    }
}
