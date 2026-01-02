use crate::middleware::auth::UserContext;
use crate::repository::review::ReviewRepoError;
use crate::state::SharedState;

use axum::{
    Json,
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use malfestio_core::srs::Grade;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
pub struct DueCardsQuery {
    deck_id: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Deserialize)]
pub struct SubmitReviewRequest {
    card_id: String,
    grade: u8,
}

#[derive(Serialize)]
pub struct SubmitReviewResponse {
    ease_factor: f32,
    interval_days: i32,
    repetitions: i32,
    due_at: String,
}

/// GET /api/review/due - Get cards due for review
pub async fn get_due_cards(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Query(query): Query<DueCardsQuery>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let result = state
        .review_repo
        .get_due_cards(&user.did, query.deck_id.as_deref(), query.limit)
        .await;

    match result {
        Ok(cards) => Json(cards).into_response(),
        Err(ReviewRepoError::InvalidArgument(msg)) => {
            (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get due cards: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get due cards"})),
            )
                .into_response()
        }
    }
}

/// POST /api/review/submit - Submit a review grade
pub async fn submit_review(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Json(payload): Json<SubmitReviewRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let grade = match Grade::new(payload.grade) {
        Some(g) => g,
        None => return (StatusCode::BAD_REQUEST, Json(json!({"error": "Grade must be 0-5"}))).into_response(),
    };

    let result = state
        .review_repo
        .submit_review(&user.did, &payload.card_id, grade)
        .await;

    match result {
        Ok(new_state) => Json(SubmitReviewResponse {
            ease_factor: new_state.ease_factor,
            interval_days: new_state.interval_days,
            repetitions: new_state.repetitions,
            due_at: new_state.due_at.to_rfc3339(),
        })
        .into_response(),
        Err(ReviewRepoError::InvalidArgument(msg)) => {
            (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to submit review: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to submit review"})),
            )
                .into_response()
        }
    }
}

/// GET /api/review/stats - Get user study statistics
pub async fn get_stats(State(state): State<SharedState>, ctx: Option<Extension<UserContext>>) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let result = state.review_repo.get_stats(&user.did).await;

    match result {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => {
            tracing::error!("Failed to get stats: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get stats"})),
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
    use crate::repository::review::{ReviewCard, ReviewRepository};
    use crate::state::AppState;
    use chrono::Utc;
    use std::sync::Arc;

    fn create_test_state_with_review(review_repo: Arc<dyn ReviewRepository>) -> SharedState {
        let pool = crate::db::create_mock_pool();
        let card_repo = Arc::new(MockCardRepository::new()) as Arc<dyn crate::repository::card::CardRepository>;
        let note_repo = Arc::new(MockNoteRepository::new()) as Arc<dyn crate::repository::note::NoteRepository>;
        let oauth_repo = Arc::new(MockOAuthRepository::new()) as Arc<dyn crate::repository::oauth::OAuthRepository>;
        let preferences_repo = Arc::new(crate::repository::preferences::mock::MockPreferencesRepository::new())
            as Arc<dyn crate::repository::preferences::PreferencesRepository>;
        let social_repo = Arc::new(crate::repository::social::mock::MockSocialRepository::new())
            as Arc<dyn crate::repository::social::SocialRepository>;

        let deck_repo = Arc::new(crate::repository::deck::mock::MockDeckRepository::new())
            as Arc<dyn crate::repository::deck::DeckRepository>;
        let config = crate::state::AppConfig { pds_url: "https://bsky.social".to_string() };

        let search_repo = Arc::new(crate::repository::search::mock::MockSearchRepository::new())
            as Arc<dyn crate::repository::search::SearchRepository>;

        let repos = crate::state::Repositories {
            card: card_repo,
            note: note_repo,
            oauth: oauth_repo,
            prefs: preferences_repo,
            review: review_repo,
            social: social_repo,
            deck: deck_repo,
            search: search_repo,
        };

        AppState::new(pool, repos, config)
    }

    #[tokio::test]
    async fn test_get_due_cards_unauthorized() {
        let review_repo = Arc::new(MockReviewRepository::new()) as Arc<dyn ReviewRepository>;
        let state = create_test_state_with_review(review_repo);

        let response = get_due_cards(State(state), None, Query(DueCardsQuery { deck_id: None, limit: 20 }))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_due_cards_success() {
        let cards = vec![ReviewCard {
            review_id: "review-1".to_string(),
            card_id: "card-1".to_string(),
            deck_id: "deck-1".to_string(),
            deck_title: "Test Deck".to_string(),
            front: "What is 2+2?".to_string(),
            back: "4".to_string(),
            media_url: None,
            hints: vec![],
            due_at: Utc::now(),
        }];
        let review_repo = Arc::new(MockReviewRepository::with_cards(cards)) as Arc<dyn ReviewRepository>;
        let state = create_test_state_with_review(review_repo);

        let user = UserContext {
            did: "did:plc:test".to_string(),
            handle: "test.handle".to_string(),
            access_token: "test_token".to_string(),
            pds_url: "https://bsky.social".to_string(),
            has_dpop: false,
        };
        let response = get_due_cards(
            State(state),
            Some(Extension(user)),
            Query(DueCardsQuery { deck_id: None, limit: 20 }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_submit_review_success() {
        let review_repo = Arc::new(MockReviewRepository::new()) as Arc<dyn ReviewRepository>;
        let state = create_test_state_with_review(review_repo);

        let user = UserContext {
            did: "did:plc:test".to_string(),
            handle: "test.handle".to_string(),
            access_token: "test_token".to_string(),
            pds_url: "https://bsky.social".to_string(),
            has_dpop: false,
        };
        let payload = SubmitReviewRequest { card_id: "card-1".to_string(), grade: 3 };

        let response = submit_review(State(state), Some(Extension(user)), Json(payload))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_submit_review_invalid_grade() {
        let review_repo = Arc::new(MockReviewRepository::new()) as Arc<dyn ReviewRepository>;
        let state = create_test_state_with_review(review_repo);

        let user = UserContext {
            did: "did:plc:test".to_string(),
            handle: "test.handle".to_string(),
            access_token: "test_token".to_string(),
            pds_url: "https://bsky.social".to_string(),
            has_dpop: false,
        };
        let payload = SubmitReviewRequest { card_id: "card-1".to_string(), grade: 10 };

        let response = submit_review(State(state), Some(Extension(user)), Json(payload))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_stats_success() {
        let review_repo = Arc::new(MockReviewRepository::new()) as Arc<dyn ReviewRepository>;
        let state = create_test_state_with_review(review_repo);

        let user = UserContext {
            did: "did:plc:test".to_string(),
            handle: "test.handle".to_string(),
            access_token: "test_token".to_string(),
            pds_url: "https://bsky.social".to_string(),
            has_dpop: false,
        };
        let response = get_stats(State(state), Some(Extension(user))).await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
