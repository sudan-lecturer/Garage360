pub mod routes;

pub use routes::routes;

use axum::Router;

pub fn routes() -> Router {
    Router::new()
        .route("/health", axum::routing::get(|| async { "Tenant API v1" }))
}
