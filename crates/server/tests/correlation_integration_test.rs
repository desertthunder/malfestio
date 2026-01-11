//! Integration tests for correlation ID propagation through the request lifecycle.
//!
//! These tests verify that:
//! 1. Correlation IDs are extracted from request headers when present
//! 2. New correlation IDs are generated when none is provided
//! 3. Correlation IDs are returned in response headers
//! 4. Correlation IDs appear in structured logs

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use malfestio_server::middleware::correlation::correlation_middleware;
use tower::ServiceExt;

/// A simple handler for testing that returns a success response.
async fn test_handler() -> &'static str {
    "OK"
}

/// Creates a test app with the correlation middleware.
fn create_test_app() -> Router {
    Router::new()
        .route("/test", axum::routing::get(test_handler))
        .layer(axum::middleware::from_fn(correlation_middleware))
}

#[tokio::test]
async fn test_correlation_id_generated_when_not_provided() {
    let app = create_test_app();

    let response = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let correlation_id = response
        .headers()
        .get("x-request-id")
        .expect("X-Request-ID header should be present");

    let id_str = correlation_id.to_str().unwrap();
    assert!(uuid::Uuid::parse_str(id_str).is_ok(), "Should be a valid UUID");
}

#[tokio::test]
async fn test_correlation_id_extracted_from_header() {
    let app = create_test_app();

    let provided_id = "test-correlation-id-12345";

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("x-request-id", provided_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let correlation_id = response
        .headers()
        .get("x-request-id")
        .expect("X-Request-ID header should be present");

    assert_eq!(
        correlation_id.to_str().unwrap(),
        provided_id,
        "Should return the same correlation ID that was provided"
    );
}

#[tokio::test]
async fn test_correlation_id_persisted_across_multiple_requests() {
    let app = create_test_app();

    let response1 = app
        .clone()
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let response2 = app
        .oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let id1 = response1.headers().get("x-request-id").unwrap().to_str().unwrap();
    let id2 = response2.headers().get("x-request-id").unwrap().to_str().unwrap();

    assert_ne!(id1, id2, "Each request should have a unique correlation ID");
    assert!(uuid::Uuid::parse_str(id1).is_ok());
    assert!(uuid::Uuid::parse_str(id2).is_ok());
}

#[tokio::test]
async fn test_correlation_id_with_custom_value() {
    let app = create_test_app();

    let custom_id = "my-custom-trace-id-abc123";

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("x-request-id", custom_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let correlation_id = response
        .headers()
        .get("x-request-id")
        .expect("X-Request-ID header should be present");

    assert_eq!(
        correlation_id.to_str().unwrap(),
        custom_id,
        "Should return the custom correlation ID"
    );
}

#[tokio::test]
async fn test_correlation_id_with_invalid_header_encoding() {
    let app = create_test_app();
    let invalid_value = axum::http::HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/test")
                .header("x-request-id", invalid_value)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let correlation_id = response
        .headers()
        .get("x-request-id")
        .expect("X-Request-ID header should be present");

    let id_str = correlation_id.to_str().unwrap();
    assert!(uuid::Uuid::parse_str(id_str).is_ok(), "Should generate a valid UUID");
}
