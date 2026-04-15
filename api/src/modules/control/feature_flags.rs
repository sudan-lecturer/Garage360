use axum::{
    extract::{State, Extension},
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::AppState;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(list_feature_flags))
        .route("/", put(set_feature_flag))
        .route("/:key", put(set_feature_flag))
}

#[derive(Debug, Serialize)]
pub struct FeatureFlagResponse {
    pub id: String,
    pub key: String,
    pub description: String,
    pub default_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct TenantOverrideResponse {
    pub tenant_id: String,
    pub feature_flag_id: String,
    pub is_enabled: bool,
}

pub async fn list_feature_flags(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<FeatureFlagResponse>>> {
    if user.role != "SUPER_ADMIN" {
        return Err(AppError::Forbidden("Super admin access required".into()));
    }

    let flags = sqlx::query_as::<_, FeatureFlagRow>(
        r#"
        SELECT id, key, description, default_enabled, created_at
        FROM feature_flags
        ORDER BY key
        "#,
    )
    .fetch_all(&*state.db)
    .await
    .map_err(|e| AppError::Database(e))?;

    let response: Vec<FeatureFlagResponse> = flags
        .into_iter()
        .map(|f| FeatureFlagResponse {
            id: f.id,
            key: f.key,
            description: f.description,
            default_enabled: f.default_enabled,
            created_at: f.created_at,
        })
        .collect();

    Ok(Json(response))
}

pub async fn set_feature_flag(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    axum::extract::Path(key): axum::extract::Path<String>,
    Json(req): Json<SetFeatureFlagRequest>,
) -> AppResult<Json<FeatureFlagResponse>> {
    if user.role != "SUPER_ADMIN" {
        return Err(AppError::Forbidden("Super admin access required".into()));
    }

    let result = sqlx::query_as::<_, FeatureFlagRow>(
        r#"
        INSERT INTO feature_flags (id, key, description, default_enabled)
        VALUES (gen_random_uuid(), $1, $2, $3)
        ON CONFLICT (key) DO UPDATE
        SET description = $2, default_enabled = $3, updated_at = NOW()
        RETURNING id, key, description, default_enabled, created_at
        "#,
    )
    .bind(&key)
    .bind(&req.description)
    .bind(&req.default_enabled)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| AppError::Database(e))?;

    Ok(Json(FeatureFlagResponse {
        id: result.id,
        key: result.key,
        description: result.description,
        default_enabled: result.default_enabled,
        created_at: result.created_at,
    }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct SetFeatureFlagRequest {
    pub description: Option<String>,
    pub default_enabled: bool,
}

struct FeatureFlagRow {
    id: String,
    key: String,
    description: String,
    default_enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}
