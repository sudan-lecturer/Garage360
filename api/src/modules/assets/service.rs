use chrono::NaiveDate;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::common::pagination::PaginationMeta;

use super::{
    repo,
    types::{
        AssetDefectResponse, AssetDetailResponse, AssetInspectionRequest, AssetInspectionResponse,
        AssetRequest, AssetResponse, CreateAssetDefectRequest,
        UpdateAssetDefectRequest,
    },
};

pub async fn list_assets(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
    category: String,
    status: String,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);
    let category_like = format!("%{}%", category);

    let assets = repo::list(pool, &search, &like, &category, &category_like, &status, limit, offset).await?;
    let total = repo::count(pool, &search, &like, &category, &category_like, &status).await?;

    Ok(json!({
        "data": assets.into_iter().map(AssetResponse::from).collect::<Vec<_>>(),
        "meta": PaginationMeta { page, limit, total }
    }))
}

pub async fn list_due_inspection_assets(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
    category: String,
    status: String,
    days_since_last: i32,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);
    let category_like = format!("%{}%", category);

    let assets = repo::list_due_inspection(
        pool,
        &search,
        &like,
        &category,
        &category_like,
        &status,
        days_since_last,
        limit,
        offset,
    )
    .await?;
    let total = repo::count_due_inspection(
        pool,
        &search,
        &like,
        &category,
        &category_like,
        &status,
        days_since_last,
    )
    .await?;

    Ok(json!({
        "data": assets.into_iter().map(AssetResponse::from).collect::<Vec<_>>(),
        "meta": PaginationMeta { page, limit, total }
    }))
}

pub async fn list_open_defects(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
    severity: String,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);

    let defects = repo::list_open_defects(pool, &search, &like, &severity, limit, offset).await?;
    let total = repo::count_open_defects(pool, &search, &like, &severity).await?;

    Ok(json!({
        "data": defects.into_iter().map(AssetDefectResponse::from).collect::<Vec<_>>(),
        "meta": PaginationMeta { page, limit, total }
    }))
}

