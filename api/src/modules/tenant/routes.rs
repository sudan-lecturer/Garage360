use axum::{routing::get, Router};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
}

async fn health() -> &'static str {
    "OK"
}
