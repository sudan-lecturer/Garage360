use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BayResponse {
    pub id: String,
    pub code: String,
    pub name: String,
    pub capacity: i32,
    pub is_active: bool,
    pub active_job_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow)]
pub struct BayRow {
    pub id: String,
    pub code: String,
    pub name: String,
    pub capacity: i32,
    pub is_active: bool,
    pub active_job_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<BayRow> for BayResponse {
    fn from(row: BayRow) -> Self {
        Self {
            id: row.id,
            code: row.code,
            name: row.name,
            capacity: row.capacity,
            is_active: row.is_active,
            active_job_count: row.active_job_count,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BayBoardResponse {
    pub id: String,
    pub code: String,
    pub name: String,
    pub capacity: i32,
    pub occupancy_count: i64,
    pub available_slots: i32,
    pub status: String,
    pub jobs: Vec<BayBoardJobResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BayBoardJobResponse {
    pub job_id: String,
    pub job_no: i32,
    pub job_status: String,
    pub vehicle_id: String,
    pub registration_no: String,
    pub customer_id: String,
    pub customer_name: String,
}

#[derive(Debug, FromRow)]
pub struct BayBoardBayRow {
    pub id: String,
    pub code: String,
    pub name: String,
    pub capacity: i32,
}

#[derive(Debug, FromRow)]
pub struct BayBoardJobRow {
    pub bay_id: String,
    pub job_id: String,
    pub job_no: i32,
    pub job_status: String,
    pub vehicle_id: String,
    pub registration_no: String,
    pub customer_id: String,
    pub customer_name: String,
}

impl From<BayBoardJobRow> for BayBoardJobResponse {
    fn from(row: BayBoardJobRow) -> Self {
        Self {
            job_id: row.job_id,
            job_no: row.job_no,
            job_status: row.job_status,
            vehicle_id: row.vehicle_id,
            registration_no: row.registration_no,
            customer_id: row.customer_id,
            customer_name: row.customer_name,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBayRequest {
    #[validate(length(min = 1, message = "Code is required"))]
    pub code: String,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    #[validate(range(min = 1, message = "Capacity must be at least 1"))]
    pub capacity: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBayStatusRequest {
    pub is_active: bool,
}
