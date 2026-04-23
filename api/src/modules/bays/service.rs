use std::collections::HashMap;

use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

use super::{
    repo,
    types::{
        BayBoardJobResponse, BayBoardResponse, BayResponse, CreateBayRequest, UpdateBayStatusRequest,
    },
};

pub async fn list_board(pool: &PgPool) -> AppResult<Vec<BayBoardResponse>> {
    let bays = repo::list_board_bays(pool).await?;
    let jobs = repo::list_board_jobs(pool).await?;

    let mut jobs_by_bay: HashMap<String, Vec<BayBoardJobResponse>> = HashMap::new();
    for job in jobs {
        jobs_by_bay
            .entry(job.bay_id.clone())
            .or_default()
            .push(BayBoardJobResponse::from(job));
    }

    Ok(bays
        .into_iter()
        .map(|bay| {
            let jobs = jobs_by_bay.remove(&bay.id).unwrap_or_default();
            let occupancy_count = jobs.len() as i64;
            let available_slots = (bay.capacity - occupancy_count as i32).max(0);
            let status = if occupancy_count == 0 {
                "AVAILABLE"
            } else if occupancy_count < bay.capacity as i64 {
                "PARTIAL"
            } else {
                "OCCUPIED"
            };

            BayBoardResponse {
                id: bay.id,
                code: bay.code,
                name: bay.name,
                capacity: bay.capacity,
                occupancy_count,
                available_slots,
                status: status.to_string(),
                jobs,
            }
        })
        .collect::<Vec<_>>())
}

pub async fn list_settings(pool: &PgPool) -> AppResult<Vec<BayResponse>> {
    Ok(repo::list_settings(pool)
        .await?
        .into_iter()
        .map(BayResponse::from)
        .collect::<Vec<_>>())
}

pub async fn create_bay(pool: &PgPool, req: &CreateBayRequest) -> AppResult<BayResponse> {
    let code = normalize_required_text(&req.code, "Code")?;
    let name = normalize_required_text(&req.name, "Name")?;

    let bay_id = Uuid::now_v7().to_string();
    repo::create(pool, &bay_id, &code, &name, req.capacity)
        .await
        .map(BayResponse::from)
}

pub async fn update_bay(pool: &PgPool, id: &str, req: &CreateBayRequest) -> AppResult<BayResponse> {
    ensure_valid_uuid(id, "bay")?;

    let code = normalize_required_text(&req.code, "Code")?;
    let name = normalize_required_text(&req.name, "Name")?;

    repo::update(pool, id, &code, &name, req.capacity)
        .await?
        .map(BayResponse::from)
        .ok_or_else(|| AppError::NotFound("Bay not found".into()))
}

pub async fn update_bay_status(
    pool: &PgPool,
    id: &str,
    req: &UpdateBayStatusRequest,
) -> AppResult<BayResponse> {
    ensure_valid_uuid(id, "bay")?;

    if !req.is_active {
        ensure_bay_is_not_occupied(pool, id).await?;
    }

    repo::set_status(pool, id, req.is_active)
        .await?
        .map(BayResponse::from)
        .ok_or_else(|| AppError::NotFound("Bay not found".into()))
}

pub async fn delete_bay(pool: &PgPool, id: &str) -> AppResult<serde_json::Value> {
    ensure_valid_uuid(id, "bay")?;
    ensure_bay_is_not_occupied(pool, id).await?;

    let rows_affected = repo::soft_delete(pool, id).await?;
    if rows_affected == 0 {
        return Err(AppError::NotFound("Bay not found".into()));
    }

    Ok(json!({ "deleted": true }))
}

async fn ensure_bay_is_not_occupied(pool: &PgPool, id: &str) -> AppResult<()> {
    let active_job_count = repo::active_job_count(pool, id).await?;
    if active_job_count > 0 {
        return Err(AppError::Conflict(
            "Cannot deactivate or delete a bay while active jobs are assigned".into(),
        ));
    }

    Ok(())
}

fn normalize_required_text(value: &str, field_name: &str) -> AppResult<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(AppError::Validation(format!("{} is required", field_name)));
    }

    Ok(normalized.to_string())
}

fn ensure_valid_uuid(value: &str, resource_name: &str) -> AppResult<()> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| AppError::Validation(format!("Invalid {} identifier", resource_name)))
}
