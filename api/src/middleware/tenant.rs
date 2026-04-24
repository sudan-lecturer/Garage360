use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use sqlx::Row;

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::AppState;

const CONTROL_TENANT_ID: &str = "control";

#[derive(Debug, Clone)]
pub struct TenantDbPool {
    pub pool: sqlx::PgPool,
    pub tenant_id: String,
}

#[async_trait]
impl FromRequestParts<AppState> for TenantDbPool {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let app_state = state;

        let auth_user = AuthUser::from_request_parts(parts, app_state).await?;

        // Super admin uses control DB directly
        if auth_user.tenant_id == CONTROL_TENANT_ID {
            return Ok(TenantDbPool {
                pool: app_state.control_db.clone(),
                tenant_id: CONTROL_TENANT_ID.to_string(),
            });
        }

        let tenant_db_url = get_tenant_database_url(
            &app_state.control_db,
            &auth_user.tenant_id,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get tenant database: {}", e)))?;

        let pool = app_state
            .tenant_registry
            .get_pool(&auth_user.tenant_id, &tenant_db_url)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to connect to tenant pool: {}", e)))?;

        Ok(TenantDbPool {
            pool,
            tenant_id: auth_user.tenant_id,
        })
    }
}

async fn get_tenant_database_url(
    control_pool: &sqlx::PgPool,
    tenant_id: &str,
) -> anyhow::Result<String> {
    let row = sqlx::query(
        r#"
        SELECT database_host, database_port, database_name, database_username, database_password
        FROM tenants
        WHERE database_name = $1 AND is_active = true
        "#,
    )
    .bind(tenant_id)
    .fetch_optional(control_pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Tenant not found"))?;

    let host: String = row.get("database_host");
    let port: i32 = row.get("database_port");
    let name: String = row.get("database_name");
    let username: String = row.get("database_username");
    let password: String = row.get("database_password");

    Ok(format!(
        "postgres://{}:{}@{}:{}/{}",
        username, password, host, port, name
    ))
}
