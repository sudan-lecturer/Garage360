use axum::{
    extract::{Path, Query},
    routing::{delete as delete_route, get, post, put},
    Json, Router,
};
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::{auth::AuthUser, tenant::TenantDbPool};
use crate::AppState;

use super::{
    service,
    types::{CreateCustomerRequest, ListQuery},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/customers", get(list))
        .route("/customers", post(create))
        .route("/customers/search", get(search))
        .route("/customers/:id", get(show))
        .route("/customers/:id", put(update))
        .route("/customers/:id", delete_route(remove))
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
        service::list_customers(&tenant_db.pool, page, limit, search).await?,
    ))
}

async fn search(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<super::types::CustomerResponse>>> {
    let search = query.search.unwrap_or_default();
    Ok(Json(service::search_customers(&tenant_db.pool, search).await?))
}

async fn show(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<super::types::CustomerResponse>> {
    Ok(Json(service::get_customer(&tenant_db.pool, &id).await?))
}

async fn create(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<CreateCustomerRequest>,
) -> AppResult<Json<super::types::CustomerResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_customer(&tenant_db.pool, &req, &auth.user_id).await?,
    ))
}

async fn update(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreateCustomerRequest>,
) -> AppResult<Json<super::types::CustomerResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::update_customer(&tenant_db.pool, &id, &req).await?))
}

async fn remove(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(service::delete_customer(&tenant_db.pool, &id).await?))
}
