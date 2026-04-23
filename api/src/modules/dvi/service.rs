use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

use super::{
    repo,
    types::{CreateDviTemplateRequest, DviResultRequest, DviResultResponse, DviTemplateResponse},
};

pub async fn list_templates(pool: &PgPool) -> AppResult<Vec<DviTemplateResponse>> {
    let templates = repo::list_templates(pool).await?;
    Ok(templates.into_iter().map(DviTemplateResponse::from).collect())
}

pub async fn create_template(
    pool: &PgPool,
    req: &CreateDviTemplateRequest,
) -> AppResult<DviTemplateResponse> {
    validate_template_sections(&req.sections)?;

    repo::create_template(pool, req)
        .await
        .map(DviTemplateResponse::from)
}

pub async fn update_template(
    pool: &PgPool,
    id: &str,
    req: &CreateDviTemplateRequest,
) -> AppResult<DviTemplateResponse> {
    ensure_uuid(id, "templateId")?;
    validate_template_sections(&req.sections)?;

    repo::update_template(pool, id, req)
        .await?
        .map(DviTemplateResponse::from)
        .ok_or_else(|| AppError::NotFound("DVI template not found".into()))
}

pub async fn delete_template(pool: &PgPool, id: &str) -> AppResult<serde_json::Value> {
    ensure_uuid(id, "templateId")?;

    let rows_affected = repo::soft_delete_template(pool, id).await?;
    if rows_affected == 0 {
        return Err(AppError::NotFound("DVI template not found".into()));
    }

    Ok(json!({ "deleted": true }))
}

pub async fn create_result(
    pool: &PgPool,
    req: &DviResultRequest,
    submitted_by: &str,
) -> AppResult<DviResultResponse> {
    validate_result_request(pool, req).await?;

    let id = repo::create_result(pool, req, submitted_by).await?;
    get_result(pool, &id).await
}

pub async fn get_result(pool: &PgPool, id: &str) -> AppResult<DviResultResponse> {
    ensure_uuid(id, "resultId")?;

    repo::find_result_by_id(pool, id)
        .await?
        .map(DviResultResponse::from)
        .ok_or_else(|| AppError::NotFound("DVI result not found".into()))
}

pub async fn update_result(
    pool: &PgPool,
    id: &str,
    req: &DviResultRequest,
    submitted_by: &str,
) -> AppResult<DviResultResponse> {
    ensure_uuid(id, "resultId")?;
    validate_result_request(pool, req).await?;

    let updated_id = repo::update_result(pool, id, req, submitted_by)
        .await?
        .ok_or_else(|| AppError::NotFound("DVI result not found".into()))?;

    get_result(pool, &updated_id).await
}

pub async fn delete_result(pool: &PgPool, id: &str) -> AppResult<serde_json::Value> {
    ensure_uuid(id, "resultId")?;

    let rows_affected = repo::delete_result(pool, id).await?;
    if rows_affected == 0 {
        return Err(AppError::NotFound("DVI result not found".into()));
    }

    Ok(json!({ "deleted": true }))
}

async fn validate_result_request(pool: &PgPool, req: &DviResultRequest) -> AppResult<()> {
    ensure_uuid(&req.job_card_id, "jobCardId")?;
    if !repo::job_card_exists(pool, &req.job_card_id).await? {
        return Err(AppError::NotFound("Job card not found".into()));
    }

    if let Some(template_id) = req.template_id.as_deref() {
        ensure_uuid(template_id, "templateId")?;
        if !repo::template_exists(pool, template_id).await? {
            return Err(AppError::NotFound("DVI template not found".into()));
        }
    }

    validate_result_data(&req.data)
}

fn validate_template_sections(sections: &serde_json::Value) -> AppResult<()> {
    let Some(items) = sections.as_array() else {
        return Err(AppError::Validation(
            "Template sections must be a JSON array".into(),
        ));
    };

    if items.is_empty() {
        return Err(AppError::Validation(
            "Template sections must contain at least one section".into(),
        ));
    }

    Ok(())
}

fn validate_result_data(data: &serde_json::Value) -> AppResult<()> {
    if !(data.is_object() || data.is_array()) {
        return Err(AppError::Validation(
            "DVI result data must be a JSON object or array".into(),
        ));
    }

    let is_empty_object = data.as_object().is_some_and(|value| value.is_empty());
    let is_empty_array = data.as_array().is_some_and(|value| value.is_empty());
    if is_empty_object || is_empty_array {
        return Err(AppError::Validation(
            "DVI result data cannot be empty".into(),
        ));
    }

    Ok(())
}

fn ensure_uuid(value: &str, field: &str) -> AppResult<()> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| AppError::Validation(format!("{field} must be a valid UUID")))
}
