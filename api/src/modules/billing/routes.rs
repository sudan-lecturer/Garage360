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
        CreateInvoiceRequest, InvoiceResponse, ListQuery, RecordPaymentRequest, VoidInvoiceRequest,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/invoices", get(list))
        .route("/invoices", post(create))
        .route("/invoices/:id", get(show))
        .route("/invoices/:id", put(update))
        .route("/invoices/:id/issue", post(issue))
        .route("/invoices/:id/payment", post(record_payment))
        .route("/invoices/:id/void", post(void_invoice))
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
        service::list_invoices(&tenant_db.pool, page, limit, search, status).await?,
    ))
}

async fn show(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<InvoiceResponse>> {
    Ok(Json(service::get_invoice(&tenant_db.pool, &id).await?))
}

async fn create(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Json(req): Json<CreateInvoiceRequest>,
) -> AppResult<Json<InvoiceResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::create_invoice(&tenant_db.pool, &req).await?))
}

async fn update(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreateInvoiceRequest>,
) -> AppResult<Json<InvoiceResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::update_invoice(&tenant_db.pool, &id, &req).await?))
}

async fn issue(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<InvoiceResponse>> {
    Ok(Json(service::issue_invoice(&tenant_db.pool, &id).await?))
}

async fn record_payment(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<RecordPaymentRequest>,
) -> AppResult<Json<InvoiceResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::record_payment(&tenant_db.pool, &id, &req).await?,
    ))
}

async fn void_invoice(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<VoidInvoiceRequest>,
) -> AppResult<Json<InvoiceResponse>> {
    Ok(Json(service::void_invoice(&tenant_db.pool, &id, &req).await?))
}
