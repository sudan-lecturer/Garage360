use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis unavailable")]
    RedisUnavailable,

    #[error("Database unavailable")]
    DatabaseUnavailable,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("argon2 error: {0}")]
    Argon2(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "A database error occurred",
                )
            }
            AppError::RedisUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "REDIS_UNAVAILABLE",
                "Cache service is unavailable",
            ),
            AppError::DatabaseUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "DATABASE_UNAVAILABLE",
                "Database is unavailable",
            ),
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg.as_str(),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                msg.as_str(),
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                msg.as_str(),
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg.as_str(),
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                msg.as_str(),
            ),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An internal error occurred",
                )
            }
            AppError::Jwt(e) => {
                tracing::warn!("JWT error: {:?}", e);
                (
                    StatusCode::UNAUTHORIZED,
                    "JWT_ERROR",
                    "Invalid or expired token",
                )
            }
            AppError::Argon2(e) => {
                tracing::warn!("Argon2 error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "HASH_ERROR",
                    "Password processing error",
                )
            }
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
            }
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_error_database_returns_500() {
        let err = AppError::Database(sqlx::Error::RowNotFound);
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_error_validation_returns_400() {
        let err = AppError::Validation("Email is required".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_error_unauthorized_returns_401() {
        let err = AppError::Unauthorized("Invalid token".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_error_forbidden_returns_403() {
        let err = AppError::Forbidden("Access denied".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_error_not_found_returns_404() {
        let err = AppError::NotFound("User not found".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_error_conflict_returns_409() {
        let err = AppError::Conflict("Resource already exists".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_error_internal_returns_500() {
        let err = AppError::Internal("Something went wrong".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_error_redis_unavailable_returns_503() {
        let err = AppError::RedisUnavailable;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_error_database_unavailable_returns_503() {
        let err = AppError::DatabaseUnavailable;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_error_response_body_contains_code_and_message() {
        use axum::body::to_bytes;
        use std::str::from_utf8;

        let err = AppError::Unauthorized("Invalid credentials".into());
        let resp = err.into_response();
        let status = resp.status();
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json_str = from_utf8(&body).unwrap();

        assert!(json_str.contains("UNAUTHORIZED"));
        assert!(json_str.contains("Invalid credentials"));
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_error_validation_response_body() {
        use axum::body::to_bytes;
        use std::str::from_utf8;

        let err = AppError::Validation("Email is required".into());
        let resp = err.into_response();
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json_str = from_utf8(&body).unwrap();

        assert!(json_str.contains("VALIDATION_ERROR"));
        assert!(json_str.contains("Email is required"));
    }
}
