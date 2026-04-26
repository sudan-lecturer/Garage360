use axum::{
    extract::State,
    extract::{Path, Query},
    routing::{delete, get, post, put},
    Json, Router,
};
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::{auth::AuthUser, tenant::TenantDbPool};
use crate::AppState;

use super::{
    service,
    types::{ListQuery, VehicleRequest, VehicleResponse},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/vehicles", get(list))
        .route("/vehicles", post(create))
        .route("/vehicles/search", get(search))
        .route("/vehicles/:id", get(show))
        .route("/vehicles/:id", put(update))
        .route("/vehicles/:id", delete(remove))
}

async fn list(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let search = query.search.unwrap_or_default();

    Ok(Json(
        service::list_vehicles(&tenant_db.pool, page, limit, search).await?,
    ))
}

async fn search(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<VehicleResponse>>> {
    let search = query.search.unwrap_or_default();
    Ok(Json(service::search_vehicles(&tenant_db.pool, search).await?))
}

async fn show(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<VehicleResponse>> {
    Ok(Json(service::get_vehicle(&tenant_db.pool, &id).await?))
}

async fn create(
    State(state): State<AppState>,
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<VehicleRequest>,
) -> AppResult<Json<VehicleResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_vehicle(&state.storage, &tenant_db.pool, &req, &auth.user_id).await?,
    ))
}

async fn update(
    State(state): State<AppState>,
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<VehicleRequest>,
) -> AppResult<Json<VehicleResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::update_vehicle(&state.storage, &tenant_db.pool, &id, &req).await?))
}

async fn remove(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(service::delete_vehicle(&tenant_db.pool, &id).await?))
}
