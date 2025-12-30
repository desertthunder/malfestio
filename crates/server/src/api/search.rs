use crate::middleware::auth::UserContext;
use crate::state::SharedState;
use axum::{
    Json,
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default = "default_offset")]
    offset: i64,
}

fn default_limit() -> i64 {
    20
}

fn default_offset() -> i64 {
    0
}

/// GET /api/search?q=...
/// Search for decks, cards, and notes using full-text search
///
/// TODO: filter by user
pub async fn search(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let user_did = ctx.map(|Extension(u)| u.did);

    match state
        .search_repo
        .search(&query.q, query.limit, query.offset, user_did.as_deref())
        .await
    {
        Ok(results) => Json(results).into_response(),
        Err(e) => {
            tracing::error!("Search failed: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Search failed"})),
            )
                .into_response()
        }
    }
}

/// GET /api/discovery
/// Get discovery info like top tags
pub async fn discovery(State(state): State<SharedState>) -> impl IntoResponse {
    match state.search_repo.get_top_tags(10).await {
        Ok(tags) => Json(json!({ "top_tags": tags })).into_response(),
        Err(e) => {
            tracing::error!("Discovery failed: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Discovery failed"})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::card::mock::MockCardRepository;
    use crate::repository::deck::mock::MockDeckRepository;
    use crate::repository::note::mock::MockNoteRepository;
    use crate::repository::oauth::mock::MockOAuthRepository;
    use crate::repository::review::mock::MockReviewRepository;
    use crate::repository::search::mock::MockSearchRepository;
    use crate::repository::search::{SearchRepository, SearchResult};
    use crate::repository::social::mock::MockSocialRepository;
    use crate::state::AppState;
    use std::sync::Arc;

    fn create_test_state_with_search(search_repo: Arc<MockSearchRepository>) -> SharedState {
        let pool = crate::db::create_mock_pool();
        let card_repo = Arc::new(MockCardRepository::new()) as Arc<dyn crate::repository::card::CardRepository>;
        let note_repo = Arc::new(MockNoteRepository::new()) as Arc<dyn crate::repository::note::NoteRepository>;
        let oauth_repo = Arc::new(MockOAuthRepository::new()) as Arc<dyn crate::repository::oauth::OAuthRepository>;
        let review_repo = Arc::new(MockReviewRepository::new()) as Arc<dyn crate::repository::review::ReviewRepository>;
        let social_repo = Arc::new(MockSocialRepository::new()) as Arc<dyn crate::repository::social::SocialRepository>;
        let deck_repo = Arc::new(MockDeckRepository::new()) as Arc<dyn crate::repository::deck::DeckRepository>;
        let config = crate::state::AppConfig { pds_url: "https://bsky.social".to_string() };
        let auth_cache = Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
        let search_repo_trait = search_repo.clone() as Arc<dyn SearchRepository>;
        let prefs_repo = Arc::new(crate::repository::preferences::mock::MockPreferencesRepository::new())
            as Arc<dyn crate::repository::preferences::PreferencesRepository>;

        Arc::new(AppState {
            pool,
            card_repo,
            note_repo,
            oauth_repo,
            prefs_repo,
            review_repo,
            social_repo,
            deck_repo,
            search_repo: search_repo_trait,
            config,
            auth_cache,
        })
    }

    #[tokio::test]
    async fn test_search_handler_passes_viewer_did() {
        let search_repo = Arc::new(MockSearchRepository::new());
        search_repo
            .add_result(SearchResult {
                item_type: "deck".to_string(),
                item_id: "private-deck".to_string(),
                creator_did: "did:alice".to_string(),
                data: serde_json::json!({ "title": "Secret", "visibility": { "type": "Private" } }),
                rank: 1.0,
            })
            .await;

        let state = create_test_state_with_search(search_repo.clone());
        let auth_ctx = Extension(UserContext { did: "did:alice".to_string(), handle: "alice.test".to_string() });
        let response = search(
            State(state.clone()),
            Some(auth_ctx),
            Query(SearchQuery { q: "private".to_string(), limit: 10, offset: 0 }),
        )
        .await
        .into_response();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let results: Vec<SearchResult> = serde_json::from_slice(&body).unwrap();

        assert_eq!(results.len(), 1, "Alice should see her private deck");
        assert_eq!(results[0].item_id, "private-deck");

        let response_anon = search(
            State(state.clone()),
            None,
            Query(SearchQuery { q: "private".to_string(), limit: 10, offset: 0 }),
        )
        .await
        .into_response();

        let body_anon = axum::body::to_bytes(response_anon.into_body(), usize::MAX)
            .await
            .unwrap();
        let results_anon: Vec<SearchResult> = serde_json::from_slice(&body_anon).unwrap();

        assert_eq!(results_anon.len(), 0, "Anonymous user should not see private deck");
    }
}
