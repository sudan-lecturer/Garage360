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
        .fetch_one(&*state.db)
        .await
        .map_err(|_| errors::AppError::DatabaseUnavailable)?;

    let _: Result<(), _> = state
        .redis
        .clone()
        .ping()
        .await
        .map_err(|_| errors::AppError::RedisUnavailable);

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
