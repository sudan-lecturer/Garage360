use axum::{
    extract::{State, Extension},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::auth::{AuthUser, JwtService};
use crate::AppState;

pub fn routes() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/change-password", post(change_password))
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email"))]
    #[validate(length(min = 1, message = "Email is required"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
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
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let user_row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT u.id, u.email, u.password_hash, u.name, u.role, u.tenant_id, t.is_active as tenant_active
        FROM users u
        JOIN tenants t ON u.tenant_id = t.id
        WHERE u.email = $1 AND u.is_active = true
        "#,
    )
    .bind(&req.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

    if !user_row.tenant_active {
        return Err(AppError::Unauthorized("Tenant is inactive".into()));
    }

    let valid = argon2::password_hash::PasswordHash::new(&user_row.password_hash)
        .and_then(|hash| {
            argon2::password_hash::PasswordVerifier::verify_password(
                &argon2::Argon2::default(),
                req.password.as_bytes(),
                &hash,
            )
        })
        .is_ok();

    if !valid {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    let jwt_service = JwtService::new(&state.config.jwt_secret);
    let access_token = jwt_service.create_access_token(
        &user_row.id,
        &user_row.tenant_id,
        &user_row.role,
        state.config.jwt_expiry_hours,
    )?;
    let refresh_token = jwt_service.create_refresh_token(
        &user_row.id,
        &user_row.tenant_id,
        &user_row.role,
        state.config.jwt_refresh_expiry_days,
    )?;

    sqlx::query(
        r#"
        INSERT INTO audit_logs (tenant_id, user_id, action, entity_type, entity_id, metadata)
        VALUES ($1, $2, 'LOGIN', 'user', $3, $4)
        "#,
    )
    .bind(&user_row.tenant_id)
    .bind(&user_row.id)
    .bind(&user_row.id)
    .bind(serde_json::json!({ "ip": "unknown" }).to_string())
    .execute(&*state.db)
    .await
    .ok();

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: state.config.jwt_expiry_hours * 3600,
        user: UserResponse {
            id: user_row.id,
            email: user_row.email,
            name: user_row.name,
            role: user_row.role,
            tenant_id: user_row.tenant_id,
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

    let user_row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, name, role, tenant_id
        FROM users
        WHERE id = $1 AND is_active = true
        "#,
    )
    .bind(&claims.sub)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    let access_token = jwt_service.create_access_token(
        &user_row.id,
        &user_row.tenant_id,
        &user_row.role,
        state.config.jwt_expiry_hours,
    )?;
    let refresh_token = jwt_service.create_refresh_token(
        &user_row.id,
        &user_row.tenant_id,
        &user_row.role,
        state.config.jwt_refresh_expiry_days,
    )?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".into(),
        expires_in: state.config.jwt_expiry_hours * 3600,
        user: UserResponse {
            id: user_row.id,
            email: user_row.email,
            name: user_row.name,
            role: user_row.role,
            tenant_id: user_row.tenant_id,
        },
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<()>> {
    let _: Result<(), _> = state
        .redis
        .clone()
        .set_ex(format!("blocklist:{}", req.refresh_token), "1", 86400 * 7)
        .await;

    Ok(Json(()))
}

pub async fn me(Extension(user): Extension<AuthUser>) -> Json<UserResponse> {
    Json(UserResponse {
        id: user.user_id,
        email: "".into(),
        name: "".into(),
        role: user.role,
        tenant_id: user.tenant_id,
    })
}

pub async fn change_password(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<()>> {
    if req.new_password != req.confirm_password {
        return Err(AppError::Validation("Passwords do not match".into()));
    }

    let password_hash = argon2::password_hash::PasswordHash::new(
        argon2::password_hash::SaltString::generate(&mut rand::thread_rng()).as_str(),
    )
    .and_then(|salt| argon2::Argon2::default().hash_password(req.new_password.as_bytes(), &salt))
    .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
    .to_string();

    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(&password_hash)
    .bind(&user.user_id)
    .execute(&*state.db)
    .await
    .map_err(|e| AppError::Database(e))?;

    Ok(Json(()))
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

struct UserRow {
    id: String,
    email: String,
    password_hash: String,
    name: String,
    role: String,
    tenant_id: String,
    tenant_active: bool,
}
