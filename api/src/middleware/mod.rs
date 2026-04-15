pub mod auth;
pub mod rbac;
pub mod tenant;
pub mod feature_flags;

pub use auth::{AuthUser, Claims, JwtService};
pub use tenant::TenantDbPool;
pub use rbac::{require_auth, require_role, RequireRole};

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
        
        if parts.headers.get(header::CONTENT_TYPE).map(|v| v.to_str().unwrap_or("")).contains("json") == Some(true) {
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
