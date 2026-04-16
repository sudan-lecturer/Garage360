pub mod routes;

pub use routes::routes as tenant_routes;

use axum::Router;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", axum::routing::get(|| async { "Tenant API v1" }))
}
