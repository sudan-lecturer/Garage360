use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use crate::errors::{AppError, AppResult};
use crate::common::pagination::PaginationMeta;

use super::{
    repo,
    types::{
        ActivityResponse, AddNoteRequest, ApprovalRequest, ApprovalResponse, AssignBayRequest,
        AssignUserRequest, CancelJobRequest, ChangeRequestResponse, CreateChangeRequestRequest,
        CreateJobRequest, EstimatedCompletionRequest, JobItemRequest, JobItemResponse,
        JobResponse, QaContextResponse, QaHistoryResponse, QaSubmitRequest,
        TransitionJobRequest, UpdateChangeRequestRequest, UpdateJobRequest,
    },
};

pub async fn list_jobs(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);

    let jobs = repo::list(pool, &search, &like, limit, offset).await?;
    let total = repo::count(pool, &search, &like).await?;

    Ok(json!({
        "data": jobs.into_iter().map(JobResponse::from).collect::<Vec<_>>(),
        "meta": crate::common::pagination::PaginationMeta { page, limit, total }
    }))
}

pub async fn search_jobs(pool: &PgPool, search: String) -> AppResult<Vec<JobResponse>> {
    let like = format!("%{}%", search);
    let jobs = repo::list(pool, &search, &like, 20, 0).await?;
    Ok(jobs.into_iter().map(JobResponse::from).collect())
}

pub async fn export_jobs(pool: &PgPool, search: String) -> AppResult<serde_json::Value> {
    let like = format!("%{}%", search);
    let jobs = repo::list(pool, &search, &like, 10_000, 0).await?;

    Ok(json!({
        "data": jobs.into_iter().map(JobResponse::from).collect::<Vec<_>>()
    }))
}

pub async fn get_job(pool: &PgPool, id: &str) -> AppResult<JobResponse> {
    repo::find_by_id(pool, id)
        .await?
        .map(JobResponse::from)
        .ok_or_else(|| AppError::NotFound("Job not found".into()))
}

