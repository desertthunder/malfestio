pub mod api;
pub mod db;
pub mod firehose;
pub mod middleware;
pub mod oauth;
pub mod pds;
pub mod repository;
pub mod state;
pub mod well_known;

use axum::http::Method;
use axum::{
    Json, Router,
    http::StatusCode,
    middleware as axum_middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn start() -> malfestio_core::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "malfestio_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Malfestio Server...");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| std::env::var("DB_URL").expect("DATABASE_URL or DB_URL must be set"));
    let pool = db::create_pool(&database_url).map_err(|e| {
        tracing::error!("Failed to create database pool: {}", e);
        malfestio_core::Error::Database(format!("Failed to create database pool: {}", e))
    })?;

    tracing::info!("Database connection pool created");

    let oauth_repo = std::sync::Arc::new(repository::oauth::DbOAuthRepository::new(pool.clone()));
    let deck_repo = std::sync::Arc::new(repository::deck::DbDeckRepository::new(pool.clone()));
    let card_repo = std::sync::Arc::new(repository::card::DbCardRepository::new(pool.clone()));
    let note_repo = std::sync::Arc::new(repository::note::DbNoteRepository::new(pool.clone()));
    let review_repo = std::sync::Arc::new(repository::review::DbReviewRepository::new(pool.clone()));
    let social_repo = std::sync::Arc::new(repository::social::DbSocialRepository::new(pool.clone()));

    let pds_url = std::env::var("PDS_URL").unwrap_or_else(|_| "https://bsky.social".to_string());
    let config = state::AppConfig { pds_url };

    let repos = state::Repositories {
        oauth: oauth_repo,
        deck: deck_repo,
        card: card_repo,
        note: note_repo,
        review: review_repo,
        social: social_repo,
    };

    let state = state::AppState::new(pool, repos, config);
    let oauth_state = std::sync::Arc::new(api::oauth::OAuthState::new());

    let auth_routes = Router::new()
        .route("/me", get(api::auth::me))
        .route("/decks", post(api::deck::create_deck))
        .route("/decks/{id}/publish", post(api::deck::publish_deck))
        .route("/decks/{id}/fork", post(api::deck::fork_deck))
        .route("/notes", post(api::note::create_note))
        .route("/cards", post(api::card::create_card))
        .route("/review/due", get(api::review::get_due_cards))
        .route("/review/submit", post(api::review::submit_review))
        .route("/review/stats", get(api::review::get_stats))
        .route("/social/follow/{did}", post(api::social::follow))
        .route("/social/unfollow/{did}", post(api::social::unfollow))
        .route("/decks/{id}/comments", post(api::social::add_comment))
        .route("/feeds/follows", get(api::feed::get_feed_follows))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::auth_middleware,
        ));

    let optional_auth_routes = Router::new()
        .route("/decks", get(api::deck::list_decks))
        .route("/decks/{id}", get(api::deck::get_deck))
        .route("/decks/{id}/cards", get(api::card::list_cards))
        .route("/notes", get(api::note::list_notes))
        .route("/notes/{id}", get(api::note::get_note))
        .route("/social/followers/{did}", get(api::social::get_followers))
        .route("/social/following/{did}", get(api::social::get_following))
        .route("/decks/{id}/comments", get(api::social::get_comments))
        .route("/feeds/trending", get(api::feed::get_feed_trending))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth::optional_auth_middleware,
        ));

    let oauth_routes = Router::new()
        .route("/authorize", post(api::oauth::authorize))
        .route("/callback", get(api::oauth::callback))
        .route("/refresh", post(api::oauth::refresh))
        .with_state(oauth_state.clone());

    let app = Router::new()
        .route("/health", get(health_check))
        .route(
            "/.well-known/oauth-client-metadata",
            get(oauth::client_metadata::client_metadata_handler),
        )
        .route("/.well-known/atproto-did", get(well_known::atproto_did_handler))
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/import/article", post(api::importer::import_article))
        .nest("/api/oauth", oauth_routes)
        .nest("/api", optional_auth_routes)
        .nest("/api", auth_routes)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any),
        )
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    tracing::info!("Listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

pub struct AppError(malfestio_core::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self.0 {
            malfestio_core::Error::NotFound(_) => (StatusCode::NOT_FOUND, self.0.to_string()),
            malfestio_core::Error::InvalidArgument(_) => (StatusCode::BAD_REQUEST, self.0.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