pub async fn get_asset_detail(pool: &PgPool, id: &str) -> AppResult<AssetDetailResponse> {
    ensure_valid_uuid(id, "asset")?;

    let asset = repo::find_asset(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Asset not found".into()))?;
    let inspections = repo::list_asset_inspections(pool, id, 10).await?;
    let defects = repo::list_asset_open_defects(pool, id).await?;

    Ok(AssetDetailResponse {
        asset: AssetResponse::from(asset),
        recent_inspections: inspections
            .into_iter()
            .map(AssetInspectionResponse::from)
            .collect::<Vec<_>>(),
        open_defects: defects
            .into_iter()
            .map(AssetDefectResponse::from)
            .collect::<Vec<_>>(),
    })
}

pub async fn create_asset(pool: &PgPool, req: &AssetRequest) -> AppResult<AssetResponse> {
    let asset_tag = normalize_required_text(&req.asset_tag, "Asset tag")?;
    let name = normalize_required_text(&req.name, "Name")?;
    let category = normalize_optional_text(req.category.as_deref());
    let location_id = normalize_optional_uuid(req.location_id.as_deref(), "location")?;
    let purchase_date = parse_optional_date(req.purchase_date.as_deref(), "purchaseDate")?;
    let purchase_cost = normalize_optional_decimal(req.purchase_cost.as_deref(), 2, true, "Purchase cost")?;
    let useful_life_years = normalize_optional_i32(req.useful_life_years, "Useful life years")?;
    let status = normalize_asset_status(req.status.as_deref())?;
    let notes = normalize_optional_text(req.notes.as_deref());

    let asset_id = Uuid::now_v7().to_string();
    repo::create_asset(
        pool,
        &asset_id,
        &asset_tag,
        &name,
        category.as_deref(),
        location_id.as_deref(),
        purchase_date,
        purchase_cost.as_deref(),
        useful_life_years,
        &status,
        notes.as_deref(),
    )
    .await
    .map(AssetResponse::from)
}

pub async fn update_asset(pool: &PgPool, id: &str, req: &AssetRequest) -> AppResult<AssetResponse> {
    ensure_valid_uuid(id, "asset")?;

    let asset_tag = normalize_required_text(&req.asset_tag, "Asset tag")?;
    let name = normalize_required_text(&req.name, "Name")?;
    let category = normalize_optional_text(req.category.as_deref());
    let location_id = normalize_optional_uuid(req.location_id.as_deref(), "location")?;
    let purchase_date = parse_optional_date(req.purchase_date.as_deref(), "purchaseDate")?;
    let purchase_cost = normalize_optional_decimal(req.purchase_cost.as_deref(), 2, true, "Purchase cost")?;
    let useful_life_years = normalize_optional_i32(req.useful_life_years, "Useful life years")?;
    let status = normalize_asset_status(req.status.as_deref())?;
    let notes = normalize_optional_text(req.notes.as_deref());

    repo::update_asset(
        pool,
        id,
        &asset_tag,
        &name,
        category.as_deref(),
        location_id.as_deref(),
        purchase_date,
        purchase_cost.as_deref(),
        useful_life_years,
        &status,
        notes.as_deref(),
    )
    .await?
    .map(AssetResponse::from)
    .ok_or_else(|| AppError::NotFound("Asset not found".into()))
}

pub async fn inspect_asset(
    pool: &PgPool,
    id: &str,
    req: &AssetInspectionRequest,
    submitted_by: &str,
) -> AppResult<AssetInspectionResponse> {
    ensure_valid_uuid(id, "asset")?;
    ensure_valid_uuid(submitted_by, "user")?;
    ensure_asset_exists(pool, id).await?;

    let template_id = match req.template_id.as_deref() {
        Some(template_id) => {
            ensure_valid_uuid(template_id, "inspection template")?;
            if !repo::active_template_exists(pool, template_id).await? {
                return Err(AppError::NotFound("Active asset inspection template not found".into()));
            }
            Some(template_id.to_string())
        }
        None => None,
    };

    validate_inspection_data(&req.data)?;

    let inspection_id = Uuid::now_v7().to_string();
    repo::insert_inspection(
        pool,
        &inspection_id,
        id,
        template_id.as_deref(),
        &req.data,
        submitted_by,
    )
    .await
    .map(AssetInspectionResponse::from)
}

pub async fn list_asset_inspections(pool: &PgPool, id: &str) -> AppResult<Vec<AssetInspectionResponse>> {
    ensure_valid_uuid(id, "asset")?;
    ensure_asset_exists(pool, id).await?;

    Ok(repo::list_asset_inspections(pool, id, 100)
        .await?
        .into_iter()
        .map(AssetInspectionResponse::from)
        .collect::<Vec<_>>())
}

pub async fn create_asset_defect(
    pool: &PgPool,
    id: &str,
    req: &CreateAssetDefectRequest,
    reported_by: &str,
) -> AppResult<AssetDefectResponse> {
    ensure_valid_uuid(id, "asset")?;
    ensure_valid_uuid(reported_by, "user")?;
    ensure_asset_exists(pool, id).await?;

    let description = normalize_required_text(&req.description, "Description")?;
    let severity = normalize_defect_severity(&req.severity)?;
    let defect_id = Uuid::now_v7().to_string();

    repo::insert_defect(pool, &defect_id, id, &description, &severity, reported_by)
        .await
        .map(AssetDefectResponse::from)
}

pub async fn update_asset_defect(
    pool: &PgPool,
    asset_id: &str,
    defect_id: &str,
    req: &UpdateAssetDefectRequest,
    resolved_by: &str,
) -> AppResult<AssetDefectResponse> {
    ensure_valid_uuid(asset_id, "asset")?;
    ensure_valid_uuid(defect_id, "asset defect")?;
    ensure_valid_uuid(resolved_by, "user")?;

    let existing = repo::find_defect(pool, asset_id, defect_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Asset defect not found".into()))?;

    let description = match req.description.as_deref() {
        Some(value) => normalize_required_text(value, "Description")?,
        None => existing.description.clone(),
    };
    let severity = match req.severity.as_deref() {
        Some(value) => normalize_defect_severity(value)?,
        None => existing.severity.clone(),
    };
    let status = normalize_defect_status(&req.status)?;
    let resolved_by = if status == "RESOLVED" {
        Some(resolved_by)
    } else {
        None
    };

    repo::update_defect(
        pool,
        defect_id,
        asset_id,
        &description,
        &severity,
        &status,
        resolved_by,
    )
    .await?
    .map(AssetDefectResponse::from)
    .ok_or_else(|| AppError::NotFound("Asset defect not found".into()))
}

async fn ensure_asset_exists(pool: &PgPool, id: &str) -> AppResult<()> {
    if repo::find_asset(pool, id).await?.is_none() {
        return Err(AppError::NotFound("Asset not found".into()));
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

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value.and_then(|value| {
        let normalized = value.trim();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
        }
    })
}

fn normalize_optional_uuid(value: Option<&str>, resource_name: &str) -> AppResult<Option<String>> {
    match value {
        Some(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                Ok(None)
            } else {
                ensure_valid_uuid(normalized, resource_name)?;
                Ok(Some(normalized.to_string()))
            }
        }
        None => Ok(None),
    }
}

fn normalize_optional_i32(value: Option<i32>, field_name: &str) -> AppResult<Option<i32>> {
    match value {
        Some(value) if value < 0 => Err(AppError::Validation(format!("{} cannot be negative", field_name))),
        _ => Ok(value),
    }
}

fn normalize_asset_status(value: Option<&str>) -> AppResult<String> {
    match value {
        Some(value) => normalize_required_text(value, "Status").map(|value| value.to_uppercase()),
        None => Ok("ACTIVE".to_string()),
    }
}

fn normalize_defect_severity(value: &str) -> AppResult<String> {
    let normalized = value.trim().to_uppercase();
    if matches!(normalized.as_str(), "LOW" | "MEDIUM" | "HIGH" | "CRITICAL") {
        Ok(normalized)
    } else {
        Err(AppError::Validation(
            "Severity must be one of LOW, MEDIUM, HIGH, or CRITICAL".into(),
        ))
    }
}

fn normalize_defect_status(value: &str) -> AppResult<String> {
    let normalized = value.trim().to_uppercase();
    if matches!(normalized.as_str(), "REPORTED" | "IN_PROGRESS" | "RESOLVED") {
        Ok(normalized)
    } else {
        Err(AppError::Validation(
            "Status must be one of REPORTED, IN_PROGRESS, or RESOLVED".into(),
        ))
    }
}

fn validate_inspection_data(value: &serde_json::Value) -> AppResult<()> {
    if value.is_null() {
        return Err(AppError::Validation("Inspection data is required".into()));
    }

    Ok(())
}

fn parse_optional_date(value: Option<&str>, field_name: &str) -> AppResult<Option<NaiveDate>> {
    match value {
        Some(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Ok(None);
            }

            let parsed = NaiveDate::parse_from_str(normalized, "%Y-%m-%d")
                .map_err(|_| AppError::Validation(format!("{} must be a valid YYYY-MM-DD date", field_name)))?;
            Ok(Some(parsed))
        }
        None => Ok(None),
    }
}

