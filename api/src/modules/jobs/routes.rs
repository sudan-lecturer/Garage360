use axum::{
    extract::{Path, Query},
    routing::{delete, get, post, put},
    Json, Router,
};
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::{auth::AuthUser, tenant::TenantDbPool};
use crate::AppState;

use super::{
    service,
    types::{
        AddNoteRequest, ApprovalRequest, AssignBayRequest, AssignUserRequest, CancelJobRequest,
        CreateChangeRequestRequest, CreateJobRequest, EstimatedCompletionRequest, JobItemRequest,
        ListQuery, QaSubmitRequest, TransitionJobRequest, UpdateChangeRequestRequest,
        UpdateJobRequest,
    },
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/jobs", get(list))
        .route("/jobs", post(create))
        .route("/jobs/search", get(search))
        .route("/jobs/export", get(export))
        .route("/jobs/:id", get(show))
        .route("/jobs/:id", put(update))
        .route("/jobs/:id/cancel", post(cancel))
        .route("/jobs/:id/transition", post(transition))
        .route("/jobs/:id/assign-mechanic", put(assign_mechanic))
        .route("/jobs/:id/assign-bay", put(assign_bay))
        .route("/jobs/:id/assign-account-manager", put(assign_account_manager))
        .route("/jobs/:id/estimated-completion", put(set_estimated_completion))
        .route("/jobs/:id/items", get(list_items))
        .route("/jobs/:id/items", post(create_item))
        .route("/jobs/:id/items/:item_id", put(update_item))
        .route("/jobs/:id/items/:item_id", delete(remove_item))
        .route("/jobs/:id/activities", get(list_activities))
        .route("/jobs/:id/activities/note", post(add_note))
        .route("/jobs/:id/change-requests", get(list_change_requests))
        .route("/jobs/:id/change-requests", post(create_change_request))
        .route("/jobs/:id/change-requests/:change_request_id", put(update_change_request))
        .route("/jobs/:id/approvals", get(list_approvals))
        .route("/jobs/:id/approvals", post(create_approval))
        .route("/jobs/:id/qa/context", get(qa_context))
        .route("/jobs/:id/qa/submit", post(submit_qa))
        .route("/jobs/:id/qa/history", get(qa_history))
}

async fn list(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let page = query.page.max(1);
    let limit = query.limit.clamp(1, 100);
    let search = query.search.unwrap_or_default();

    Ok(Json(service::list_jobs(&tenant_db.pool, page, limit, search).await?))
}

async fn search(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<super::types::JobResponse>>> {
    let search = query.search.unwrap_or_default();
    Ok(Json(service::search_jobs(&tenant_db.pool, search).await?))
}

async fn export(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let search = query.search.unwrap_or_default();
    Ok(Json(service::export_jobs(&tenant_db.pool, search).await?))
}

async fn show(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<super::types::JobResponse>> {
    Ok(Json(service::get_job(&tenant_db.pool, &id).await?))
}

async fn create(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Json(req): Json<CreateJobRequest>,
) -> AppResult<Json<super::types::JobResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_job(&tenant_db.pool, &req, &auth.user_id).await?,
    ))
}

async fn update(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateJobRequest>,
) -> AppResult<Json<super::types::JobResponse>> {
    Ok(Json(
        service::update_job(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn cancel(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CancelJobRequest>,
) -> AppResult<Json<super::types::JobResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::cancel_job(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn transition(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<TransitionJobRequest>,
) -> AppResult<Json<super::types::JobResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::transition_job(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn assign_mechanic(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<AssignUserRequest>,
) -> AppResult<Json<super::types::JobResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::assign_mechanic(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn assign_bay(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<AssignBayRequest>,
) -> AppResult<Json<super::types::JobResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::assign_bay(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn assign_account_manager(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<AssignUserRequest>,
) -> AppResult<Json<super::types::JobResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::assign_account_manager(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn set_estimated_completion(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<EstimatedCompletionRequest>,
) -> AppResult<Json<super::types::JobResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::set_estimated_completion(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn list_items(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<super::types::JobItemResponse>>> {
    Ok(Json(service::list_job_items(&tenant_db.pool, &id).await?))
}

async fn create_item(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<JobItemRequest>,
) -> AppResult<Json<super::types::JobItemResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_job_item(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn update_item(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path((id, item_id)): Path<(String, String)>,
    Json(req): Json<JobItemRequest>,
) -> AppResult<Json<super::types::JobItemResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_job_item(&tenant_db.pool, &id, &item_id, &req, &auth.user_id).await?,
    ))
}

async fn remove_item(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path((id, item_id)): Path<(String, String)>,
) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(
        service::delete_job_item(&tenant_db.pool, &id, &item_id, &auth.user_id).await?,
    ))
}

async fn list_activities(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<super::types::ActivityResponse>>> {
    Ok(Json(service::list_job_activities(&tenant_db.pool, &id).await?))
}

async fn add_note(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<AddNoteRequest>,
) -> AppResult<Json<super::types::ActivityResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::add_job_note(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn list_change_requests(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<super::types::ChangeRequestResponse>>> {
    Ok(Json(
        service::list_job_change_requests(&tenant_db.pool, &id).await?,
    ))
}

async fn create_change_request(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<CreateChangeRequestRequest>,
) -> AppResult<Json<super::types::ChangeRequestResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_job_change_request(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn update_change_request(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path((id, change_request_id)): Path<(String, String)>,
    Json(req): Json<UpdateChangeRequestRequest>,
) -> AppResult<Json<super::types::ChangeRequestResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::update_job_change_request(
            &tenant_db.pool,
            &id,
            &change_request_id,
            &req,
            &auth.user_id,
        )
        .await?,
    ))
}

async fn list_approvals(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<super::types::ApprovalResponse>>> {
    Ok(Json(service::list_job_approvals(&tenant_db.pool, &id).await?))
}

async fn create_approval(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<ApprovalRequest>,
) -> AppResult<Json<super::types::ApprovalResponse>> {
    req.validate()
        .map_err(|err| AppError::Validation(err.to_string()))?;

    Ok(Json(
        service::create_job_approval(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn qa_context(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<super::types::QaContextResponse>> {
    Ok(Json(service::get_qa_context(&tenant_db.pool, &id).await?))
}

async fn submit_qa(
    tenant_db: TenantDbPool,
    auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<QaSubmitRequest>,
) -> AppResult<Json<super::types::JobResponse>> {
    Ok(Json(
        service::submit_qa(&tenant_db.pool, &id, &req, &auth.user_id).await?,
    ))
}

async fn qa_history(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
) -> AppResult<Json<Vec<super::types::QaHistoryResponse>>> {
    Ok(Json(service::get_qa_history(&tenant_db.pool, &id).await?))
}
