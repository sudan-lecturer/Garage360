use axum::{routing::get, Router};

use crate::modules::{auth, assets, bays, billing, customers, dvi, dashboard, hr, inventory, jobs, purchases, reports, settings, vehicles};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/feature-flags", get(auth::feature_flags))
        .nest("/auth", auth::routes())
        .merge(customers::routes())
        .merge(vehicles::routes())
        .merge(jobs::routes())
        .merge(bays::routes())
        .merge(inventory::routes())
        .merge(purchases::routes())
        .merge(billing::routes())
        .merge(dvi::routes())
        .merge(hr::routes())
        .merge(reports::routes())
        .merge(settings::routes())
        .merge(assets::routes())
        .merge(dashboard::routes::routes())
}

async fn health() -> &'static str {
    "OK"
}
