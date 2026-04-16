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

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_feature_flags))
        .route("/", put(set_feature_flag))
        .route("/:key", put(set_feature_flag))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureFlagResponse {
    pub id: String,
    pub key: String,
    pub description: String,
    pub default_enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    .fetch_all(&state.db)
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
    .fetch_one(&state.db)
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

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct SetFeatureFlagRequest {
    pub description: Option<String>,
    pub default_enabled: bool,
}

#[derive(sqlx::FromRow)]
struct FeatureFlagRow {
    id: String,
    key: String,
    description: String,
    default_enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_feature_flag_request_serialization() {
        let req = SetFeatureFlagRequest {
            description: Some("Enable DVI module".to_string()),
            default_enabled: true,
        };
        let json = serde_json::to_string(&req).expect("should serialize");
        assert!(json.contains("Enable DVI module"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_set_feature_flag_request_optional_description() {
        let req = SetFeatureFlagRequest {
            description: None,
            default_enabled: false,
        };
        let json = serde_json::to_string(&req).expect("should serialize");
        assert!(json.contains("false"));
    }

    #[test]
    fn test_feature_flag_response_serialization() {
        let resp = FeatureFlagResponse {
            id: "flag-uuid".to_string(),
            key: "module.dvi".to_string(),
            description: "Digital Vehicle Inspection".to_string(),
            default_enabled: true,
            created_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&resp).expect("should serialize");
        assert!(json.contains("module.dvi"));
        assert!(json.contains("Digital Vehicle Inspection"));
        assert!(json.contains("flag-uuid"));
    }

    #[test]
    fn test_feature_flag_response_deserialization() {
        let json = r#"{
            "id": "flag-123",
            "key": "jobs.intake",
            "description": "Intake inspection",
            "default_enabled": true,
            "created_at": "2024-01-01T00:00:00Z"
        }"#;
        let resp: FeatureFlagResponse =
            serde_json::from_str(json).expect("should deserialize");
        assert_eq!(resp.key, "jobs.intake");
        assert_eq!(resp.default_enabled, true);
    }

    #[test]
    fn test_tenant_override_response_serialization() {
        let resp = TenantOverrideResponse {
            tenant_id: "tenant-abc".to_string(),
            feature_flag_id: "flag-xyz".to_string(),
            is_enabled: false,
        };
        let json = serde_json::to_string(&resp).expect("should serialize");
        assert!(json.contains("tenant-abc"));
        assert!(json.contains("flag-xyz"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_tenant_override_response_round_trip() {
        let original = TenantOverrideResponse {
            tenant_id: "tenant-test".to_string(),
            feature_flag_id: "flag-test".to_string(),
            is_enabled: true,
        };
        let json = serde_json::to_string(&original).expect("should serialize");
        let parsed: TenantOverrideResponse =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(parsed.tenant_id, original.tenant_id);
        assert_eq!(parsed.feature_flag_id, original.feature_flag_id);
        assert_eq!(parsed.is_enabled, original.is_enabled);
    }
}
