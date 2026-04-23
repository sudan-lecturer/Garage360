use axum::{
    extract::Path,
    routing::{delete as delete_route, get, post, put},
    Json, Router,
};
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::{auth::AuthUser, tenant::TenantDbPool};
use crate::AppState;

use super::{
    service,
    types::{CreateDviTemplateRequest, DviResultRequest, DviResultResponse, DviTemplateResponse},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dvi/templates", get(list_templates))
        .route("/dvi/templates", post(create_template))
        .route("/dvi/templates/:id", put(update_template))
        .route("/dvi/templates/:id", delete_route(delete_template))
        .route("/dvi/results", post(create_result))
        .route("/dvi/results/:id", get(get_result))
        .route("/dvi/results/:id", put(update_result))
        .route("/dvi/results/:id", delete_route(delete_result))
}

async fn list_templates(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
) -> AppResult<Json<Vec<DviTemplateResponse>>> {
    Ok(Json(service::list_templates(&tenant_db.pool).await?))
}

async fn create_template(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Json(req): Json<CreateDviTemplateRequest>,
) -> AppResult<Json<DviTemplateResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::create_template(&tenant_db.pool, &req).await?))
}

async fn update_template(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreateDviTemplateRequest>,
) -> AppResult<Json<DviTemplateResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(service::update_template(&tenant_db.pool, &id, &req).await?))
}

async fn delete_template(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(service::delete_template(&tenant_db.pool, &id).await?))
}

async fn create_result(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<DviResultRequest>,
) -> AppResult<Json<DviResultResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_result(&tenant_db.pool, &req, &auth.user_id).await?,
    ))
}

async fn get_result(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<DviResultResponse>> {
    Ok(Json(service::get_result(&tenant_db.pool, &id).await?))
}

async fn update_result(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<DviResultRequest>,
) -> AppResult<Json<DviResultResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_result(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn delete_result(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(service::delete_result(&tenant_db.pool, &id).await?))
}
