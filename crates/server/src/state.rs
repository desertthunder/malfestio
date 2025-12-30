use crate::db::DbPool;
use crate::middleware::auth::UserContext;
use crate::repository::card::CardRepository;
use crate::repository::deck::DeckRepository;
use crate::repository::note::NoteRepository;
use crate::repository::oauth::OAuthRepository;
use crate::repository::review::ReviewRepository;
use crate::repository::social::SocialRepository;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

pub type SharedState = Arc<AppState>;

#[derive(Clone)]
pub struct AppConfig {
    pub pds_url: String,
}

pub type AuthCache = Arc<RwLock<HashMap<String, (UserContext, Instant)>>>;

pub struct Repositories {
    pub oauth: Arc<dyn OAuthRepository>,
    pub deck: Arc<dyn DeckRepository>,
    pub card: Arc<dyn CardRepository>,
    pub note: Arc<dyn NoteRepository>,
    pub review: Arc<dyn ReviewRepository>,
    pub social: Arc<dyn SocialRepository>,
}

pub struct AppState {
    pub pool: DbPool,
    pub card_repo: Arc<dyn CardRepository>,
    pub deck_repo: Arc<dyn DeckRepository>,
    pub note_repo: Arc<dyn NoteRepository>,
    pub oauth_repo: Arc<dyn OAuthRepository>,
    pub review_repo: Arc<dyn ReviewRepository>,
    pub social_repo: Arc<dyn SocialRepository>,
    pub config: AppConfig,
    pub auth_cache: AuthCache,
}

impl AppState {
    pub fn new(pool: DbPool, repos: Repositories, config: AppConfig) -> SharedState {
        let auth_cache = Arc::new(RwLock::new(HashMap::new()));
        Arc::new(Self {
            pool,
            oauth_repo: repos.oauth,
            deck_repo: repos.deck,
            card_repo: repos.card,
            note_repo: repos.note,
            review_repo: repos.review,
            social_repo: repos.social,
            config,
            auth_cache,
        })
    }

    #[cfg(test)]
    pub fn new_with_repos(
        pool: DbPool, card_repo: Arc<dyn CardRepository>, note_repo: Arc<dyn NoteRepository>,
        oauth_repo: Arc<dyn OAuthRepository>,
    ) -> SharedState {
        use crate::repository;
        let review_repo = Arc::new(repository::review::mock::MockReviewRepository::new()) as Arc<dyn ReviewRepository>;
        let social_repo = Arc::new(repository::social::mock::MockSocialRepository::new()) as Arc<dyn SocialRepository>;
        let deck_repo = Arc::new(repository::deck::mock::MockDeckRepository::new()) as Arc<dyn DeckRepository>;
        let config = AppConfig { pds_url: "https://bsky.social".to_string() };

        let repos = Repositories {
            card: card_repo,
            note: note_repo,
            oauth: oauth_repo,
            review: review_repo,
            social: social_repo,
            deck: deck_repo,
        };

        Self::new(pool, repos, config)
    }
}
