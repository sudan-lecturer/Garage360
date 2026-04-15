use axum::{
    extract::{State, Extension},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::AppState;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(list_tenants))
        .route("/", post(create_tenant))
        .route("/:id", get(get_tenant))
        .route("/:id", put(update_tenant))
        .route("/:id", delete(deactivate_tenant))
}

#[derive(Debug, Serialize)]
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
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<TenantResponse>>> {
    if user.role != "SUPER_ADMIN" {
        return Err(AppError::Forbidden("Super admin access required".into()));
    }

    let tenants = sqlx::query_as::<_, TenantRow>(
        r#"
        SELECT id, name, slug, database_host, database_port, database_name, 
               is_active, created_at, updated_at
        FROM tenants
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&*state.db)
    .await
    .map_err(|e| AppError::Database(e))?;

    let response: Vec<TenantResponse> = tenants
        .into_iter()
        .map(|t| TenantResponse {
            id: t.id,
            name: t.name,
            slug: t.slug,
            database_host: t.database_host,
            database_port: t.database_port,
            database_name: t.database_name,
            is_active: t.is_active,
            created_at: t.created_at,
            settings: serde_json::json!({}),
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_tenant(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    Json(req): Json<CreateTenantRequest>,
) -> AppResult<Json<TenantResponse>> {
    if user.role != "SUPER_ADMIN" {
        return Err(AppError::Forbidden("Super admin access required".into()));
    }

    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let tenant_id = Uuid::now_v7().to_string();
    let database_name = format!("tenant_{}", tenant_id.replace("-", ""));

    let mut tx = state.db.begin().await.map_err(|e| AppError::Database(e))?;

    sqlx::query(
        r#"
        INSERT INTO tenants (id, name, slug, database_host, database_port, database_name, database_username, database_password)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(&tenant_id)
    .bind(&req.name)
    .bind(&req.slug)
    .bind("localhost")
    .bind(5432)
    .bind(&database_name)
    .bind("postgres")
    .bind("postgres")
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e))?;

    sqlx::query(
        r#"
        CREATE DATABASE "{}"
        "#,
    )
    .bind(&database_name)
    .execute(&mut *tx)
    .await
    .ok();

    sqlx::query(
        r#"
        INSERT INTO control_audit_logs (action, entity_type, entity_id, performed_by, metadata)
        VALUES ('CREATE_TENANT', 'tenant', $1, $2, $3)
        "#,
    )
    .bind(&tenant_id)
    .bind(&user.user_id)
    .bind(serde_json::json!({ "name": &req.name }))
    .execute(&mut *tx)
    .await
    .ok();

    tx.commit().await.map_err(|e| AppError::Database(e))?;

    Ok(Json(TenantResponse {
        id: tenant_id,
        name: req.name,
        slug: req.slug,
        database_host: "localhost".into(),
        database_port: 5432,
        database_name,
        is_active: true,
        created_at: chrono::Utc::now(),
        settings: serde_json::json!({}),
    }))
}

pub async fn get_tenant(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> AppResult<Json<TenantResponse>> {
    if user.role != "SUPER_ADMIN" {
        return Err(AppError::Forbidden("Super admin access required".into()));
    }

    let tenant = sqlx::query_as::<_, TenantRow>(
        r#"
        SELECT id, name, slug, database_host, database_port, database_name,
               is_active, created_at, updated_at
        FROM tenants
        WHERE id = $1
        "#,
    )
    .bind(&id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::NotFound("Tenant not found".into()))?;

    Ok(Json(TenantResponse {
        id: tenant.id,
        name: tenant.name,
        slug: tenant.slug,
        database_host: tenant.database_host,
        database_port: tenant.database_port,
        database_name: tenant.database_name,
        is_active: tenant.is_active,
        created_at: tenant.created_at,
        settings: serde_json::json!({}),
    }))
}

pub async fn update_tenant(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<UpdateTenantRequest>,
) -> AppResult<Json<TenantResponse>> {
    if user.role != "SUPER_ADMIN" {
        return Err(AppError::Forbidden("Super admin access required".into()));
    }

    sqlx::query(
        r#"
        UPDATE tenants
        SET name = COALESCE($1, name),
            is_active = COALESCE($2, is_active),
            updated_at = NOW()
        WHERE id = $3
        "#,
    )
    .bind(&req.name)
    .bind(&req.is_active)
    .bind(&id)
    .execute(&*state.db)
    .await
    .map_err(|e| AppError::Database(e))?;

    get_tenant(State(state), Extension(user), axum::extract::Path(id)).await
}

pub async fn deactivate_tenant(
    State(state): State<AppState>,
    Extension(user): Extension<AuthUser>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> AppResult<Json<()>> {
    if user.role != "SUPER_ADMIN" {
        return Err(AppError::Forbidden("Super admin access required".into()));
    }

    sqlx::query(
        r#"
        UPDATE tenants
        SET is_active = false, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(&id)
    .execute(&*state.db)
    .await
    .map_err(|e| AppError::Database(e))?;

    Ok(Json(()))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTenantRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    #[validate(length(min = 1, message = "Slug is required"))]
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

struct TenantRow {
    id: String,
    name: String,
    slug: String,
    database_host: String,
    database_port: i32,
    database_name: String,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}
