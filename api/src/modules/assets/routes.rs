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
        AssetInspectionRequest, AssetRequest, CreateAssetDefectRequest, DueInspectionQuery,
        ListQuery, OpenDefectQuery, UpdateAssetDefectRequest,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/assets", get(list))
        .route("/assets", post(create))
        .route("/assets/due-inspection", get(due_inspection))
        .route("/assets/defects/open", get(open_defects))
        .route("/assets/:id", get(show))
        .route("/assets/:id", put(update))
        .route("/assets/:id/inspect", post(inspect))
        .route("/assets/:id/inspections", get(list_inspections))
        .route("/assets/:id/defects", post(create_defect))
        .route("/assets/:id/defects/:def_id", put(update_defect))
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
    let status = query.status.unwrap_or_default().trim().to_uppercase();

    Ok(Json(
        service::list_assets(&tenant_db.pool, page, limit, search, category, status).await?,
    ))
}

async fn create(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Json(req): Json<AssetRequest>,
) -> AppResult<Json<super::types::AssetResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::create_asset(&tenant_db.pool, &req).await?))
}

async fn due_inspection(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<DueInspectionQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let search = query.search.unwrap_or_default().trim().to_string();
    let category = query.category.unwrap_or_default().trim().to_string();
    let status = query.status.unwrap_or_default().trim().to_uppercase();
    let days_since_last = query.days_since_last.max(1);

    Ok(Json(
        service::list_due_inspection_assets(
            &tenant_db.pool,
            page,
            limit,
            search,
            category,
            status,
            days_since_last,
        )
        .await?,
    ))
}

async fn open_defects(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<OpenDefectQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let search = query.search.unwrap_or_default().trim().to_string();
    let severity = query.severity.unwrap_or_default().trim().to_uppercase();

    Ok(Json(
        service::list_open_defects(&tenant_db.pool, page, limit, search, severity).await?,
    ))
}

async fn show(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<super::types::AssetDetailResponse>> {
    Ok(Json(service::get_asset_detail(&tenant_db.pool, &id).await?))
}

async fn update(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<AssetRequest>,
) -> AppResult<Json<super::types::AssetResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::update_asset(&tenant_db.pool, &id, &req).await?))
}

async fn inspect(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<AssetInspectionRequest>,
) -> AppResult<Json<super::types::AssetInspectionResponse>> {
    Ok(Json(
        service::inspect_asset(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn list_inspections(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<super::types::AssetInspectionResponse>>> {
    Ok(Json(service::list_asset_inspections(&tenant_db.pool, &id).await?))
}

async fn create_defect(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreateAssetDefectRequest>,
) -> AppResult<Json<super::types::AssetDefectResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_asset_defect(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn update_defect(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path((id, def_id)): Path<(String, String)>,
    Json(req): Json<UpdateAssetDefectRequest>,
) -> AppResult<Json<super::types::AssetDefectResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_asset_defect(&tenant_db.pool, &id, &def_id, &req, &auth.user_id).await?,
    ))
}
