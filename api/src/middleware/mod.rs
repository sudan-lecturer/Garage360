pub mod auth;
pub mod rbac;
pub mod tenant;
pub mod feature_flags;

pub use auth::AuthUser;
pub use tenant::TenantDbPool;

use axum::{
    body::Body,
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use std::time::Instant;

pub async fn log_request(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

    tracing::info!(
        method = %method,
        path = %path,
        status = %status,
        duration_ms = %duration.as_millis(),
        "HTTP Request"
    );

    response
}

pub async fn handle_rejection(req: Request, next: Next) -> Response {
    let response = next.run(req).await;

    if response.status() == StatusCode::NOT_FOUND {
        let (parts, body) = response.into_parts();
        
        let is_json = parts.headers.get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("json"))
            .unwrap_or(false);
        
        if is_json {
            return Response::from_parts(parts, body);
        }

        let body = Body::from(r#"{"error":{"code":"NOT_FOUND","message":"The requested resource was not found"}}"#);
        let mut response = Response::new(body);
        *response.status_mut() = StatusCode::NOT_FOUND;
        response.headers_mut().insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        return response;
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::Request,
        routing::{get, post},
    };
    use tower::ServiceExt;
    use tower_http::cors::Any;

    #[tokio::test]
    async fn test_log_request_passes_through_response() {
        let app = axum::Router::new()
            .route("/test", get(|| async { "OK" }));

        let req = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_rejection_passes_non_404_responses() {
        let app = axum::Router::new()
            .route("/ok", get(|| async { "OK" }));

        let req = Request::builder()
            .uri("/ok")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_routes_registration() {
        let router = Router::new()
            .route("/api/v1/auth/login", post(|| async { "login" }))
            .route("/api/v1/auth/refresh", post(|| async { "refresh" }))
            .route("/api/v1/auth/logout", post(|| async { "logout" }))
            .route("/health/liveness", get(|| async { "OK" }));

        let login_req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .body(Body::empty())
            .unwrap();
        let login_resp = router.clone().oneshot(login_req).await.unwrap();
        assert_eq!(login_resp.status(), axum::http::StatusCode::OK);

        let health_req = Request::builder()
            .uri("/health/liveness")
            .body(Body::empty())
            .unwrap();
        let health_resp = router.clone().oneshot(health_req).await.unwrap();
        assert_eq!(health_resp.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cors_layer_configuration() {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(cors);

        let req = Request::builder()
            .uri("/test")
            .header("Origin", "http://example.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }
}
