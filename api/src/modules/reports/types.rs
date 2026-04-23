use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedReportResponse {
    pub id: String,
    pub name: String,
    pub report_type: String,
    pub config: Value,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct SavedReportRow {
    pub id: String,
    pub name: String,
    pub report_type: String,
    pub config: Value,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<SavedReportRow> for SavedReportResponse {
    fn from(row: SavedReportRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            report_type: row.report_type,
            config: row.config,
            created_at: row.created_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct GenerateReportRequest {
    #[validate(length(min = 1))]
    pub report_type: String,
    pub config: Option<Value>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ExportReportRequest {
    #[validate(length(min = 1))]
    pub report_type: String,
    pub config: Option<Value>,
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SaveReportRequest {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub report_type: String,
    pub config: Value,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RevenueTotalsRow {
    pub invoice_count: i64,
    pub total_invoiced: String,
    pub total_collected: String,
    pub total_outstanding: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RevenueStatusRow {
    pub status: String,
    pub invoice_count: i64,
    pub total_amount: String,
    pub amount_paid: String,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct JobSummaryRow {
    pub total_jobs: i64,
    pub completed_jobs: i64,
    pub cancelled_jobs: i64,
    pub open_jobs: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct JobStatusCountRow {
    pub status: String,
    pub job_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MechanicJobCountRow {
    pub mechanic_name: String,
    pub job_count: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CustomerActivityRow {
    pub customer_id: String,
    pub customer_name: String,
    pub job_count: i64,
    pub invoiced_amount: String,
    pub paid_amount: String,
}