fn normalize_optional_decimal(
    value: Option<&str>,
    scale: usize,
    allow_zero: bool,
    field_name: &str,
) -> AppResult<Option<String>> {
    match value {
        Some(value) => {
            let normalized = value.trim();
            if normalized.is_empty() {
                return Ok(None);
            }

            let parsed = normalized
                .parse::<f64>()
                .map_err(|_| AppError::Validation(format!("{} must be a valid number", field_name)))?;

            if !parsed.is_finite() {
                return Err(AppError::Validation(format!("{} must be a valid number", field_name)));
            }
            if parsed < 0.0 {
                return Err(AppError::Validation(format!("{} cannot be negative", field_name)));
            }
            if !allow_zero && parsed <= 0.0 {
                return Err(AppError::Validation(format!("{} must be greater than zero", field_name)));
            }
            if let Some((_, fractional)) = normalized.split_once('.') {
                if fractional.len() > scale {
                    return Err(AppError::Validation(format!(
                        "{} cannot have more than {} decimal places",
                        field_name, scale
                    )));
                }
            }

            Ok(Some(normalized.to_string()))
        }
        None => Ok(None),
    }
}

fn ensure_valid_uuid(value: &str, resource_name: &str) -> AppResult<()> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| AppError::Validation(format!("Invalid {} identifier", resource_name)))
}
