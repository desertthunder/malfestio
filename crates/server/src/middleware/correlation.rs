use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use tracing::Instrument;
use uuid::Uuid;

/// Header name for correlation/request ID.
///
/// Follows the de facto standard used by many HTTP clients and proxies.
const CORRELATION_HEADER: &str = "x-request-id";

/// Extension key to store the correlation ID in the request.
#[derive(Debug, Clone)]
pub struct CorrelationId(pub String);

/// Extract or generate a correlation ID from request headers.
///
/// First checks for existing `X-Request-ID` header, then generates a new UUID v4 if not present.
/// This enables distributed tracing across multiple services.
fn extract_correlation_id(headers: &HeaderMap) -> String {
    headers
        .get(CORRELATION_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

/// Middleware that adds correlation ID to all requests.
///
/// This middleware:
/// 1. Extracts or generates a correlation ID
/// 2. Adds it to the tracing span for this request
/// 3. Stores it in request extensions for handler access
/// 4. Returns it in the response header for client tracking
///
/// The correlation ID flows through the entire request lifecycle and appears in all logs,
/// enabling traceability across distributed systems.
pub async fn correlation_middleware(mut req: Request, next: Next) -> Response {
    let correlation_id = extract_correlation_id(req.headers());
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    req.extensions_mut().insert(CorrelationId(correlation_id.clone()));

    let span = tracing::info_span!(
        "request",
        correlation_id = %correlation_id,
        method = %method,
        uri = %uri,
    );

    async move {
        let mut response = next.run(req).await;

        response
            .headers_mut()
            .insert(CORRELATION_HEADER, correlation_id.parse().expect("valid header value"));
        response
    }
    .instrument(span)
    .await
}

/// Helper to extract the correlation ID from request extensions.
///
/// Use this in handlers to access the current request's correlation ID.
pub fn get_correlation_id(headers: &HeaderMap) -> String {
    extract_correlation_id(headers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_correlation_id_from_header() {
        let mut headers = HeaderMap::new();
        headers.insert(CORRELATION_HEADER, HeaderValue::from_static("test-id-123"));

        let id = extract_correlation_id(&headers);
        assert_eq!(id, "test-id-123");
    }

    #[test]
    fn test_extract_correlation_id_generates_new() {
        let headers = HeaderMap::new();
        let id1 = extract_correlation_id(&headers);
        let id2 = extract_correlation_id(&headers);

        assert!(Uuid::parse_str(&id1).is_ok());
        assert!(Uuid::parse_str(&id2).is_ok());
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_extract_correlation_id_invalid_header_fallback() {
        let mut headers = HeaderMap::new();
        headers.insert(CORRELATION_HEADER, HeaderValue::from_bytes(&[0xFF, 0xFE]).unwrap());

        let id = extract_correlation_id(&headers);
        assert!(Uuid::parse_str(&id).is_ok());
    }
}
