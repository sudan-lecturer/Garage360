use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tenants))
        .route("/", post(create_tenant))
        .route("/:id", get(get_tenant))
        .route("/:id", put(update_tenant))
        .route("/:id", delete(deactivate_tenant))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub database_host: String,
    pub database_port: i32,
    pub database_name: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub settings: serde_json::Value,
}

pub async fn list_tenants(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<TenantResponse>>> {
    ensure_super_admin(&user)?;

    let tenants = sqlx::query_as::<_, TenantRow>(
        r#"
        SELECT id, name, slug, database_host, database_port, database_name, is_active, created_at
        FROM tenants
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.control_db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(
        tenants.into_iter().map(TenantResponse::from).collect::<Vec<_>>(),
    ))
}

pub async fn create_tenant(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateTenantRequest>,
) -> AppResult<Json<TenantResponse>> {
    ensure_super_admin(&user)?;
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    let tenant_id = Uuid::now_v7().to_string();
    let database_name = format!("tenant_{}", tenant_id.replace('-', ""));

    let tenant = sqlx::query_as::<_, TenantRow>(
        r#"
        INSERT INTO tenants (
            id, name, slug, database_host, database_port, database_name, database_username, database_password
        )
        VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, name, slug, database_host, database_port, database_name, is_active, created_at
        "#,
    )
    .bind(&tenant_id)
    .bind(&req.name)
    .bind(&req.slug)
    .bind("localhost")
    .bind(5432_i32)
    .bind(&database_name)
    .bind("postgres")
    .bind("postgres")
    .fetch_one(&state.control_db)
    .await
    .map_err(AppError::Database)?;

    sqlx::query(
        r#"
        INSERT INTO control_audit_logs (action, entity_type, entity_id, performed_by, performed_by_role, metadata)
        VALUES ('CREATE_TENANT', 'tenant', $1, $2, $3, $4)
        "#,
    )
    .bind(&tenant_id)
    .bind(&user.user_id)
    .bind(&user.role)
    .bind(serde_json::json!({ "name": req.name, "slug": req.slug }))
    .execute(&state.control_db)
    .await
    .ok();

    Ok(Json(TenantResponse::from(tenant)))
}

pub async fn get_tenant(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<TenantResponse>> {
    ensure_super_admin(&user)?;

    let tenant = sqlx::query_as::<_, TenantRow>(
        r#"
        SELECT id, name, slug, database_host, database_port, database_name, is_active, created_at
        FROM tenants
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.control_db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Tenant not found".into()))?;

    Ok(Json(TenantResponse::from(tenant)))
}

pub async fn update_tenant(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateTenantRequest>,
) -> AppResult<Json<TenantResponse>> {
    ensure_super_admin(&user)?;

    let tenant = sqlx::query_as::<_, TenantRow>(
        r#"
        UPDATE tenants
        SET name = COALESCE($2, name),
            is_active = COALESCE($3, is_active),
            updated_at = NOW()
        WHERE id = $1::uuid
        RETURNING id, name, slug, database_host, database_port, database_name, is_active, created_at
        "#,
    )
    .bind(&id)
    .bind(&req.name)
    .bind(req.is_active)
    .fetch_optional(&state.control_db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Tenant not found".into()))?;

    Ok(Json(TenantResponse::from(tenant)))
}

pub async fn deactivate_tenant(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_super_admin(&user)?;

    let result = sqlx::query(
        r#"
        UPDATE tenants
        SET is_active = false, updated_at = NOW()
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .execute(&state.control_db)
    .await
    .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Tenant not found".into()));
    }

    Ok(Json(serde_json::json!({ "deactivated": true })))
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateTenantRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    #[validate(length(min = 1, message = "Slug is required"))]
    pub slug: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(sqlx::FromRow)]
struct TenantRow {
    id: String,
    name: String,
    slug: String,
    database_host: String,
    database_port: i32,
    database_name: String,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<TenantRow> for TenantResponse {
    fn from(row: TenantRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            slug: row.slug,
            database_host: row.database_host,
            database_port: row.database_port,
            database_name: row.database_name,
            is_active: row.is_active,
            created_at: row.created_at,
            settings: serde_json::json!({}),
        }
    }
}

fn ensure_super_admin(user: &AuthUser) -> AppResult<()> {
    if user.role == "SUPER_ADMIN" {
        Ok(())
    } else {
        Err(AppError::Forbidden("Super admin access required".into()))
    }
}
