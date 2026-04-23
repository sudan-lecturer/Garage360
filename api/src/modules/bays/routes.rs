use axum::{
    extract::Path,
    routing::{delete, get, post, put},
    Json, Router,
};
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::{auth::AuthUser, tenant::TenantDbPool};
use crate::AppState;

use super::{
    service,
    types::{CreateBayRequest, UpdateBayStatusRequest},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/bays/board", get(board))
        .route("/settings/bays", get(list_settings))
        .route("/settings/bays", post(create))
        .route("/settings/bays/:id", put(update))
        .route("/settings/bays/:id/status", put(update_status))
        .route("/settings/bays/:id", delete(remove))
}

async fn board(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
) -> AppResult<Json<Vec<super::types::BayBoardResponse>>> {
    Ok(Json(service::list_board(&tenant_db.pool).await?))
}

async fn list_settings(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
) -> AppResult<Json<Vec<super::types::BayResponse>>> {
    Ok(Json(service::list_settings(&tenant_db.pool).await?))
}

async fn create(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Json(req): Json<CreateBayRequest>,
) -> AppResult<Json<super::types::BayResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::create_bay(&tenant_db.pool, &req).await?))
}

async fn update(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreateBayRequest>,
) -> AppResult<Json<super::types::BayResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::update_bay(&tenant_db.pool, &id, &req).await?))
}

async fn update_status(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateBayStatusRequest>,
) -> AppResult<Json<super::types::BayResponse>> {
    Ok(Json(
        service::update_bay_status(&tenant_db.pool, &id, &req).await?,
    ))
}

async fn remove(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(service::delete_bay(&tenant_db.pool, &id).await?))
}
