use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use validator::Validate;
use crate::common::pagination::PaginationMeta;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobResponse {
    pub id: String,
    pub job_no: Option<i32>,
    pub vehicle_id: String,
    pub vehicle_registration_no: String,
    pub vehicle_make: String,
    pub vehicle_model: String,
    pub customer_id: String,
    pub customer_name: String,
    pub bay_id: Option<String>,
    pub bay_code: Option<String>,
    pub bay_name: Option<String>,
    pub status: String,
    pub complaint: Option<String>,
    pub diagnosis: Option<String>,
    pub odometer_in: Option<i32>,
    pub odometer_out: Option<i32>,
    pub estimated_completion: Option<String>,
    pub mechanic_id: Option<String>,
    pub mechanic_name: Option<String>,
    pub account_manager_id: Option<String>,
    pub account_manager_name: Option<String>,
    pub qa_by: Option<String>,
    pub qa_name: Option<String>,
    pub qa_cycles: i32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct JobRow {
    pub id: String,
    pub job_no: Option<i32>,
    pub vehicle_id: String,
    pub vehicle_registration_no: String,
    pub vehicle_make: String,
    pub vehicle_model: String,
    pub customer_id: String,
    pub customer_name: String,
    pub bay_id: Option<String>,
    pub bay_code: Option<String>,
    pub bay_name: Option<String>,
    pub status: String,
    pub complaint: Option<String>,
    pub diagnosis: Option<String>,
    pub odometer_in: Option<i32>,
    pub odometer_out: Option<i32>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub mechanic_id: Option<String>,
    pub mechanic_name: Option<String>,
    pub account_manager_id: Option<String>,
    pub account_manager_name: Option<String>,
    pub qa_by: Option<String>,
    pub qa_name: Option<String>,
    pub qa_cycles: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<JobRow> for JobResponse {
    fn from(row: JobRow) -> Self {
        Self {
            id: row.id,
            job_no: row.job_no,
            vehicle_id: row.vehicle_id,
            vehicle_registration_no: row.vehicle_registration_no,
            vehicle_make: row.vehicle_make,
            vehicle_model: row.vehicle_model,
            customer_id: row.customer_id,
            customer_name: row.customer_name,
            bay_id: row.bay_id,
            bay_code: row.bay_code,
            bay_name: row.bay_name,
            status: row.status,
            complaint: row.complaint,
            diagnosis: row.diagnosis,
            odometer_in: row.odometer_in,
            odometer_out: row.odometer_out,
            estimated_completion: row.estimated_completion.map(|value| value.to_rfc3339()),
            mechanic_id: row.mechanic_id,
            mechanic_name: row.mechanic_name,
            account_manager_id: row.account_manager_id,
            account_manager_name: row.account_manager_name,
            qa_by: row.qa_by,
            qa_name: row.qa_name,
            qa_cycles: row.qa_cycles,
            created_at: row.created_at.map(|value| value.to_rfc3339()),
            updated_at: row.updated_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobItemResponse {
    pub id: String,
    pub job_card_id: String,
    pub item_type: String,
    pub description: String,
    pub quantity: String,
    pub unit_price: String,
    pub discount_pct: String,
    pub line_total: String,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct JobItemRow {
    pub id: String,
    pub job_card_id: String,
    pub item_type: String,
    pub description: String,
    pub quantity: String,
    pub unit_price: String,
    pub discount_pct: String,
    pub line_total: String,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<JobItemRow> for JobItemResponse {
    fn from(row: JobItemRow) -> Self {
        Self {
            id: row.id,
            job_card_id: row.job_card_id,
            item_type: row.item_type,
            description: row.description,
            quantity: row.quantity,
            unit_price: row.unit_price,
            discount_pct: row.discount_pct,
            line_total: row.line_total,
            created_at: row.created_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityResponse {
    pub id: String,
    pub job_card_id: String,
    pub action: String,
    pub description: Option<String>,
    pub metadata: Option<Value>,
    pub performed_by: Option<String>,
    pub performed_by_name: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct ActivityRow {
    pub id: String,
    pub job_card_id: String,
    pub action: String,
    pub description: Option<String>,
    pub metadata: Option<Value>,
    pub performed_by: Option<String>,
    pub performed_by_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<ActivityRow> for ActivityResponse {
    fn from(row: ActivityRow) -> Self {
        Self {
            id: row.id,
            job_card_id: row.job_card_id,
            action: row.action,
            description: row.description,
            metadata: row.metadata,
            performed_by: row.performed_by,
            performed_by_name: row.performed_by_name,
            created_at: row.created_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalResponse {
    pub id: String,
    pub job_card_id: String,
    pub approved_by: Option<String>,
    pub approved_by_name: Option<String>,
    pub approval_type: String,
    pub channel: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct ApprovalRow {
    pub id: String,
    pub job_card_id: String,
    pub approved_by: Option<String>,
    pub approved_by_name: Option<String>,
    pub approval_type: String,
    pub channel: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<ApprovalRow> for ApprovalResponse {
    fn from(row: ApprovalRow) -> Self {
        Self {
            id: row.id,
            job_card_id: row.job_card_id,
            approved_by: row.approved_by,
            approved_by_name: row.approved_by_name,
            approval_type: row.approval_type,
            channel: row.channel,
            notes: row.notes,
            created_at: row.created_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRequestItemResponse {
    pub id: String,
    pub change_request_id: String,
    pub description: String,
    pub quantity: String,
    pub unit_price: String,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct ChangeRequestItemRow {
    pub id: String,
    pub change_request_id: String,
    pub description: String,
    pub quantity: String,
    pub unit_price: String,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<ChangeRequestItemRow> for ChangeRequestItemResponse {
    fn from(row: ChangeRequestItemRow) -> Self {
        Self {
            id: row.id,
            change_request_id: row.change_request_id,
            description: row.description,
            quantity: row.quantity,
            unit_price: row.unit_price,
            created_at: row.created_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRequestResponse {
    pub id: String,
    pub job_card_id: String,
    pub status: String,
    pub requested_by: Option<String>,
    pub approved_by: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
    pub resolved_at: Option<String>,
    pub items: Vec<ChangeRequestItemResponse>,
}

#[derive(Debug, FromRow)]
pub struct ChangeRequestRow {
    pub id: String,
    pub job_card_id: String,
    pub status: String,
    pub requested_by: Option<String>,
    pub approved_by: Option<String>,
    pub notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl ChangeRequestResponse {
    pub fn from_parts(row: ChangeRequestRow, items: Vec<ChangeRequestItemResponse>) -> Self {
        Self {
            id: row.id,
            job_card_id: row.job_card_id,
            status: row.status,
            requested_by: row.requested_by,
            approved_by: row.approved_by,
            notes: row.notes,
            created_at: row.created_at.map(|value| value.to_rfc3339()),
            resolved_at: row.resolved_at.map(|value| value.to_rfc3339()),
            items,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QaHistoryResponse {
    pub id: String,
    pub result: String,
    pub notes: Option<String>,
    pub metadata: Option<Value>,
    pub performed_by: Option<String>,
    pub performed_by_name: Option<String>,
    pub created_at: Option<String>,
}

impl From<ActivityRow> for QaHistoryResponse {
    fn from(row: ActivityRow) -> Self {
        Self {
            id: row.id,
            result: if row.action == "qa.failed" {
                "FAILED".to_string()
            } else {
                "PASSED".to_string()
            },
            notes: row.description,
            metadata: row.metadata,
            performed_by: row.performed_by,
            performed_by_name: row.performed_by_name,
            created_at: row.created_at.map(|value| value.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QaContextResponse {
    pub job: JobResponse,
    pub items: Vec<JobItemResponse>,
    pub change_requests: Vec<ChangeRequestResponse>,
    pub qa_history: Vec<QaHistoryResponse>,
}

#[derive(Debug, FromRow)]
pub struct UserSummaryRow {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, FromRow)]
pub struct BaySummaryRow {
    pub id: String,
    pub code: String,
    pub name: String,
    pub capacity: i32,
}

#[derive(Debug, FromRow)]
pub struct LockedJobRow {
    pub id: String,
    pub status: String,
    pub complaint: Option<String>,
    pub odometer_out: Option<i32>,
    pub mechanic_id: Option<String>,
    pub bay_id: Option<String>,
    pub qa_by: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobRequest {
    #[validate(length(min = 1))]
    pub customer_id: String,
    #[validate(length(min = 1))]
    pub vehicle_id: String,
    pub complaint: Option<String>,
    pub diagnosis: Option<String>,
    pub odometer_in: Option<i32>,
    pub estimated_completion: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateJobRequest {
    pub customer_id: Option<String>,
    pub vehicle_id: Option<String>,
    pub complaint: Option<String>,
    pub diagnosis: Option<String>,
    pub odometer_in: Option<i32>,
    pub odometer_out: Option<i32>,
    pub estimated_completion: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpsertIntakeChecklistRequest {
    pub template_id: Option<String>,
    pub data: Value,
    pub completed: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UploadIntakePhotoRequest {
    #[validate(length(min = 1))]
    pub photo_type: String,
    #[validate(length(min = 1))]
    pub file_name: String,
    #[validate(length(min = 1))]
    pub mime_type: String,
    #[validate(length(min = 1))]
    pub image_base64: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpsertCustomerSignatureRequest {
    #[validate(length(min = 1))]
    pub signature_type: String,
    #[validate(length(min = 1))]
    pub signed_by: String,
    #[validate(length(min = 1))]
    pub mime_type: String,
    #[validate(length(min = 1))]
    pub image_base64: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ListQuery {
    pub page: i64,
    pub limit: i64,
    pub search: Option<String>,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
            search: None,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransitionJobRequest {
    #[validate(length(min = 1))]
    pub status: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntakeChecklistResponse {
    pub id: String,
    pub template_id: Option<String>,
    pub data: Value,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntakePhotoResponse {
    pub id: String,
    pub photo_type: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub uploaded_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerSignatureResponse {
    pub id: String,
    pub signature_type: String,
    pub file_path: String,
    pub signed_by: Option<String>,
    pub signed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntakeSnapshotResponse {
    pub checklist: Option<IntakeChecklistResponse>,
    pub photos: Vec<IntakePhotoResponse>,
    pub signature: Option<CustomerSignatureResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntakeSignedPhotoUrlResponse {
    pub photo_id: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntakeSignedUrlsResponse {
    pub photos: Vec<IntakeSignedPhotoUrlResponse>,
    pub signature_url: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, FromRow)]
pub struct IntakeChecklistRow {
    pub id: String,
    pub template_id: Option<String>,
    pub data: Value,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct IntakePhotoRow {
    pub id: String,
    pub photo_type: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub uploaded_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct CustomerSignatureRow {
    pub id: String,
    pub signature_type: String,
    pub file_path: String,
    pub signed_by: Option<String>,
    pub signed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<IntakeChecklistRow> for IntakeChecklistResponse {
    fn from(row: IntakeChecklistRow) -> Self {
        Self {
            id: row.id,
            template_id: row.template_id,
            data: row.data,
            completed_at: row.completed_at.map(|value| value.to_rfc3339()),
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl From<IntakePhotoRow> for IntakePhotoResponse {
    fn from(row: IntakePhotoRow) -> Self {
        Self {
            id: row.id,
            photo_type: row.photo_type,
            file_path: row.file_path,
            thumbnail_path: row.thumbnail_path,
            uploaded_by: row.uploaded_by,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl From<CustomerSignatureRow> for CustomerSignatureResponse {
    fn from(row: CustomerSignatureRow) -> Self {
        Self {
            id: row.id,
            signature_type: row.signature_type,
            file_path: row.file_path,
            signed_by: row.signed_by,
            signed_at: row.signed_at.map(|value| value.to_rfc3339()),
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CancelJobRequest {
    #[validate(length(min = 1))]
    pub reason: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AssignUserRequest {
    #[validate(length(min = 1))]
    pub user_id: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AssignBayRequest {
    #[validate(length(min = 1))]
    pub bay_id: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EstimatedCompletionRequest {
    #[validate(length(min = 1))]
    pub estimated_completion: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct JobItemRequest {
    #[validate(length(min = 1))]
    pub item_type: String,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(length(min = 1))]
    pub quantity: String,
    #[validate(length(min = 1))]
    pub unit_price: String,
    pub discount_pct: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AddNoteRequest {
    #[validate(length(min = 1))]
    pub note: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequest {
    #[validate(length(min = 1))]
    pub approval_type: String,
    pub channel: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ChangeRequestItemRequest {
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(length(min = 1))]
    pub quantity: String,
    #[validate(length(min = 1))]
    pub unit_price: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeRequestRequest {
    pub notes: Option<String>,
    #[validate(length(min = 1))]
    #[validate(nested)]
    pub items: Vec<ChangeRequestItemRequest>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChangeRequestRequest {
    #[validate(length(min = 1))]
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QaSubmitRequest {
    pub passed: bool,
    pub notes: Option<String>,
}
