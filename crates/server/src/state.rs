use crate::db::DbPool;
use crate::middleware::auth::UserContext;
use crate::repository;
use crate::repository::card::CardRepository;
use crate::repository::deck::DeckRepository;
use crate::repository::note::NoteRepository;
use crate::repository::oauth::OAuthRepository;
use crate::repository::preferences::PreferencesRepository;
use crate::repository::review::ReviewRepository;
use crate::repository::search::SearchRepository;
use crate::repository::social::SocialRepository;

use deadpool_postgres::Pool;
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

/// Cache for DPoP nonces with their creation timestamps for TTL enforcement.
pub type DpopNonceCache = Arc<RwLock<HashMap<String, Instant>>>;

pub struct Repositories {
    pub oauth: Arc<dyn OAuthRepository>,
    pub deck: Arc<dyn DeckRepository>,
    pub card: Arc<dyn CardRepository>,
    pub note: Arc<dyn NoteRepository>,
    pub prefs: Arc<dyn PreferencesRepository>,
    pub review: Arc<dyn ReviewRepository>,
    pub social: Arc<dyn SocialRepository>,
    pub search: Arc<dyn SearchRepository>,
}

#[cfg(test)]
impl Default for Repositories {
    fn default() -> Self {
        Self {
            oauth: Arc::new(repository::oauth::mock::MockOAuthRepository::new()),
            deck: Arc::new(repository::deck::mock::MockDeckRepository::new()),
            card: Arc::new(repository::card::mock::MockCardRepository::new()),
            note: Arc::new(repository::note::mock::MockNoteRepository::new()),
            prefs: Arc::new(repository::preferences::mock::MockPreferencesRepository::new()),
            review: Arc::new(repository::review::mock::MockReviewRepository::new()),
            social: Arc::new(repository::social::mock::MockSocialRepository::new()),
            search: Arc::new(repository::search::mock::MockSearchRepository::new()),
        }
    }
}

impl From<&Pool> for Repositories {
    fn from(pool: &Pool) -> Self {
        let oauth_repo = std::sync::Arc::new(repository::oauth::DbOAuthRepository::new(pool.clone()));
        let deck_repo = std::sync::Arc::new(repository::deck::DbDeckRepository::new(pool.clone()));
        let card_repo = std::sync::Arc::new(repository::card::DbCardRepository::new(pool.clone()));
        let note_repo = std::sync::Arc::new(repository::note::DbNoteRepository::new(pool.clone()));
        let prefs_repo = std::sync::Arc::new(repository::preferences::DbPreferencesRepository::new(pool.clone()));
        let review_repo = std::sync::Arc::new(repository::review::DbReviewRepository::new(pool.clone()));
        let social_repo = std::sync::Arc::new(repository::social::DbSocialRepository::new(pool.clone()));
        let search_repo = std::sync::Arc::new(repository::search::DbSearchRepository::new(pool.clone()));

        Self {
            oauth: oauth_repo,
            deck: deck_repo,
            card: card_repo,
            note: note_repo,
            prefs: prefs_repo,
            review: review_repo,
            social: social_repo,
            search: search_repo,
        }
    }
}

pub struct AppState {
    pub pool: DbPool,
    pub card_repo: Arc<dyn CardRepository>,
    pub deck_repo: Arc<dyn DeckRepository>,
    pub note_repo: Arc<dyn NoteRepository>,
    pub oauth_repo: Arc<dyn OAuthRepository>,
    pub prefs_repo: Arc<dyn PreferencesRepository>,
    pub review_repo: Arc<dyn ReviewRepository>,
    pub social_repo: Arc<dyn SocialRepository>,
    pub search_repo: Arc<dyn SearchRepository>,
    pub config: AppConfig,
    pub auth_cache: AuthCache,
    /// Cache of valid DPoP nonces. Nonces are single-use and expire after TTL.
    pub dpop_nonces: DpopNonceCache,
}

impl AppState {
    pub fn new(pool: DbPool, repos: Repositories, config: AppConfig) -> SharedState {
        let auth_cache = Arc::new(RwLock::new(HashMap::new()));
        let dpop_nonces = Arc::new(RwLock::new(HashMap::new()));
        Arc::new(Self {
            pool,
            oauth_repo: repos.oauth,
            deck_repo: repos.deck,
            card_repo: repos.card,
            note_repo: repos.note,
            prefs_repo: repos.prefs,
            review_repo: repos.review,
            social_repo: repos.social,
            search_repo: repos.search,
            config,
            auth_cache,
            dpop_nonces,
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
        let search_repo = Arc::new(repository::search::mock::MockSearchRepository::new()) as Arc<dyn SearchRepository>;
        let deck_repo = Arc::new(repository::deck::mock::MockDeckRepository::new()) as Arc<dyn DeckRepository>;
        let config = AppConfig { pds_url: "https://bsky.social".to_string() };
        let prefs_repo =
            Arc::new(repository::preferences::mock::MockPreferencesRepository::new()) as Arc<dyn PreferencesRepository>;

        let repos = Repositories {
            card: card_repo,
            note: note_repo,
            oauth: oauth_repo,
            prefs: prefs_repo,
            review: review_repo,
            social: social_repo,
            search: search_repo,
            deck: deck_repo,
        };

        Self::new(pool, repos, config)
    }
}
