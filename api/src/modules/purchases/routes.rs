use axum::{
    extract::{Path, Query},
    routing::{get, post, put},
    Json, Router,
};
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::{auth::AuthUser, tenant::TenantDbPool};
use crate::AppState;

use super::{
    service,
    types::{
        CreateGrnRequest, CreatePurchaseOrderRequest, CreateQaInspectionRequest, ListQuery,
        PurchaseActionRequest, PurchaseHistoryResponse, PurchaseOrderResponse,
        TransitPurchaseRequest,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/purchases", get(list))
        .route("/purchases", post(create))
        .route("/purchases/in-transit", get(in_transit))
        .route("/purchases/:id", get(show))
        .route("/purchases/:id", put(update))
        .route("/purchases/:id/history", get(history))
        .route("/purchases/:id/submit", post(submit))
        .route("/purchases/:id/approve", post(approve))
        .route("/purchases/:id/reject", post(reject))
        .route("/purchases/:id/send", post(send))
        .route("/purchases/:id/transit", post(mark_transit))
        .route("/purchases/:id/grn", post(create_grn))
        .route("/purchases/:id/grn/:grn_id", get(show_grn))
        .route("/purchases/:id/grn/:grn_id/qa", post(record_qa))
}

async fn list(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let search = query.search.unwrap_or_default();
    let status = query.status.unwrap_or_default();

    Ok(Json(
        service::list_purchases(&tenant_db.pool, page, limit, search, status, false).await?,
    ))
}

async fn in_transit(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let search = query.search.unwrap_or_default();

    Ok(Json(
        service::list_purchases(&tenant_db.pool, page, limit, search, String::new(), true).await?,
    ))
}

async fn show(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<PurchaseOrderResponse>> {
    Ok(Json(service::get_purchase(&tenant_db.pool, &id).await?))
}

async fn create(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<CreatePurchaseOrderRequest>,
) -> AppResult<Json<PurchaseOrderResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_purchase(&tenant_db.pool, &req, &auth.user_id).await?,
    ))
}

async fn update(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreatePurchaseOrderRequest>,
) -> AppResult<Json<PurchaseOrderResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_purchase(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn history(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<PurchaseHistoryResponse>> {
    Ok(Json(service::get_purchase_history(&tenant_db.pool, &id).await?))
}

async fn submit(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<PurchaseActionRequest>,
) -> AppResult<Json<PurchaseOrderResponse>> {
    Ok(Json(
        service::submit_purchase(&tenant_db.pool, &id, &auth.user_id, req.notes).await?,
    ))
}

async fn approve(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<PurchaseActionRequest>,
) -> AppResult<Json<PurchaseOrderResponse>> {
    Ok(Json(
        service::approve_purchase(&tenant_db.pool, &id, &auth.user_id, req.notes).await?,
    ))
}

async fn reject(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<PurchaseActionRequest>,
) -> AppResult<Json<PurchaseOrderResponse>> {
    Ok(Json(
        service::reject_purchase(&tenant_db.pool, &id, &auth.user_id, req.notes).await?,
    ))
}

async fn send(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<PurchaseActionRequest>,
) -> AppResult<Json<PurchaseOrderResponse>> {
    Ok(Json(
        service::send_purchase(&tenant_db.pool, &id, &auth.user_id, req.notes).await?,
    ))
}

async fn mark_transit(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<TransitPurchaseRequest>,
) -> AppResult<Json<PurchaseOrderResponse>> {
    Ok(Json(
        service::mark_purchase_in_transit(&tenant_db.pool, &id, &auth.user_id, &req).await?,
    ))
}

async fn create_grn(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreateGrnRequest>,
) -> AppResult<Json<super::types::GoodsReceiptNoteResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_grn(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn show_grn(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path((id, grn_id)): Path<(String, String)>,
) -> AppResult<Json<super::types::GoodsReceiptNoteResponse>> {
    Ok(Json(service::get_grn(&tenant_db.pool, &id, &grn_id).await?))
}

async fn record_qa(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path((id, grn_id)): Path<(String, String)>,
    Json(req): Json<CreateQaInspectionRequest>,
) -> AppResult<Json<super::types::GoodsReceiptNoteResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::record_qa(&tenant_db.pool, &id, &grn_id, &req, &auth.user_id).await?,
    ))
}
