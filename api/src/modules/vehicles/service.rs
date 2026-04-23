use serde_json::json;
use sqlx::PgPool;

use crate::errors::{AppError, AppResult};
use crate::common::pagination::PaginationMeta;

use super::{
    repo,
    types::{VehicleRequest, VehicleResponse},
};

pub async fn list_vehicles(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);

    let vehicles = repo::list(pool, &search, &like, limit, offset).await?;
    let total = repo::count(pool, &search, &like).await?;

    Ok(json!({
        "data": vehicles,
        "meta": crate::common::pagination::PaginationMeta { page, limit, total }
    }))
}

pub async fn search_vehicles(
    pool: &PgPool,
    search: String,
) -> AppResult<Vec<VehicleResponse>> {
    let like = format!("%{}%", search);
    repo::list(pool, &search, &like, 20, 0).await
}

pub async fn get_vehicle(pool: &PgPool, id: &str) -> AppResult<VehicleResponse> {
    repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Vehicle not found".into()))
}

pub async fn create_vehicle(
    pool: &PgPool,
    req: &VehicleRequest,
    created_by: &str,
) -> AppResult<VehicleResponse> {
    repo::create(pool, req, created_by).await
}

pub async fn update_vehicle(
    pool: &PgPool,
    id: &str,
    req: &VehicleRequest,
) -> AppResult<VehicleResponse> {
    repo::update(pool, id, req)
        .await?
        .ok_or_else(|| AppError::NotFound("Vehicle not found".into()))
}

pub async fn delete_vehicle(pool: &PgPool, id: &str) -> AppResult<serde_json::Value> {
    let rows_affected = repo::soft_delete(pool, id).await?;
    if rows_affected == 0 {
        return Err(AppError::NotFound("Vehicle not found".into()));
    }

    Ok(json!({ "deleted": true }))
}
