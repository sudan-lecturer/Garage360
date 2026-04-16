mod config;
mod db;
mod errors;
mod middleware;
mod modules;

use axum::{
    body::Body,
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use redis::AsyncCommands;

use crate::config::AppConfig;
use crate::db::control::DbPool;
use crate::db::tenant::TenantPoolRegistry;
use crate::middleware::{handle_rejection, log_request};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: DbPool,
    pub redis: Arc<redis::aio::ConnectionManager>,
    pub tenant_registry: Arc<TenantPoolRegistry>,
}

#[utoipa::path(
    get,
    path = "/health/liveness",
    responses(
        (status = 200, description = "Service is alive")
    )
)]
async fn liveness() -> &'static str {
    "OK"
}

#[utoipa::path(
    get,
    path = "/health/readiness",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service is not ready")
    )
)]
async fn readiness(state: axum::extract::State<AppState>) -> Result<&'static str, errors::AppError> {
    sqlx::query("SELECT 1")
        .fetch_one(&state.db)
        .await
        .map_err(|_| errors::AppError::DatabaseUnavailable)?;

    Ok("OK")
}

fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health/liveness", get(liveness))
        .route("/health/readiness", get(readiness))
        .nest("/api/v1/auth", modules::auth::routes())
        .nest("/control/v1", modules::control::routes())
        .layer(axum_middleware::from_fn(log_request))
        .layer(axum_middleware::from_fn_with_state(state.clone(), handle_rejection))
        .layer(cors)
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "garage360_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = AppConfig::load()?;
    let db = db::control::create_pool(&config.database_url).await?;
    let redis = db::redis::create_client(&config.redis_url).await?;

    let state = AppState {
        config,
        db,
        redis: Arc::new(redis),
        tenant_registry: Arc::new(TenantPoolRegistry::default()),
    };

    let app = create_app(state.clone());

    let addr = format!("0.0.0.0:{}", state.config.app_port);
    let listener = TcpListener::bind(&addr).await?;

    info!("Garage360 API starting on {}", addr);
    info!("Health check: http://{}/health/liveness", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::Request,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_liveness_returns_ok() {
        let app = Router::new()
            .route("/health/liveness", get(liveness));

        let req = Request::builder()
            .uri("/health/liveness")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), 4096).await.unwrap();
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn test_create_app_registers_health_routes() {
        let config = AppConfig {
            database_url: "postgres://localhost/test".to_string(),
            redis_url: "redis://localhost".to_string(),
            app_port: 8080,
            jwt_secret: "test-secret".to_string(),
            jwt_expiry_hours: 1,
            jwt_refresh_expiry_days: 7,
            minio_endpoint: "localhost:9000".to_string(),
            minio_access_key: "key".to_string(),
            minio_secret_key: "key".to_string(),
            cors_origins: vec![],
            app_env: "test".to_string(),
        };

        let db = db::control::create_pool(&config.database_url).await.unwrap();
        let redis = db::redis::create_client(&config.redis_url).await.unwrap();

        let state = AppState {
            config,
            db,
            redis: Arc::new(redis),
            tenant_registry: Arc::new(TenantPoolRegistry::default()),
        };

        let app = create_app(state);

        let liveness_req = Request::builder()
            .uri("/health/liveness")
            .body(Body::empty())
            .unwrap();
        let liveness_resp = app.clone().oneshot(liveness_req).await.unwrap();
        assert_eq!(liveness_resp.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_app_state_clone() {
        let config = AppConfig {
            database_url: "postgres://localhost/test".to_string(),
            redis_url: "redis://localhost".to_string(),
            app_port: 8080,
            jwt_secret: "test-secret".to_string(),
            jwt_expiry_hours: 1,
            jwt_refresh_expiry_days: 7,
            minio_endpoint: "localhost:9000".to_string(),
            minio_access_key: "key".to_string(),
            minio_secret_key: "key".to_string(),
            cors_origins: vec![],
            app_env: "test".to_string(),
        };

        let db = db::control::create_pool(&config.database_url).await.unwrap();
        let redis = db::redis::create_client(&config.redis_url).await.unwrap();

        let state = AppState {
            config: config.clone(),
            db: db.clone(),
            redis: Arc::new(redis.clone()),
            tenant_registry: Arc::new(TenantPoolRegistry::default()),
        };

        let _cloned = state.clone();
    }

    #[tokio::test]
    async fn test_app_routes_include_auth_endpoints() {
        let config = AppConfig {
            database_url: "postgres://localhost/test".to_string(),
            redis_url: "redis://localhost".to_string(),
            app_port: 8080,
            jwt_secret: "test-secret".to_string(),
            jwt_expiry_hours: 1,
            jwt_refresh_expiry_days: 7,
            minio_endpoint: "localhost:9000".to_string(),
            minio_access_key: "key".to_string(),
            minio_secret_key: "key".to_string(),
            cors_origins: vec![],
            app_env: "test".to_string(),
        };

        let db = db::control::create_pool(&config.database_url).await.unwrap();
        let redis = db::redis::create_client(&config.redis_url).await.unwrap();

        let state = AppState {
            config,
            db,
            redis: Arc::new(redis),
            tenant_registry: Arc::new(TenantPoolRegistry::default()),
        };

        let app = create_app(state);

        let login_req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .body(Body::empty())
            .unwrap();
        let login_resp = app.clone().oneshot(login_req).await.unwrap();
        assert_eq!(login_resp.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}
