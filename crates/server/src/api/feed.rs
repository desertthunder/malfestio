use crate::middleware::auth::UserContext;
use crate::state::SharedState;

use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;

pub async fn get_feed_follows(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    match state.social_repo.get_feed_follows(&user.did).await {
        Ok(decks) => Json(decks).into_response(),
        Err(e) => {
            tracing::error!("Failed to get feed: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve feed"})),
            )
                .into_response()
        }
    }
}

pub async fn get_feed_trending(State(state): State<SharedState>) -> impl IntoResponse {
    match state.social_repo.get_feed_trending().await {
        Ok(decks) => Json(decks).into_response(),
        Err(e) => {
            tracing::error!("Failed to get trending: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to retrieve trending feed"})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::card::mock::MockCardRepository;
    use crate::repository::note::mock::MockNoteRepository;
    use crate::repository::oauth::mock::MockOAuthRepository;
    use crate::repository::review::mock::MockReviewRepository;
    use crate::repository::social::{SocialRepository, mock::MockSocialRepository};
    use crate::state::AppState;
    use std::sync::Arc;

    fn create_test_state_with_social(social_repo: Arc<dyn SocialRepository>) -> SharedState {
        let pool = crate::db::create_mock_pool();
        let card_repo = Arc::new(MockCardRepository::new()) as Arc<dyn crate::repository::card::CardRepository>;
        let note_repo = Arc::new(MockNoteRepository::new()) as Arc<dyn crate::repository::note::NoteRepository>;
        let oauth_repo = Arc::new(MockOAuthRepository::new()) as Arc<dyn crate::repository::oauth::OAuthRepository>;
        let review_repo = Arc::new(MockReviewRepository::new()) as Arc<dyn crate::repository::review::ReviewRepository>;

        let deck_repo = Arc::new(crate::repository::deck::mock::MockDeckRepository::new())
            as Arc<dyn crate::repository::deck::DeckRepository>;
        let config = crate::state::AppConfig { pds_url: "https://bsky.social".to_string() };

        let search_repo = Arc::new(crate::repository::search::mock::MockSearchRepository::new())
            as Arc<dyn crate::repository::search::SearchRepository>;

        let repos = crate::state::Repositories {
            card: card_repo,
            note: note_repo,
            oauth: oauth_repo,
            review: review_repo,
            social: social_repo,
            deck: deck_repo,
            search: search_repo,
        };

        AppState::new(pool, repos, config)
    }

    #[tokio::test]
    async fn test_get_feed_follows_success() {
        let social_repo = Arc::new(MockSocialRepository::new());
        let state = create_test_state_with_social(social_repo);
        let user = UserContext { did: "did:plc:test".to_string(), handle: "test.handle".to_string() };
        let response = get_feed_follows(State(state), Some(Extension(user)))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_feed_trending_success() {
        let social_repo = Arc::new(MockSocialRepository::new());
        let state = create_test_state_with_social(social_repo);
        let response = get_feed_trending(State(state)).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
