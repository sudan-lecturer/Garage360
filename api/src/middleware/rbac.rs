use axum::{
    body::Body,
    extract::Extension,
    http::Request,
    middleware::Next,
    response::Response,
};

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;

pub async fn require_auth(
    Extension(user): Extension<AuthUser>,
    request: Request<Body>,
    next: Next,
) -> Response {
    next.run(request).await
}

pub async fn require_role(
    Extension(user): Extension<AuthUser>,
    roles: Vec<String>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    if !roles.contains(&user.role) {
        return Err(AppError::Forbidden(format!(
            "Role '{}' is not authorized for this action",
            user.role
        )));
    }
    Ok(next.run(request).await)
}
