use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use sqlx::{postgres::PgPoolOptions, Executor};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::feature_flags;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tenants))
        .route("/", post(create_tenant))
        .route("/:id/feature-flags/:key", put(feature_flags::set_tenant_feature_flag))
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
        SELECT id::text AS id, name, slug, database_host, database_port, database_name, is_active, created_at
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
    let database_parts = parse_postgres_url(&state.config.database_url)?;

    let slug_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM tenants
            WHERE slug = $1
        )
        "#,
    )
    .bind(&req.slug)
    .fetch_one(&state.control_db)
    .await
    .map_err(AppError::Database)?;

    if slug_exists {
        return Err(AppError::Conflict("Tenant slug already exists".into()));
    }

    create_database(&state.control_db, &database_name).await?;

    let tenant_database_url = build_database_url(&database_parts, &database_name);
    if let Err(err) = bootstrap_tenant_database(&tenant_database_url).await {
        drop_database(&state.control_db, &database_name).await;
        return Err(err);
    }

    let tenant_result = sqlx::query_as::<_, TenantRow>(
        r#"
        INSERT INTO tenants (
            id, name, slug, database_host, database_port, database_name, database_username, database_password
        )
        VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id::text AS id, name, slug, database_host, database_port, database_name, is_active, created_at
        "#,
    )
    .bind(&tenant_id)
    .bind(&req.name)
    .bind(&req.slug)
    .bind(&database_parts.host)
    .bind(i32::from(database_parts.port))
    .bind(&database_name)
    .bind(&database_parts.username)
    .bind(&database_parts.password)
    .fetch_one(&state.control_db)
    .await;

    let tenant = match tenant_result {
        Ok(tenant) => tenant,
        Err(err) => {
            drop_database(&state.control_db, &database_name).await;
            return Err(AppError::Database(err));
        }
    };

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
        SELECT id::text AS id, name, slug, database_host, database_port, database_name, is_active, created_at
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
        RETURNING id::text AS id, name, slug, database_host, database_port, database_name, is_active, created_at
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct DatabaseUrlParts {
    username: String,
    password: String,
    host: String,
    port: u16,
}

const SCHEMA_VERSION: &str = "1.0.0";
const TENANT_SCHEMA_SQL: &str = include_str!("../../../schema/tenant_schema.sql");

fn ensure_super_admin(user: &AuthUser) -> AppResult<()> {
    if user.role == "SUPER_ADMIN" {
        Ok(())
    } else {
        Err(AppError::Forbidden("Super admin access required".into()))
    }
}

async fn create_database(control_db: &sqlx::PgPool, database_name: &str) -> AppResult<()> {
    let database_identifier = quoted_identifier(database_name)?;
    let query = format!("CREATE DATABASE {database_identifier}");

    control_db
        .execute(query.as_str())
        .await
        .map_err(AppError::Database)?;

    Ok(())
}

async fn drop_database(control_db: &sqlx::PgPool, database_name: &str) {
    let Ok(database_identifier) = quoted_identifier(database_name) else {
        return;
    };

    let query = format!("DROP DATABASE IF EXISTS {database_identifier}");

    if let Err(err) = control_db.execute(query.as_str()).await {
        tracing::warn!(database_name, error = ?err, "failed to clean up tenant database");
    }
}

async fn bootstrap_tenant_database(database_url: &str) -> AppResult<()> {
    let tenant_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await
        .map_err(AppError::Database)?;

    let bootstrap_result = run_tenant_schema(&tenant_pool).await;
    tenant_pool.close().await;
    bootstrap_result
}

