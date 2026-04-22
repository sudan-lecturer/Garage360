pub mod auth;
pub mod feature_flags;
pub mod rbac;
pub mod tenant;

use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
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

    tracing::info!(
        method = %method,
        path = %path,
        status = %response.status(),
        duration_ms = duration.as_millis(),
        "http_request"
    );

    response
}

pub async fn handle_rejection(req: Request, next: Next) -> Response {
    let response = next.run(req).await;

    if response.status() != StatusCode::NOT_FOUND {
        return response;
    }

    let (parts, body) = response.into_parts();
    let is_json = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("json"))
        .unwrap_or(false);

    if is_json {
        return Response::from_parts(parts, body);
    }

    let body = Body::from(
        r#"{"error":{"code":"NOT_FOUND","message":"The requested resource was not found"}}"#,
    );
    let mut response = Response::new(body);
    *response.status_mut() = StatusCode::NOT_FOUND;
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, middleware, routing::get, Router};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_handle_rejection_wraps_404s_as_json() {
        let app = Router::new().layer(middleware::from_fn(handle_rejection));

        let req = Request::builder().uri("/missing").body(Body::empty()).unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn test_log_request_passes_through_response() {
        let app = Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(middleware::from_fn(log_request));

        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let response = app.oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
