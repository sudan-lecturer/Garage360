pub mod tenants;
pub mod feature_flags;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { "Control Plane API v1" }))
        .nest("/tenants", tenants::routes())
        .nest("/feature-flags", feature_flags::routes())
}
