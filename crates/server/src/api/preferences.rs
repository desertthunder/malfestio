use crate::middleware::auth::UserContext;
use crate::repository::preferences::{PreferencesRepoError, UpdatePreferences};
use crate::state::SharedState;

use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct UpdatePreferencesRequest {
    pub persona: Option<String>,
    pub complete_onboarding: Option<bool>,
    pub tutorial_deck_completed: Option<bool>,
    pub density_mode: Option<String>,
}

/// GET /api/preferences - Get current user preferences
pub async fn get_preferences(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let result = state.prefs_repo.get_or_create(&user.did).await;

    match result {
        Ok(prefs) => Json(prefs).into_response(),
        Err(e) => {
            tracing::error!("Failed to get preferences: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to get preferences"})),
            )
                .into_response()
        }
    }
}

/// PUT /api/preferences - Update user preferences
pub async fn update_preferences(
    State(state): State<SharedState>, ctx: Option<Extension<UserContext>>,
    Json(payload): Json<UpdatePreferencesRequest>,
) -> impl IntoResponse {
    let user = match ctx {
        Some(Extension(user)) => user,
        None => return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response(),
    };

    let persona = if let Some(ref p) = payload.persona {
        match p.parse() {
            Ok(persona) => Some(persona),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid persona. Must be 'learner', 'creator', or 'curator'"})),
                )
                    .into_response();
            }
        }
    } else {
        None
    };

    let updates = UpdatePreferences {
        persona,
        complete_onboarding: payload.complete_onboarding,
        tutorial_deck_completed: payload.tutorial_deck_completed,
        density_mode: payload.density_mode,
    };

    let result = state.prefs_repo.update(&user.did, updates).await;

    match result {
        Ok(prefs) => Json(prefs).into_response(),
        Err(PreferencesRepoError::NotFound(msg)) => {
            (StatusCode::NOT_FOUND, Json(json!({"error": msg}))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update preferences: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update preferences"})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::preferences::PreferencesRepository;
    use crate::repository::preferences::mock::MockPreferencesRepository;
    use crate::state::{AppConfig, AppState, Repositories};
    use std::sync::Arc;

    fn create_test_state_with_prefs(prefs_repo: Arc<dyn PreferencesRepository>) -> SharedState {
        let pool = crate::db::create_mock_pool();
        let card_repo = Arc::new(crate::repository::card::mock::MockCardRepository::new())
            as Arc<dyn crate::repository::card::CardRepository>;
        let note_repo = Arc::new(crate::repository::note::mock::MockNoteRepository::new())
            as Arc<dyn crate::repository::note::NoteRepository>;
        let oauth_repo = Arc::new(crate::repository::oauth::mock::MockOAuthRepository::new())
            as Arc<dyn crate::repository::oauth::OAuthRepository>;
        let social_repo = Arc::new(crate::repository::social::mock::MockSocialRepository::new())
            as Arc<dyn crate::repository::social::SocialRepository>;
        let deck_repo = Arc::new(crate::repository::deck::mock::MockDeckRepository::new())
            as Arc<dyn crate::repository::deck::DeckRepository>;
        let search_repo = Arc::new(crate::repository::search::mock::MockSearchRepository::new())
            as Arc<dyn crate::repository::search::SearchRepository>;
        let review_repo = Arc::new(crate::repository::review::mock::MockReviewRepository::new())
            as Arc<dyn crate::repository::review::ReviewRepository>;

        let config = AppConfig { pds_url: "https://bsky.social".to_string() };

        let repos = Repositories {
            card: card_repo,
            note: note_repo,
            oauth: oauth_repo,
            review: review_repo,
            social: social_repo,
            deck: deck_repo,
            search: search_repo,
            prefs: prefs_repo,
        };

        AppState::new(pool, repos, config)
    }

    #[tokio::test]
    async fn test_get_preferences_unauthorized() {
        let prefs_repo = Arc::new(MockPreferencesRepository::new()) as Arc<dyn PreferencesRepository>;
        let state = create_test_state_with_prefs(prefs_repo);

        let response = get_preferences(State(state), None).await.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_preferences_success() {
        let prefs_repo = Arc::new(MockPreferencesRepository::new()) as Arc<dyn PreferencesRepository>;
        let state = create_test_state_with_prefs(prefs_repo);

        let user = UserContext {
            did: "did:plc:test".to_string(),
            handle: "test.handle".to_string(),
            access_token: "test_token".to_string(),
            pds_url: "https://bsky.social".to_string(),
            has_dpop: false,
        };
        let response = get_preferences(State(state), Some(Extension(user)))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_preferences_set_persona() {
        let prefs_repo = Arc::new(MockPreferencesRepository::new()) as Arc<dyn PreferencesRepository>;
        let state = create_test_state_with_prefs(prefs_repo);

        let user = UserContext {
            did: "did:plc:test".to_string(),
            handle: "test.handle".to_string(),
            access_token: "test_token".to_string(),
            pds_url: "https://bsky.social".to_string(),
            has_dpop: false,
        };
        let payload = UpdatePreferencesRequest {
            persona: Some("creator".to_string()),
            complete_onboarding: Some(true),
            tutorial_deck_completed: None,
            density_mode: None,
        };

        let response = update_preferences(State(state), Some(Extension(user)), Json(payload))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_preferences_invalid_persona() {
        let prefs_repo = Arc::new(MockPreferencesRepository::new()) as Arc<dyn PreferencesRepository>;
        let state = create_test_state_with_prefs(prefs_repo);

        let user = UserContext {
            did: "did:plc:test".to_string(),
            handle: "test.handle".to_string(),
            access_token: "test_token".to_string(),
            pds_url: "https://bsky.social".to_string(),
            has_dpop: false,
        };
        let payload = UpdatePreferencesRequest {
            persona: Some("invalid".to_string()),
            complete_onboarding: None,
            tutorial_deck_completed: None,
            density_mode: None,
        };

        let response = update_preferences(State(state), Some(Extension(user)), Json(payload))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
