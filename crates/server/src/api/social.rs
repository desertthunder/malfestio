use crate::middleware::auth::UserContext;
use crate::state::SharedState;

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct AddCommentRequest {
    pub content: String,
    pub parent_id: Option<String>,
}

pub async fn follow(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Path(subject_did): Path<String>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let result = state.social_repo.follow(&user.did, &subject_did).await;

    match result {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "followed"}))).into_response(),
        Err(malfestio_core::Error::Database(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to follow"})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{:?}", e)})),
        )
            .into_response(),
    }
}

pub async fn unfollow(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Path(subject_did): Path<String>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let result = state.social_repo.unfollow(&user.did, &subject_did).await;

    match result {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "unfollowed"}))).into_response(),
        Err(malfestio_core::Error::Database(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to unfollow"})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{:?}", e)})),
        )
            .into_response(),
    }
}

pub async fn get_followers(State(state): State<SharedState>, Path(did): Path<String>) -> impl IntoResponse {
    let result = state.social_repo.get_followers(&did).await;

    match result {
        Ok(followers) => Json(followers).into_response(),
        Err(malfestio_core::Error::Database(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get followers"})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{:?}", e)})),
        )
            .into_response(),
    }
}

pub async fn get_following(State(state): State<SharedState>, Path(did): Path<String>) -> impl IntoResponse {
    let result = state.social_repo.get_following(&did).await;

    match result {
        Ok(following) => Json(following).into_response(),
        Err(malfestio_core::Error::Database(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get following"})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{:?}", e)})),
        )
            .into_response(),
    }
}

pub async fn add_comment(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>, Path(deck_id): Path<String>,
    Json(payload): Json<AddCommentRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let result = state
        .social_repo
        .add_comment(&deck_id, &user.did, &payload.content, payload.parent_id.as_deref())
        .await;

    match result {
        Ok(comment) => (StatusCode::CREATED, Json(comment)).into_response(),
        Err(malfestio_core::Error::Database(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to add comment"})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{:?}", e)})),
        )
            .into_response(),
    }
}

pub async fn get_comments(State(state): State<SharedState>, Path(deck_id): Path<String>) -> impl IntoResponse {
    let result = state.social_repo.get_comments(&deck_id).await;

    match result {
        Ok(comments) => Json(comments).into_response(),
        Err(malfestio_core::Error::Database(msg)) => {
            tracing::error!("Database error: {}", msg);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get comments"})),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("{:?}", e)})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::auth::UserContext;
    use crate::repository::card::mock::MockCardRepository;
    use crate::repository::note::mock::MockNoteRepository;
    use crate::repository::oauth::mock::MockOAuthRepository;
    use crate::repository::review::mock::MockReviewRepository;
    use crate::repository::social::{SocialRepository, mock::MockSocialRepository};
    use crate::state::AppState;
    use axum::extract::Json;
    use std::sync::Arc;

    fn create_test_state_with_social(social_repo: Arc<dyn SocialRepository>) -> SharedState {
        let pool = crate::db::create_mock_pool();
        let card_repo = Arc::new(MockCardRepository::new()) as Arc<dyn crate::repository::card::CardRepository>;
        let note_repo = Arc::new(MockNoteRepository::new()) as Arc<dyn crate::repository::note::NoteRepository>;
        let oauth_repo = Arc::new(MockOAuthRepository::new()) as Arc<dyn crate::repository::oauth::OAuthRepository>;
        let preferences_repo = Arc::new(crate::repository::preferences::mock::MockPreferencesRepository::new())
            as Arc<dyn crate::repository::preferences::PreferencesRepository>;
        let review_repo = Arc::new(MockReviewRepository::new()) as Arc<dyn crate::repository::review::ReviewRepository>;

        let deck_repo = Arc::new(crate::repository::deck::mock::MockDeckRepository::new())
            as Arc<dyn crate::repository::deck::DeckRepository>;
        let config = crate::state::AppConfig { pds_url: "https://bsky.social".to_string() };
        let search_repo = Arc::new(crate::repository::search::mock::MockSearchRepository::new())
            as Arc<dyn crate::repository::search::SearchRepository>;
        let auth_cache = Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));

        Arc::new(AppState {
            pool,
            card_repo,
            note_repo,
            oauth_repo,
            prefs_repo: preferences_repo,
            review_repo,
            social_repo,
            deck_repo,
            search_repo,
            config,
            auth_cache,
            dpop_nonces: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        })
    }

    #[tokio::test]
    async fn test_follow_success() {
        let social_repo = Arc::new(MockSocialRepository::new());
        let state = create_test_state_with_social(social_repo.clone());
        let user = UserContext { did: "did:plc:follower".to_string(), handle: "follower".to_string() };

        let response = follow(State(state), Some(Extension(user)), Path("did:plc:subject".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let followers = social_repo.get_followers("did:plc:subject").await.unwrap();
        assert!(followers.contains(&"did:plc:follower".to_string()));
    }

    #[tokio::test]
    async fn test_unfollow_success() {
        let social_repo = Arc::new(MockSocialRepository::new());
        social_repo.follow("did:plc:follower", "did:plc:subject").await.unwrap();

        let state = create_test_state_with_social(social_repo.clone());
        let user = UserContext { did: "did:plc:follower".to_string(), handle: "follower".to_string() };

        let response = unfollow(State(state), Some(Extension(user)), Path("did:plc:subject".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let followers = social_repo.get_followers("did:plc:subject").await.unwrap();
        assert!(followers.is_empty());
    }

    #[tokio::test]
    async fn test_get_followers() {
        let social_repo = Arc::new(MockSocialRepository::new());
        social_repo.follow("did:plc:1", "did:plc:subject").await.unwrap();
        social_repo.follow("did:plc:2", "did:plc:subject").await.unwrap();

        let state = create_test_state_with_social(social_repo);

        let response = get_followers(State(state), Path("did:plc:subject".to_string()))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let followers: Vec<String> = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(followers.len(), 2);
        assert!(followers.contains(&"did:plc:1".to_string()));
        assert!(followers.contains(&"did:plc:2".to_string()));
    }

    #[tokio::test]
    async fn test_add_comment_success() {
        let social_repo = Arc::new(MockSocialRepository::new());
        let state = create_test_state_with_social(social_repo.clone());
        let user = UserContext { did: "did:plc:author".to_string(), handle: "author".to_string() };

        let payload = AddCommentRequest { content: "Great deck!".to_string(), parent_id: None };

        let response = add_comment(
            State(state),
            Some(Extension(user)),
            Path("deck-1".to_string()),
            Json(payload),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::CREATED);

        let comments = social_repo.get_comments("deck-1").await.unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].content, "Great deck!");
    }
}
