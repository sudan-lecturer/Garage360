use axum::{
    extract::{State, Extension},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;
use redis::AsyncCommands;
use argon2::{Argon2, PasswordHasher, password_hash::{PasswordHash, PasswordVerifier, SaltString}};
use rand::thread_rng;

use crate::errors::{AppError, AppResult};
use crate::middleware::auth::{AuthUser, JwtService};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/change-password", post(change_password))
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
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let user_row = sqlx::query_as::<sqlx::Postgres, UserRow>(
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

    let valid = PasswordHash::new(&user_row.password_hash)
        .and_then(|hash| {
            PasswordVerifier::verify_password(
                &Argon2::default(),
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
    .execute(&state.db)
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
    .fetch_optional(&state.db)
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
    Json(_req): Json<RefreshRequest>,
) -> AppResult<Json<()>> {
    // TODO: Implement token blocklist in redis
    // let mut conn = state.redis.clone();
    // conn.set_ex(format!("blocklist:{}", _req.refresh_token), "1", 86400 * 7)
    //     .await
    //     .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

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

    let salt = SaltString::generate(&mut thread_rng());
    let password_hash = Argon2::default()
        .hash_password(req.new_password.as_bytes(), &salt)
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
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(e))?;

    Ok(Json(()))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    email: String,
    password_hash: String,
    name: String,
    role: String,
    tenant_id: String,
    tenant_active: bool,
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
        let result = req.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_login_request_empty_email() {
        let req = LoginRequest {
            email: "".to_string(),
            password: "password123".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_login_request_empty_password() {
        let req = LoginRequest {
            email: "test@example.com".to_string(),
            password: "".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_login_request_both_empty() {
        let req = LoginRequest {
            email: "".to_string(),
            password: "".to_string(),
        };
        let result = req.validate();
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("Email") || err_str.contains("email"));
    }

    #[test]
    fn test_login_request_email_without_at_symbol() {
        let req = LoginRequest {
            email: "testexample.com".to_string(),
            password: "password123".to_string(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn test_login_request_email_without_domain() {
        let req = LoginRequest {
            email: "test@".to_string(),
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
    fn test_change_password_request_serialization() {
        let req = ChangePasswordRequest {
            current_password: "oldpass".to_string(),
            new_password: "newpass123".to_string(),
            confirm_password: "newpass123".to_string(),
        };
        let json = serde_json::to_string(&req).expect("should serialize");
        assert!(json.contains("oldpass"));
        assert!(json.contains("newpass123"));
    }

    #[test]
    fn test_change_password_passwords_must_match_logic() {
        let req = ChangePasswordRequest {
            current_password: "oldpass".to_string(),
            new_password: "newpass123".to_string(),
            confirm_password: "different".to_string(),
        };
        assert_ne!(req.new_password, req.confirm_password);
    }

    #[test]
    fn test_login_response_serialization() {
        let resp = LoginResponse {
            access_token: "access-token-abc".to_string(),
            refresh_token: "refresh-token-xyz".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: UserResponse {
                id: "user-1".to_string(),
                email: "user@example.com".to_string(),
                name: "Test User".to_string(),
                role: "ADMIN".to_string(),
                tenant_id: "tenant-1".to_string(),
            },
        };
        let json = serde_json::to_string(&resp).expect("should serialize");
        assert!(json.contains("access-token-abc"));
        assert!(json.contains("Bearer"));
        assert!(json.contains("user@example.com"));
        assert!(json.contains("ADMIN"));
    }

    #[test]
    fn test_user_response_serialization() {
        let user = UserResponse {
            id: "user-123".to_string(),
            email: "john@example.com".to_string(),
            name: "John Doe".to_string(),
            role: "MECHANIC".to_string(),
            tenant_id: "tenant-abc".to_string(),
        };
        let json = serde_json::to_string(&user).expect("should serialize");
        let parsed: UserResponse = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed.id, "user-123");
        assert_eq!(parsed.email, "john@example.com");
        assert_eq!(parsed.name, "John Doe");
        assert_eq!(parsed.role, "MECHANIC");
    }

    #[test]
    fn test_login_request_various_email_formats() {
        let valid_emails = vec![
            "user@example.com",
            "user.name@example.com",
            "user+tag@example.com",
            "user@subdomain.example.com",
        ];
        for email in valid_emails {
            let req = LoginRequest {
                email: email.to_string(),
                password: "password123".to_string(),
            };
            assert!(req.validate().is_ok(), "Expected {} to be valid", email);
        }
    }

    #[test]
    fn test_login_request_invalid_email_formats() {
        let invalid_emails = vec![
            "notanemail",
            "missing@",
            "@nodomain.com",
            "spaces in@email.com",
            "",
        ];
        for email in invalid_emails {
            let req = LoginRequest {
                email: email.to_string(),
                password: "password123".to_string(),
            };
            let result = req.validate();
            assert!(result.is_err(), "Expected {} to be invalid", email);
        }
    }
}