pub async fn create_job(
    pool: &PgPool,
    req: &CreateJobRequest,
    created_by: &str,
) -> AppResult<JobResponse> {
    ensure_customer_and_vehicle(pool, &req.customer_id, &req.vehicle_id).await?;
    let estimated_completion = parse_optional_datetime(req.estimated_completion.as_deref())?;

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let job_id = repo::create_job(&mut tx, req, created_by, estimated_completion).await?;

    repo::insert_activity(
        &mut tx,
        &job_id,
        "job.created",
        Some("Job card created".to_string()),
        Some(json!({
            "customerId": &req.customer_id,
            "vehicleId": &req.vehicle_id,
            "status": "INTAKE"
        })),
        Some(created_by),
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;
    get_job(pool, &job_id).await
}

pub async fn update_job(
    pool: &PgPool,
    id: &str,
    req: &UpdateJobRequest,
    updated_by: &str,
) -> AppResult<JobResponse> {
    let existing = repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".into()))?;

    let customer_id = req.customer_id.clone().unwrap_or(existing.customer_id.clone());
    let vehicle_id = req.vehicle_id.clone().unwrap_or(existing.vehicle_id.clone());
    ensure_customer_and_vehicle(pool, &customer_id, &vehicle_id).await?;

    let complaint = req.complaint.clone().or(existing.complaint.clone());
    let diagnosis = req.diagnosis.clone().or(existing.diagnosis.clone());
    let odometer_in = req.odometer_in.or(existing.odometer_in);
    let odometer_out = req.odometer_out.or(existing.odometer_out);

    let estimated_completion = match req.estimated_completion.as_deref() {
        Some(value) => parse_datetime(value).map(Some)?,
        None => existing.estimated_completion,
    };

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let updated = repo::update_job(
        &mut tx,
        id,
        &customer_id,
        &vehicle_id,
        complaint.as_deref(),
        diagnosis.as_deref(),
        odometer_in,
        odometer_out,
        estimated_completion,
    )
    .await?;

    if updated == 0 {
        return Err(AppError::NotFound("Job not found".into()));
    }

    repo::insert_activity(
        &mut tx,
        id,
        "job.updated",
        Some("Job card updated".to_string()),
        None,
        Some(updated_by),
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;
    get_job(pool, id).await
}

pub async fn cancel_job(
    pool: &PgPool,
    id: &str,
    req: &CancelJobRequest,
    performed_by: &str,
) -> AppResult<JobResponse> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let locked = repo::lock_job_for_update(&mut tx, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".into()))?;

    if locked.status == "COMPLETED" {
        return Err(AppError::Conflict("Completed jobs cannot be cancelled".into()));
    }
    if locked.status == "CANCELLED" {
        return Err(AppError::Conflict("Job is already cancelled".into()));
    }

    repo::update_job_status(&mut tx, id, "CANCELLED").await?;
    repo::insert_activity(
        &mut tx,
        id,
        "job.cancelled",
        Some(req.reason.clone()),
        Some(json!({ "fromStatus": locked.status })),
        Some(performed_by),
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;
    get_job(pool, id).await
}

pub async fn transition_job(
    pool: &PgPool,
    id: &str,
    req: &TransitionJobRequest,
    performed_by: &str,
) -> AppResult<JobResponse> {
    let target_status = normalize_status(&req.status);

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let locked = repo::lock_job_for_update(&mut tx, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".into()))?;

    ensure_transition_allowed(pool, id, &locked, &target_status).await?;
    repo::update_job_status(&mut tx, id, &target_status).await?;
    repo::insert_activity(
        &mut tx,
        id,
        "job.transitioned",
        Some(format!("Status changed from {} to {}", locked.status, target_status)),
        Some(json!({
            "fromStatus": locked.status,
            "toStatus": &target_status,
        })),
        Some(performed_by),
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;
    get_job(pool, id).await
}

pub async fn assign_mechanic(
    pool: &PgPool,
    id: &str,
    req: &AssignUserRequest,
    performed_by: &str,
) -> AppResult<JobResponse> {
    let user = repo::find_user_by_id(pool, &req.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Mechanic user not found".into()))?;

    if user.role != "MECHANIC" {
        return Err(AppError::Validation("Assigned mechanic must have role MECHANIC".into()));
    }

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let locked = repo::lock_job_for_update(&mut tx, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".into()))?;

    repo::set_mechanic(&mut tx, id, &req.user_id).await?;
    repo::insert_activity(
        &mut tx,
        id,
        "job.mechanic_assigned",
        Some(format!("Mechanic assigned to {}", user.name)),
        Some(json!({
            "mechanicId": &req.user_id,
            "previousMechanicId": locked.mechanic_id,
        })),
        Some(performed_by),
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;
    get_job(pool, id).await
}

pub async fn assign_account_manager(
    pool: &PgPool,
    id: &str,
    req: &AssignUserRequest,
    performed_by: &str,
) -> AppResult<JobResponse> {
    let user = repo::find_user_by_id(pool, &req.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Account manager user not found".into()))?;

    if !matches!(
        user.role.as_str(),
        "ACCOUNT_MGR" | "MANAGER" | "ADMIN" | "OWNER"
    ) {
        return Err(AppError::Validation(
            "Assigned account manager must be ACCOUNT_MGR, MANAGER, ADMIN, or OWNER".into(),
        ));
    }

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::lock_job_for_update(&mut tx, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".into()))?;

    repo::set_account_manager(&mut tx, id, &req.user_id).await?;
    repo::insert_activity(
        &mut tx,
        id,
        "job.account_manager_assigned",
        Some(format!("Account manager assigned to {}", user.name)),
        Some(json!({ "accountManagerId": &req.user_id })),
        Some(performed_by),
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;
    get_job(pool, id).await
}

pub async fn assign_bay(
    pool: &PgPool,
    id: &str,
    req: &AssignBayRequest,
    performed_by: &str,
) -> AppResult<JobResponse> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::lock_job_for_update(&mut tx, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".into()))?;

    let bay = repo::lock_bay_for_update(&mut tx, &req.bay_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Bay not found".into()))?;

    if let Some(conflicting_job_id) = repo::find_conflicting_job_in_bay(&mut tx, &req.bay_id, id).await? {
        return Err(AppError::Conflict(format!(
            "Bay {} is already occupied by another active job ({conflicting_job_id})",
            bay.code
        )));
    }

    repo::set_bay(&mut tx, id, &req.bay_id).await?;
    repo::insert_activity(
        &mut tx,
        id,
        "job.bay_assigned",
        Some(format!("Assigned to bay {}", bay.code)),
        Some(json!({
            "bayId": &req.bay_id,
            "bayCode": bay.code,
        })),
        Some(performed_by),
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;
    get_job(pool, id).await
}

pub async fn set_estimated_completion(
    pool: &PgPool,
    id: &str,
    req: &EstimatedCompletionRequest,
    performed_by: &str,
) -> AppResult<JobResponse> {
    let estimated_completion = parse_datetime(&req.estimated_completion)?;

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::lock_job_for_update(&mut tx, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".into()))?;

    repo::set_estimated_completion(&mut tx, id, estimated_completion).await?;
    repo::insert_activity(
        &mut tx,
        id,
        "job.estimated_completion_set",
        Some("Estimated completion updated".to_string()),
        Some(json!({ "estimatedCompletion": estimated_completion.to_rfc3339() })),
        Some(performed_by),
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;
    get_job(pool, id).await
}

pub async fn list_job_items(pool: &PgPool, id: &str) -> AppResult<Vec<JobItemResponse>> {
    ensure_job_exists(pool, id).await?;
    let items = repo::list_items(pool, id).await?;
    Ok(items.into_iter().map(JobItemResponse::from).collect())
}

pub async fn create_job_item(
    pool: &PgPool,
    id: &str,
    req: &JobItemRequest,
    performed_by: &str,
) -> AppResult<JobItemResponse> {
    ensure_job_exists(pool, id).await?;
    ensure_valid_item_type(&req.item_type)?;
    let item = repo::create_item(pool, id, req).await?;

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::insert_activity(
        &mut tx,
        id,
        "job.item_added",
        Some(format!("Added {}", item.description)),
        Some(json!({
            "itemId": &item.id,
            "itemType": &item.item_type,
            "quantity": &item.quantity,
            "unitPrice": &item.unit_price,
        })),
        Some(performed_by),
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    Ok(JobItemResponse::from(item))
}

pub async fn update_job_item(
    pool: &PgPool,
    id: &str,
    item_id: &str,
    req: &JobItemRequest,
    performed_by: &str,
) -> AppResult<JobItemResponse> {
    ensure_job_exists(pool, id).await?;
    ensure_valid_item_type(&req.item_type)?;

    let item = repo::update_item(pool, id, item_id, req)
        .await?
        .ok_or_else(|| AppError::NotFound("Job item not found".into()))?;

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::insert_activity(
        &mut tx,
        id,
        "job.item_updated",
        Some(format!("Updated {}", item.description)),
        Some(json!({ "itemId": &item.id })),
        Some(performed_by),
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    Ok(JobItemResponse::from(item))
}

pub async fn delete_job_item(
    pool: &PgPool,
    id: &str,
    item_id: &str,
    performed_by: &str,
) -> AppResult<serde_json::Value> {
    let item = repo::find_item(pool, id, item_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job item not found".into()))?;

    let deleted = repo::soft_delete_item(pool, id, item_id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("Job item not found".into()));
    }

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::insert_activity(
        &mut tx,
        id,
        "job.item_removed",
        Some(format!("Removed {}", item.description)),
        Some(json!({ "itemId": &item.id })),
        Some(performed_by),
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    Ok(json!({ "deleted": true }))
}

pub async fn list_job_activities(pool: &PgPool, id: &str) -> AppResult<Vec<ActivityResponse>> {
    ensure_job_exists(pool, id).await?;
    let activities = repo::list_activities(pool, id).await?;
    Ok(activities.into_iter().map(ActivityResponse::from).collect())
}

pub async fn add_job_note(
    pool: &PgPool,
    id: &str,
    req: &AddNoteRequest,
    performed_by: &str,
) -> AppResult<ActivityResponse> {
    ensure_job_exists(pool, id).await?;

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let activity_id = repo::insert_activity(
        &mut tx,
        id,
        "note.added",
        Some(req.note.clone()),
        None,
        Some(performed_by),
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    let activity = repo::find_activity_by_id(pool, id, &activity_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Activity not found".into()))?;

    Ok(ActivityResponse::from(activity))
}

pub async fn list_job_approvals(pool: &PgPool, id: &str) -> AppResult<Vec<ApprovalResponse>> {
    ensure_job_exists(pool, id).await?;
    let approvals = repo::list_approvals(pool, id).await?;
    Ok(approvals.into_iter().map(ApprovalResponse::from).collect())
}

pub async fn create_job_approval(
    pool: &PgPool,
    id: &str,
    req: &ApprovalRequest,
    performed_by: &str,
) -> AppResult<ApprovalResponse> {
    ensure_job_exists(pool, id).await?;

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let approval_id = repo::create_approval(&mut tx, id, performed_by, req).await?;
    repo::insert_activity(
        &mut tx,
        id,
        "approval.recorded",
        Some(req.notes.clone().unwrap_or_else(|| "Approval recorded".to_string())),
        Some(json!({
            "approvalId": &approval_id,
            "approvalType": &req.approval_type,
            "channel": &req.channel,
        })),
        Some(performed_by),
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    let approval = repo::find_approval_by_id(pool, id, &approval_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Approval not found".into()))?;

    Ok(ApprovalResponse::from(approval))
}

pub async fn list_job_change_requests(
    pool: &PgPool,
    id: &str,
) -> AppResult<Vec<ChangeRequestResponse>> {
    ensure_job_exists(pool, id).await?;
    let change_requests = repo::list_change_requests(pool, id).await?;

    let mut response = Vec::with_capacity(change_requests.len());
    for row in change_requests {
        let items = repo::list_change_request_items(pool, &row.id).await?;
        response.push(ChangeRequestResponse::from_parts(
            row,
            items.into_iter().map(super::types::ChangeRequestItemResponse::from).collect(),
        ));
    }

    Ok(response)
}

pub async fn create_job_change_request(
    pool: &PgPool,
    id: &str,
    req: &CreateChangeRequestRequest,
    performed_by: &str,
) -> AppResult<ChangeRequestResponse> {
    ensure_job_exists(pool, id).await?;

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let change_request_id = repo::create_change_request(&mut tx, id, performed_by, req).await?;

    for item in &req.items {
        repo::create_change_request_item(&mut tx, &change_request_id, item).await?;
    }

    repo::insert_activity(
        &mut tx,
        id,
        "change_request.created",
        Some(req.notes.clone().unwrap_or_else(|| "Change request created".to_string())),
        Some(json!({
            "changeRequestId": &change_request_id,
            "itemCount": req.items.len(),
        })),
        Some(performed_by),
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    get_change_request(pool, id, &change_request_id).await
}

pub async fn update_job_change_request(
    pool: &PgPool,
    job_id: &str,
    change_request_id: &str,
    req: &UpdateChangeRequestRequest,
    performed_by: &str,
) -> AppResult<ChangeRequestResponse> {
    ensure_job_exists(pool, job_id).await?;
    let next_status = normalize_change_request_status(&req.status)?;

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let existing = repo::lock_change_request_for_update(&mut tx, job_id, change_request_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Change request not found".into()))?;

    if existing.status != "PENDING" {
        return Err(AppError::Conflict(
            "Only pending change requests can be updated".into(),
        ));
    }

    let items = repo::list_change_request_items_tx(&mut tx, change_request_id).await?;
    repo::update_change_request_status(
        &mut tx,
        change_request_id,
        &next_status,
        performed_by,
        req.notes.as_deref(),
    )
    .await?;

    if next_status == "APPROVED" {
        for item in &items {
            repo::insert_item_from_change_request(&mut tx, job_id, item).await?;
        }
    }

    repo::insert_activity(
        &mut tx,
        job_id,
        "change_request.updated",
        Some(req.notes.clone().unwrap_or_else(|| format!("Change request {next_status}"))),
        Some(json!({
            "changeRequestId": change_request_id,
            "status": &next_status,
            "itemCount": items.len(),
        })),
        Some(performed_by),
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    get_change_request(pool, job_id, change_request_id).await
}

pub async fn get_qa_context(pool: &PgPool, id: &str) -> AppResult<QaContextResponse> {
    let job = get_job(pool, id).await?;
    let items = list_job_items(pool, id).await?;
    let change_requests = list_job_change_requests(pool, id).await?;
    let qa_history = get_qa_history(pool, id).await?;

    Ok(QaContextResponse {
        job,
        items,
        change_requests,
        qa_history,
    })
}

pub async fn submit_qa(
    pool: &PgPool,
    id: &str,
    req: &QaSubmitRequest,
    performed_by: &str,
) -> AppResult<JobResponse> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let locked = repo::lock_job_for_update(&mut tx, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".into()))?;

    if locked.status != "QA" {
        return Err(AppError::Conflict(
            "QA can only be submitted when the job is in QA status".into(),
        ));
    }

    let next_status = if req.passed { "BILLING" } else { "IN_SERVICE" };
    let action = if req.passed { "qa.passed" } else { "qa.failed" };
    repo::apply_qa_result(&mut tx, id, next_status, performed_by, !req.passed).await?;
    repo::insert_activity(
        &mut tx,
        id,
        action,
        req.notes.clone(),
        Some(json!({ "nextStatus": next_status })),
        Some(performed_by),
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    get_job(pool, id).await
}

pub async fn get_qa_history(pool: &PgPool, id: &str) -> AppResult<Vec<QaHistoryResponse>> {
    ensure_job_exists(pool, id).await?;
    let history = repo::list_qa_activities(pool, id).await?;
    Ok(history.into_iter().map(QaHistoryResponse::from).collect())
}

async fn get_change_request(
    pool: &PgPool,
    job_id: &str,
    change_request_id: &str,
) -> AppResult<ChangeRequestResponse> {
    let row = repo::find_change_request(pool, job_id, change_request_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Change request not found".into()))?;
    let items = repo::list_change_request_items(pool, change_request_id).await?;

    Ok(ChangeRequestResponse::from_parts(
        row,
        items
            .into_iter()
            .map(super::types::ChangeRequestItemResponse::from)
            .collect(),
    ))
}

async fn ensure_job_exists(pool: &PgPool, id: &str) -> AppResult<()> {
    if repo::find_by_id(pool, id).await?.is_some() {
        Ok(())
    } else {
        Err(AppError::NotFound("Job not found".into()))
    }
}

async fn ensure_customer_and_vehicle(pool: &PgPool, customer_id: &str, vehicle_id: &str) -> AppResult<()> {
    if !repo::customer_exists(pool, customer_id).await? {
        return Err(AppError::Validation("Customer does not exist".into()));
    }

    if !repo::vehicle_exists_for_customer(pool, vehicle_id, customer_id).await? {
        return Err(AppError::Validation(
            "Vehicle does not exist or does not belong to the customer".into(),
        ));
    }

    Ok(())
}

async fn ensure_transition_allowed(
    pool: &PgPool,
    job_id: &str,
    current: &super::types::LockedJobRow,
    target_status: &str,
) -> AppResult<()> {
    if current.status == target_status {
        return Ok(());
    }

    if current.status == "COMPLETED" || current.status == "CANCELLED" {
        return Err(AppError::Conflict(format!(
            "Cannot transition a {} job",
            current.status.to_lowercase()
        )));
    }

    match (current.status.as_str(), target_status) {
        ("INTAKE", "AUDIT") => {
            if !repo::has_completed_intake_checklist(pool, job_id).await? {
                return Err(AppError::Validation(
                    "Intake checklist must be completed before moving to AUDIT".into(),
                ));
            }
            if !repo::has_customer_signature(pool, job_id).await? {
                return Err(AppError::Validation(
                    "Customer signature is required before moving to AUDIT".into(),
                ));
            }
        }
        ("AUDIT", "QUOTE") => {
            if current
                .complaint
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none()
            {
                return Err(AppError::Validation(
                    "Complaint is required before moving to QUOTE".into(),
                ));
            }
            if current.mechanic_id.is_none() {
                return Err(AppError::Validation(
                    "Mechanic must be assigned before moving to QUOTE".into(),
                ));
            }
            if current.bay_id.is_none() {
                return Err(AppError::Validation(
                    "Bay must be assigned before moving to QUOTE".into(),
                ));
            }
        }
        ("QUOTE", "APPROVAL") | ("QUOTE", "IN_SERVICE") => {
            if repo::active_item_count(pool, job_id).await? == 0 {
                return Err(AppError::Validation(
                    "At least one active line item is required before progressing from QUOTE"
                        .into(),
                ));
            }
        }
        ("APPROVAL", "IN_SERVICE") => {
            if !repo::has_approval(pool, job_id).await? {
                return Err(AppError::Validation(
                    "Approval must be recorded before moving to IN_SERVICE".into(),
                ));
            }
        }
        ("IN_SERVICE", "QA") => {
            if current.mechanic_id.is_none() {
                return Err(AppError::Validation(
                    "Mechanic must be assigned before moving to QA".into(),
                ));
            }
        }
        ("QA", "BILLING") => {
            if current.qa_by.is_none() {
                return Err(AppError::Validation(
                    "QA must be submitted before moving to BILLING".into(),
                ));
            }
        }
        ("BILLING", "COMPLETED") => {
            if !repo::has_paid_invoice(pool, job_id).await? {
                return Err(AppError::Validation(
                    "A paid invoice is required before moving to COMPLETED".into(),
                ));
            }
        }
        _ => {
            return Err(AppError::Validation(format!(
                "Invalid transition from {} to {}",
                current.status, target_status
            )));
        }
    }

    Ok(())
}

fn normalize_status(status: &str) -> String {
    status.trim().to_ascii_uppercase()
}

fn normalize_change_request_status(status: &str) -> AppResult<String> {
    let status = normalize_status(status);
    if status == "APPROVED" || status == "DECLINED" {
        Ok(status)
    } else {
        Err(AppError::Validation(
            "Change request status must be APPROVED or DECLINED".into(),
        ))
    }
}

fn ensure_valid_item_type(item_type: &str) -> AppResult<()> {
    let item_type = normalize_status(item_type);
    if item_type == "PART" || item_type == "LABOUR" {
        Ok(())
    } else {
        Err(AppError::Validation(
            "Job item type must be PART or LABOUR".into(),
        ))
    }
}

fn parse_optional_datetime(value: Option<&str>) -> AppResult<Option<DateTime<Utc>>> {
    value.map(parse_datetime).transpose()
}

fn parse_datetime(value: &str) -> AppResult<DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|datetime| datetime.with_timezone(&Utc))
        .map_err(|_| AppError::Validation("Expected RFC3339 datetime".into()))
}
