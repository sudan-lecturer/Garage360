use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DviTemplateResponse {
    pub id: String,
    pub name: String,
    pub sections: Value,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct DviTemplateRow {
    pub id: String,
    pub name: String,
    pub sections: Value,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<DviTemplateRow> for DviTemplateResponse {
    fn from(row: DviTemplateRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            sections: row.sections,
            created_at: row.created_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDviTemplateRequest {
    #[validate(length(min = 1))]
    pub name: String,
    pub sections: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DviResultResponse {
    pub id: String,
    pub job_card_id: String,
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub data: Value,
    pub submitted_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct DviResultRow {
    pub id: String,
    pub job_card_id: String,
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub data: Value,
    pub submitted_by: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<DviResultRow> for DviResultResponse {
    fn from(row: DviResultRow) -> Self {
        Self {
            id: row.id,
            job_card_id: row.job_card_id,
            template_id: row.template_id,
            template_name: row.template_name,
            data: row.data,
            submitted_by: row.submitted_by,
            created_at: row.created_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct DviResultRequest {
    #[validate(length(min = 1))]
    pub job_card_id: String,
    pub template_id: Option<String>,
    pub data: Value,
}
