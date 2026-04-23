use axum::{
    extract::Path,
    routing::{delete as delete_route, get, post},
    Json, Router,
};
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::{auth::AuthUser, tenant::TenantDbPool};
use crate::AppState;

use super::{
    service,
    types::{
        ExportReportRequest, GenerateReportRequest, SaveReportRequest, SavedReportResponse,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/reports/generate", post(generate))
        .route("/reports/export", post(export))
        .route("/reports/saved", get(list_saved))
        .route("/reports/saved", post(create_saved))
        .route("/reports/saved/:id", delete_route(delete_saved))
}

async fn generate(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Json(req): Json<GenerateReportRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::generate_report(&tenant_db.pool, &req).await?))
}

async fn export(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Json(req): Json<ExportReportRequest>,
) -> AppResult<Json<serde_json::Value>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::export_report(&tenant_db.pool, &req).await?))
}

async fn list_saved(
    tenant_db: TenantDbPool,
    auth: AuthUser,
) -> AppResult<Json<Vec<SavedReportResponse>>> {
    Ok(Json(
        service::list_saved_reports(&tenant_db.pool, &auth.user_id).await?,
    ))
}

async fn create_saved(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<SaveReportRequest>,
) -> AppResult<Json<SavedReportResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_saved_report(&tenant_db.pool, &req, &auth.user_id).await?,
    ))
}

async fn delete_saved(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(
        service::delete_saved_report(&tenant_db.pool, &id, &auth.user_id).await?,
    ))
}
