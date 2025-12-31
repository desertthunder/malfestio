use crate::state::SharedState;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;

/// GET /api/users/:did/profile
/// Get aggregated profile information for a user
pub async fn get_profile(State(state): State<SharedState>, Path(did): Path<String>) -> impl IntoResponse {
    match state.social_repo.get_user_profile(&did).await {
        Ok(profile) => Json(profile).into_response(),
        Err(e) => {
            tracing::error!("Failed to get user profile: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get user profile"})),
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
    use crate::repository::preferences::mock::MockPreferencesRepository;
    use crate::repository::review::mock::MockReviewRepository;
    use crate::repository::search::mock::MockSearchRepository;
    use crate::repository::social::mock::MockSocialRepository;
    use crate::repository::social::{SocialRepository, UserProfile};
    use crate::state::AppState;
    use std::sync::Arc;

    fn create_test_state(social_repo: Arc<MockSocialRepository>) -> SharedState {
        let pool = crate::db::create_mock_pool();
        Arc::new(AppState {
            pool,
            card_repo: Arc::new(MockCardRepository::new()) as Arc<dyn crate::repository::card::CardRepository>,
            note_repo: Arc::new(MockNoteRepository::new()) as Arc<dyn crate::repository::note::NoteRepository>,
            oauth_repo: Arc::new(MockOAuthRepository::new()) as Arc<dyn crate::repository::oauth::OAuthRepository>,
            prefs_repo: Arc::new(MockPreferencesRepository::new())
                as Arc<dyn crate::repository::preferences::PreferencesRepository>,
            review_repo: Arc::new(MockReviewRepository::new()) as Arc<dyn crate::repository::review::ReviewRepository>,
            social_repo: social_repo.clone() as Arc<dyn SocialRepository>,
            deck_repo: Arc::new(MockDeckRepository::new()) as Arc<dyn crate::repository::deck::DeckRepository>,
            search_repo: Arc::new(MockSearchRepository::new()) as Arc<dyn crate::repository::search::SearchRepository>,
            config: crate::state::AppConfig { pds_url: "https://bsky.social".to_string() },
            auth_cache: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            dpop_nonces: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        })
    }

    #[tokio::test]
    async fn test_get_profile_returns_counts() {
        let social_repo = Arc::new(MockSocialRepository::new());
        social_repo.follow("did:follower1", "did:user").await.unwrap();
        social_repo.follow("did:follower2", "did:user").await.unwrap();
        social_repo.follow("did:user", "did:following").await.unwrap();

        let state = create_test_state(social_repo);
        let response = get_profile(State(state), Path("did:user".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let profile: UserProfile = serde_json::from_slice(&body).unwrap();

        assert_eq!(profile.did, "did:user");
        assert_eq!(profile.follower_count, 2);
        assert_eq!(profile.following_count, 1);
    }

    #[tokio::test]
    async fn test_get_profile_unknown_user_returns_zero() {
        let social_repo = Arc::new(MockSocialRepository::new());
        let state = create_test_state(social_repo);

        let response = get_profile(State(state), Path("did:unknown".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let profile: UserProfile = serde_json::from_slice(&body).unwrap();

        assert_eq!(profile.follower_count, 0);
        assert_eq!(profile.following_count, 0);
    }
}
