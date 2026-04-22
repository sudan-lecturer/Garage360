use axum::{routing::get, Router};

use crate::modules::{auth, bays, billing, customers, inventory, jobs, purchases, vehicles};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .nest("/auth", auth::routes())
        .merge(customers::routes())
        .merge(vehicles::routes())
        .merge(jobs::routes())
        .merge(bays::routes())
        .merge(inventory::routes())
        .merge(purchases::routes())
        .merge(billing::routes())
}

async fn health() -> &'static str {
    "OK"
}
