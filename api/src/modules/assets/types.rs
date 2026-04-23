use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ListQuery {
    pub page: i64,
    pub limit: i64,
    pub search: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
            search: None,
            category: None,
            status: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct DueInspectionQuery {
    pub page: i64,
    pub limit: i64,
    pub search: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub days_since_last: i32,
}

impl Default for DueInspectionQuery {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
            search: None,
            category: None,
            status: None,
            days_since_last: 30,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct OpenDefectQuery {
    pub page: i64,
    pub limit: i64,
    pub search: Option<String>,
    pub severity: Option<String>,
}

impl Default for OpenDefectQuery {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
            search: None,
            severity: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetResponse {
    pub id: String,
    pub asset_tag: String,
    pub name: String,
    pub category: Option<String>,
    pub location_id: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_cost: Option<String>,
    pub useful_life_years: Option<i32>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub last_inspection_at: Option<String>,
    pub open_defect_count: i64,
}

#[derive(Debug, FromRow)]
pub struct AssetRow {
    pub id: String,
    pub asset_tag: String,
    pub name: String,
    pub category: Option<String>,
    pub location_id: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub purchase_cost: Option<String>,
    pub useful_life_years: Option<i32>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_inspection_at: Option<DateTime<Utc>>,
    pub open_defect_count: i64,
}

impl From<AssetRow> for AssetResponse {
    fn from(row: AssetRow) -> Self {
        Self {
            id: row.id,
            asset_tag: row.asset_tag,
            name: row.name,
            category: row.category,
            location_id: row.location_id,
            purchase_date: row.purchase_date.map(|value| value.to_string()),
            purchase_cost: row.purchase_cost,
            useful_life_years: row.useful_life_years,
            status: row.status,
            notes: row.notes,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            last_inspection_at: row.last_inspection_at.map(|value| value.to_rfc3339()),
            open_defect_count: row.open_defect_count,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInspectionResponse {
    pub id: String,
    pub asset_id: String,
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub data: Value,
    pub submitted_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, FromRow)]
pub struct AssetInspectionRow {
    pub id: String,
    pub asset_id: String,
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub data: Value,
    pub submitted_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<AssetInspectionRow> for AssetInspectionResponse {
    fn from(row: AssetInspectionRow) -> Self {
        Self {
            id: row.id,
            asset_id: row.asset_id,
            template_id: row.template_id,
            template_name: row.template_name,
            data: row.data,
            submitted_by: row.submitted_by,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDefectResponse {
    pub id: String,
    pub asset_id: String,
    pub asset_tag: String,
    pub asset_name: String,
    pub description: String,
    pub severity: String,
    pub status: String,
    pub reported_by: Option<String>,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, FromRow)]
pub struct AssetDefectRow {
    pub id: String,
    pub asset_id: String,
    pub asset_tag: String,
    pub asset_name: String,
    pub description: String,
    pub severity: String,
    pub status: String,
    pub reported_by: Option<String>,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<AssetDefectRow> for AssetDefectResponse {
    fn from(row: AssetDefectRow) -> Self {
        Self {
            id: row.id,
            asset_id: row.asset_id,
            asset_tag: row.asset_tag,
            asset_name: row.asset_name,
            description: row.description,
            severity: row.severity,
            status: row.status,
            reported_by: row.reported_by,
            resolved_by: row.resolved_by,
            resolved_at: row.resolved_at.map(|value| value.to_rfc3339()),
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDetailResponse {
    pub asset: AssetResponse,
    pub recent_inspections: Vec<AssetInspectionResponse>,
    pub open_defects: Vec<AssetDefectResponse>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AssetRequest {
    #[validate(length(min = 1, message = "Asset tag is required"))]
    pub asset_tag: String,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    pub category: Option<String>,
    pub location_id: Option<String>,
    pub purchase_date: Option<String>,
    pub purchase_cost: Option<String>,
    pub useful_life_years: Option<i32>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInspectionRequest {
    pub template_id: Option<String>,
    pub data: Value,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateAssetDefectRequest {
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: String,
    #[validate(length(min = 1, message = "Severity is required"))]
    pub severity: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAssetDefectRequest {
    pub description: Option<String>,
    pub severity: Option<String>,
    #[validate(length(min = 1, message = "Status is required"))]
    pub status: String,
}
