use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::auth::{AuthUser, JwtService};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email"))]
    #[validate(length(min = 1, message = "Email is required"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub tenant_id: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    let user = sqlx::query_as::<_, SuperAdminUserRow>(
        r#"
        SELECT id, email, password_hash, name, is_active
        FROM super_admin_users
        WHERE email = $1
        "#,
    )
    .bind(&req.email)
    .fetch_optional(&state.control_db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

    if !user.is_active {
        return Err(AppError::Unauthorized("User is inactive".into()));
    }

    let password_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &password_hash)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?;

    let jwt_service = JwtService::new(&state.config.jwt_secret);
    let access_token = jwt_service.create_access_token(
        &user.id,
        CONTROL_TENANT_ID,
        SUPER_ADMIN_ROLE,
        state.config.jwt_expiry_hours,
    )?;
    let refresh_token = jwt_service.create_refresh_token(
        &user.id,
        CONTROL_TENANT_ID,
        SUPER_ADMIN_ROLE,
        state.config.jwt_refresh_expiry_days,
    )?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: state.config.jwt_expiry_hours * 3600,
        user: UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: SUPER_ADMIN_ROLE.into(),
            tenant_id: CONTROL_TENANT_ID.into(),
        },
    }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<LoginResponse>> {
    let jwt_service = JwtService::new(&state.config.jwt_secret);
    let claims = jwt_service.decode(&req.refresh_token)?;

    if claims.kid != "refresh" {
        return Err(AppError::Unauthorized("Invalid refresh token".into()));
    }

    let user = sqlx::query_as::<_, RefreshUserRow>(
        r#"
        SELECT id, email, name, is_active
        FROM super_admin_users
        WHERE id = $1
        "#,
    )
    .bind(&claims.sub)
    .fetch_optional(&state.control_db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    if !user.is_active {
        return Err(AppError::Unauthorized("User is inactive".into()));
    }

    let access_token = jwt_service.create_access_token(
        &user.id,
        CONTROL_TENANT_ID,
        SUPER_ADMIN_ROLE,
        state.config.jwt_expiry_hours,
    )?;
    let refresh_token = jwt_service.create_refresh_token(
        &user.id,
        CONTROL_TENANT_ID,
        SUPER_ADMIN_ROLE,
        state.config.jwt_refresh_expiry_days,
    )?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: state.config.jwt_expiry_hours * 3600,
        user: UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: SUPER_ADMIN_ROLE.into(),
            tenant_id: CONTROL_TENANT_ID.into(),
        },
    }))
}

pub async fn logout() -> AppResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn me(auth_user: AuthUser) -> Json<UserResponse> {
    Json(UserResponse {
        id: auth_user.user_id,
        email: String::new(),
        name: String::new(),
        role: auth_user.role,
        tenant_id: auth_user.tenant_id,
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(sqlx::FromRow)]
struct SuperAdminUserRow {
    id: String,
    email: String,
    password_hash: String,
    name: String,
    is_active: bool,
}

#[derive(sqlx::FromRow)]
struct RefreshUserRow {
    id: String,
    email: String,
    name: String,
    is_active: bool,
}

const CONTROL_TENANT_ID: &str = "control";
const SUPER_ADMIN_ROLE: &str = "SUPER_ADMIN";

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_login_request_valid_email_and_password() {
        let req = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_login_request_invalid_email_format() {
        let req = LoginRequest {
            email: "not-an-email".to_string(),
            password: "password123".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_refresh_request_deserialization() {
        let json = r#"{"refresh_token": "eyJhbGciOiJIUzI1NiJ9.test"}"#;
        let req: RefreshRequest = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(req.refresh_token, "eyJhbGciOiJIUzI1NiJ9.test");
    }
}
