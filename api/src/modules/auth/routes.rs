use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::auth::{AuthUser, JwtService};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/change-password", post(change_password))
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
        SELECT id::text AS id, email, password_hash, name, is_active
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
        SELECT id::text AS id, email, name, is_active
        FROM super_admin_users
        WHERE id = $1::uuid
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

pub async fn change_password(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    if req.current_password == req.new_password {
        return Err(AppError::Validation(
            "New password must be different from the current password".into(),
        ));
    }

    if auth_user.tenant_id == CONTROL_TENANT_ID {
        let user = sqlx::query_as::<_, PasswordRow>(
            r#"
            SELECT password_hash, is_active
            FROM super_admin_users
            WHERE id = $1::uuid
            "#,
        )
        .bind(&auth_user.user_id)
        .fetch_optional(&state.control_db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

        ensure_user_is_active(user.is_active)?;
        verify_password(&req.current_password, &user.password_hash)?;

        sqlx::query(
            r#"
            UPDATE super_admin_users
            SET password_hash = $2, updated_at = NOW()
            WHERE id = $1::uuid
            "#,
        )
        .bind(&auth_user.user_id)
        .bind(hash_password(&req.new_password)?)
        .execute(&state.control_db)
        .await
        .map_err(AppError::Database)?;
    } else {
        let tenant_pool = get_tenant_pool(&state, &auth_user.tenant_id).await?;
        let user = sqlx::query_as::<_, PasswordRow>(
            r#"
            SELECT password_hash, is_active
            FROM users
            WHERE id = $1::uuid
            "#,
        )
        .bind(&auth_user.user_id)
        .fetch_optional(&tenant_pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

        ensure_user_is_active(user.is_active)?;
        verify_password(&req.current_password, &user.password_hash)?;

        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $2, updated_at = NOW()
            WHERE id = $1::uuid
            "#,
        )
        .bind(&auth_user.user_id)
        .bind(hash_password(&req.new_password)?)
        .execute(&tenant_pool)
        .await
        .map_err(AppError::Database)?;
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<Json<UserResponse>> {
    if auth_user.tenant_id == CONTROL_TENANT_ID {
        let user = sqlx::query_as::<_, SuperAdminProfileRow>(
            r#"
            SELECT id::text AS id, email, name, is_active
            FROM super_admin_users
            WHERE id = $1::uuid
            "#,
        )
        .bind(&auth_user.user_id)
        .fetch_optional(&state.control_db)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

        ensure_user_is_active(user.is_active)?;

        return Ok(Json(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: SUPER_ADMIN_ROLE.into(),
            tenant_id: CONTROL_TENANT_ID.into(),
        }));
    }

    let tenant_pool = get_tenant_pool(&state, &auth_user.tenant_id).await?;
    let user = sqlx::query_as::<_, TenantUserProfileRow>(
        r#"
        SELECT id::text AS id, email, name, role, is_active
        FROM users
        WHERE id = $1::uuid
        "#,
    )
    .bind(&auth_user.user_id)
    .fetch_optional(&tenant_pool)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    ensure_user_is_active(user.is_active)?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role,
        tenant_id: auth_user.tenant_id,
    }))
}

pub async fn feature_flags(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<Json<HashMap<String, bool>>> {
    if auth_user.tenant_id == CONTROL_TENANT_ID {
        let flags = sqlx::query_as::<_, (String, bool)>(
            r#"
            SELECT key, default_enabled
            FROM feature_flags
            ORDER BY key
            "#,
        )
        .fetch_all(&state.control_db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .collect();

        return Ok(Json(flags));
    }

    let flags = sqlx::query_as::<_, (String, bool)>(
        r#"
        SELECT ff.key, COALESCE(tfo.is_enabled, ff.default_enabled, false) AS enabled
        FROM feature_flags ff
        LEFT JOIN tenant_feature_flag_overrides tfo
            ON tfo.feature_flag_id = ff.id
           AND tfo.tenant_id = $1::uuid
        ORDER BY ff.key
        "#,
    )
    .bind(&auth_user.tenant_id)
    .fetch_all(&state.control_db)
        .await
        .map_err(AppError::Database)?
        .into_iter()
        .collect();

    Ok(Json(flags))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password is required"))]
    pub current_password: String,
    #[validate(length(min = 8, message = "New password must be at least 8 characters"))]
    pub new_password: String,
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

#[derive(sqlx::FromRow)]
struct SuperAdminProfileRow {
    id: String,
    email: String,
    name: String,
    is_active: bool,
}

#[derive(sqlx::FromRow)]
struct TenantUserProfileRow {
    id: String,
    email: String,
    name: String,
    role: String,
    is_active: bool,
}

#[derive(sqlx::FromRow)]
struct PasswordRow {
    password_hash: String,
    is_active: bool,
}

#[derive(sqlx::FromRow)]
struct TenantDatabaseRow {
    database_host: String,
    database_port: i32,
    database_name: String,
    database_username: String,
    database_password: String,
}

const CONTROL_TENANT_ID: &str = "control";
const SUPER_ADMIN_ROLE: &str = "SUPER_ADMIN";

fn ensure_user_is_active(is_active: bool) -> AppResult<()> {
    if is_active {
        Ok(())
    } else {
        Err(AppError::Unauthorized("User is inactive".into()))
    }
}

fn verify_password(password: &str, password_hash: &str) -> AppResult<()> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| AppError::Unauthorized("Invalid credentials".into()))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("Current password is incorrect".into()))
}

fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| AppError::Argon2(err.to_string()))
}

async fn get_tenant_pool(state: &AppState, tenant_id: &str) -> AppResult<sqlx::PgPool> {
    let tenant = sqlx::query_as::<_, TenantDatabaseRow>(
        r#"
        SELECT database_host, database_port, database_name, database_username, database_password
        FROM tenants
        WHERE id = $1::uuid AND is_active = true
        "#,
    )
    .bind(tenant_id)
    .fetch_optional(&state.control_db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::Unauthorized("Invalid tenant context".into()))?;

    let tenant_db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        tenant.database_username,
        tenant.database_password,
        tenant.database_host,
        tenant.database_port,
        tenant.database_name,
    );

    state
        .tenant_registry
        .get_pool(tenant_id, &tenant_db_url)
        .await
        .map_err(|err| AppError::Internal(format!("Failed to connect to tenant database: {err}")))
}

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

    #[test]
    fn test_change_password_request_requires_new_password_length() {
        let req = ChangePasswordRequest {
            current_password: "current-password".to_string(),
            new_password: "short".to_string(),
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_hash_password_creates_verifiable_argon2_hash() {
        let password = "new-password-123";
        let hash = hash_password(password).expect("should hash password");
        let parsed = PasswordHash::new(&hash).expect("should parse hash");

        assert!(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok());
    }
}
