use axum::{
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
    types::{InventoryItemRequest, ListQuery, StockAdjustmentRequest},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/inventory", get(list))
        .route("/inventory", post(create))
        .route("/inventory/search", get(search))
        .route("/inventory/low-stock", get(low_stock))
        .route("/inventory/export", get(export))
        .route("/inventory/:id", get(show))
        .route("/inventory/:id", put(update))
        .route("/inventory/:id", delete(remove))
        .route("/inventory/:id/adjust", post(adjust))
}

async fn list(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let search = query.search.unwrap_or_default().trim().to_string();
    let category = query.category.unwrap_or_default().trim().to_string();

    Ok(Json(
        service::list_inventory(&tenant_db.pool, page, limit, search, category).await?,
    ))
}

async fn create(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<InventoryItemRequest>,
) -> AppResult<Json<super::types::InventoryItemResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_inventory_item(&tenant_db.pool, &req, &auth.user_id).await?,
    ))
}

async fn search(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<super::types::InventoryItemResponse>>> {
    let search = query.search.unwrap_or_default().trim().to_string();
    Ok(Json(service::search_inventory(&tenant_db.pool, search).await?))
}

async fn low_stock(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let search = query.search.unwrap_or_default().trim().to_string();
    let category = query.category.unwrap_or_default().trim().to_string();

    Ok(Json(
        service::list_low_stock(&tenant_db.pool, page, limit, search, category).await?,
    ))
}

async fn export(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let search = query.search.unwrap_or_default().trim().to_string();
    let category = query.category.unwrap_or_default().trim().to_string();
    Ok(Json(
      service::export_inventory(&tenant_db.pool, search, category).await?
    ))
}

async fn show(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<super::types::InventoryItemDetailResponse>> {
    Ok(Json(service::get_inventory_item(&tenant_db.pool, &id).await?))
}

async fn update(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<InventoryItemRequest>,
) -> AppResult<Json<super::types::InventoryItemResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_inventory_item(&tenant_db.pool, &id, &req).await?,
    ))
}

async fn remove(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(service::delete_inventory_item(&tenant_db.pool, &id).await?))
}

async fn adjust(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<StockAdjustmentRequest>,
) -> AppResult<Json<super::types::InventoryAdjustmentResult>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::adjust_inventory_stock(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}
