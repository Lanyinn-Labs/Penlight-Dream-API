use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

use crate::crypto::CryptoError;
use crate::proto::decoder::ProtoError;

/// Application-wide error type mapped to JSON HTTP responses.
///
/// Response bodies use the envelope:
/// `{ "result": "failed", "status": <http status>, "message": <human readable message> }`.
#[derive(Debug, Clone, Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    /// Missing or invalid API key.
    #[error("{0}")]
    Unauthorized(String),
    /// The official Garupa API returned a non-2xx status.
    #[error("upstream request failed (HTTP {0})")]
    Upstream(u16),
    /// Upstream request timed out or failed at the transport level.
    #[error("{0}")]
    UpstreamError(String),
    #[error("{0}")]
    Crypto(String),
    #[error("{0}")]
    Proto(String),
    #[error("{0}")]
    Json(String),
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn not_found(message: impl Into<String>) -> Self {
        AppError::NotFound(message.into())
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        AppError::BadRequest(message.into())
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        AppError::Unauthorized(message.into())
    }
}

impl From<CryptoError> for AppError {
    fn from(e: CryptoError) -> Self {
        AppError::Crypto(e.to_string())
    }
}

impl From<ProtoError> for AppError {
    fn from(e: ProtoError) -> Self {
        AppError::Proto(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            AppError::UpstreamError("upstream request timed out".to_string())
        } else {
            AppError::UpstreamError(format!("upstream request failed: {e}"))
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m),
            AppError::Upstream(code) => (StatusCode::BAD_GATEWAY, format!("Upstream request failed (HTTP {code})")),
            AppError::UpstreamError(m) => (StatusCode::BAD_GATEWAY, m),
            AppError::Crypto(m) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Decryption error: {m}")),
            AppError::Proto(m) => (StatusCode::BAD_GATEWAY, format!("Protobuf parse error: {m}")),
            AppError::Json(m) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization error: {m}")),
            AppError::Io(m) => (StatusCode::INTERNAL_SERVER_ERROR, format!("IO error: {m}")),
            AppError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
        };

        let body = json!({ "result": "failed", "status": status.as_u16(), "message": message });
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
