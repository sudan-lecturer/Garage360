use axum::{routing::get, Json, Router};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/jobs", get(index))
}

async fn index() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "module": "jobs",
        "status": "scaffolded"
    }))
}
