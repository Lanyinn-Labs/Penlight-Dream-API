//! Optional API-key authentication middleware.
//!
//! When `API_KEY` is configured, every `/api/*` request must present the key
//! either in the `X-API-Key` header or as `Authorization: Bearer <key>`.
//! With an empty key the middleware is a no-op.

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::api::SharedState;
use crate::error::{AppError, AppResult};

/// Compares two strings in time independent of their contents using a
/// byte-wise XOR fold. String lengths still leak, which is acceptable for an
/// API key.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub async fn require_api_key(State(state): State<SharedState>, request: Request, next: Next) -> AppResult<Response> {
    let expected = state.config.api_key.as_str();
    if expected.is_empty() {
        return Ok(next.run(request).await);
    }

    let provided = request.headers().get("X-API-Key").and_then(|v| v.to_str().ok()).or_else(|| {
        request
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|auth| auth.strip_prefix("Bearer "))
            .map(str::trim)
    });

    if provided.is_some_and(|provided| constant_time_eq(provided, expected)) {
        Ok(next.run(request).await)
    } else {
        Err(AppError::unauthorized("invalid or missing API key"))
    }
}