async fn run_tenant_schema(tenant_pool: &sqlx::PgPool) -> AppResult<()> {
    for statement in split_sql_statements(TENANT_SCHEMA_SQL) {
        tenant_pool
            .execute(statement.as_str())
            .await
            .map_err(AppError::Database)?;
    }

    sqlx::query(
        r#"
        INSERT INTO tenant_settings (key, value, is_encrypted)
        VALUES ('schema_version', $1, false)
        ON CONFLICT (key) DO UPDATE
        SET value = EXCLUDED.value, is_encrypted = EXCLUDED.is_encrypted, updated_at = NOW()
        "#,
    )
    .bind(SCHEMA_VERSION)
    .execute(tenant_pool)
    .await
    .map_err(AppError::Database)?;

    Ok(())
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut chars = sql.chars().peekable();
    let mut in_single_quote = false;
    let mut in_line_comment = false;

    while let Some(ch) = chars.next() {
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        if !in_single_quote && ch == '-' && chars.peek() == Some(&'-') {
            chars.next();
            in_line_comment = true;
            continue;
        }

        if ch == '\'' {
            current.push(ch);

            if in_single_quote && chars.peek() == Some(&'\'') {
                current.push(chars.next().expect("peeked single quote must exist"));
                continue;
            }

            in_single_quote = !in_single_quote;
            continue;
        }

        if !in_single_quote && ch == ';' {
            let statement = current.trim();
            if !statement.is_empty() {
                statements.push(statement.to_string());
            }
            current.clear();
            continue;
        }

        current.push(ch);
    }

    let trailing = current.trim();
    if !trailing.is_empty() {
        statements.push(trailing.to_string());
    }

    statements
}

fn parse_postgres_url(database_url: &str) -> AppResult<DatabaseUrlParts> {
    let without_scheme = database_url
        .strip_prefix("postgres://")
        .or_else(|| database_url.strip_prefix("postgresql://"))
        .ok_or_else(|| AppError::Internal("Unsupported database URL format".into()))?;

    let (credentials, host_and_db) = without_scheme
        .split_once('@')
        .ok_or_else(|| AppError::Internal("Invalid database URL format".into()))?;
    let (username, password) = credentials
        .split_once(':')
        .ok_or_else(|| AppError::Internal("Invalid database credentials format".into()))?;
    let (host_port, _) = host_and_db
        .split_once('/')
        .ok_or_else(|| AppError::Internal("Invalid database host format".into()))?;

    let (host, port) = match host_port.split_once(':') {
        Some((host, port)) => (
            host,
            port.parse::<u16>()
                .map_err(|_| AppError::Internal("Invalid database port".into()))?,
        ),
        None => (host_port, 5432),
    };

    Ok(DatabaseUrlParts {
        username: username.to_string(),
        password: password.to_string(),
        host: host.to_string(),
        port,
    })
}

fn build_database_url(parts: &DatabaseUrlParts, database_name: &str) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        parts.username, parts.password, parts.host, parts.port, database_name
    )
}

fn quoted_identifier(identifier: &str) -> AppResult<String> {
    if identifier
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        Ok(format!("\"{identifier}\""))
    } else {
        Err(AppError::Validation("Invalid database identifier".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_postgres_url_extracts_connection_parts() {
        let parts = parse_postgres_url("postgres://postgres:postgres@control-db:5432/control")
            .expect("should parse postgres url");

        assert_eq!(
            parts,
            DatabaseUrlParts {
                username: "postgres".to_string(),
                password: "postgres".to_string(),
                host: "control-db".to_string(),
                port: 5432,
            }
        );
    }

    #[test]
    fn test_build_database_url_replaces_database_name() {
        let parts = DatabaseUrlParts {
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            host: "control-db".to_string(),
            port: 5432,
        };

        assert_eq!(
            build_database_url(&parts, "tenant_test"),
            "postgres://postgres:postgres@control-db:5432/tenant_test"
        );
    }

    #[test]
    fn test_split_sql_statements_ignores_comments_and_keeps_quotes() {
        let statements = split_sql_statements(
            "-- comment\nCREATE TABLE test (name TEXT DEFAULT 'a;b');\nINSERT INTO test VALUES ('ok');",
        );

        assert_eq!(statements.len(), 2);
        assert!(statements[0].contains("CREATE TABLE test"));
        assert!(statements[0].contains("'a;b'"));
        assert!(statements[1].contains("INSERT INTO test"));
    }
}
