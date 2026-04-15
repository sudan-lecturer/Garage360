pub mod tenants;
pub mod feature_flags;

use axum::routing::get;
use axum::Router;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(|| async { "Control Plane API v1" }))
        .nest("/tenants", tenants::routes())
        .nest("/feature-flags", feature_flags::routes())
}
