use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::common::pagination::PaginationMeta;

use super::{
    repo,
    types::{
        InventoryAdjustmentResult, InventoryItemDetailResponse, InventoryItemRequest,
        InventoryItemResponse, StockAdjustmentRequest, StockAdjustmentResponse,
    },
};

pub async fn list_inventory(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
    category: String,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);
    let category_like = format!("%{}%", category);

    let items = repo::list(pool, &search, &like, &category, &category_like, limit, offset).await?;
    let total = repo::count(pool, &search, &like, &category, &category_like).await?;

    Ok(json!({
        "data": items.into_iter().map(InventoryItemResponse::from).collect::<Vec<_>>(),
        "meta": crate::common::pagination::PaginationMeta { page, limit, total }
    }))
}

pub async fn search_inventory(pool: &PgPool, search: String) -> AppResult<Vec<InventoryItemResponse>> {
    let like = format!("%{}%", search);
    Ok(repo::list(pool, &search, &like, "", "%%", 20, 0)
        .await?
        .into_iter()
        .map(InventoryItemResponse::from)
        .collect::<Vec<_>>())
}

pub async fn list_low_stock(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
    category: String,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);
    let category_like = format!("%{}%", category);

    let items = repo::list_low_stock(pool, &search, &like, &category, &category_like, limit, offset).await?;
    let total = repo::count_low_stock(pool, &search, &like, &category, &category_like).await?;

    Ok(json!({
        "data": items.into_iter().map(InventoryItemResponse::from).collect::<Vec<_>>(),
        "meta": crate::common::pagination::PaginationMeta { page, limit, total }
    }))
}

pub async fn get_inventory_item(pool: &PgPool, id: &str) -> AppResult<InventoryItemDetailResponse> {
    ensure_valid_uuid(id, "inventory item")?;

    let item = repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Inventory item not found".into()))?;
    let adjustments = repo::list_adjustments(pool, id).await?;

    Ok(InventoryItemDetailResponse {
        item: InventoryItemResponse::from(item),
        adjustments: adjustments
            .into_iter()
            .map(StockAdjustmentResponse::from)
            .collect::<Vec<_>>(),
    })
}

pub async fn create_inventory_item(
    pool: &PgPool,
    req: &InventoryItemRequest,
    created_by: &str,
) -> AppResult<InventoryItemResponse> {
    ensure_valid_uuid(created_by, "user")?;

    let sku = normalize_required_text(&req.sku, "SKU")?;
    let name = normalize_required_text(&req.name, "Name")?;
    let unit = normalize_required_text(&req.unit, "Unit")?;
    let description = normalize_optional_text(req.description.as_deref());
    let category = normalize_optional_text(req.category.as_deref());
    let cost_price = normalize_decimal_input(&req.cost_price, 2, true, "Cost price")?;
    let sell_price = normalize_decimal_input(&req.sell_price, 2, true, "Sell price")?;

    let item_id = Uuid::now_v7().to_string();
    repo::create(
        pool,
        &item_id,
        &sku,
        &name,
        description.as_deref(),
        category.as_deref(),
        &unit,
        &cost_price,
        &sell_price,
        req.min_stock_level,
        created_by,
    )
    .await
    .map(InventoryItemResponse::from)
}

pub async fn update_inventory_item(
    pool: &PgPool,
    id: &str,
    req: &InventoryItemRequest,
) -> AppResult<InventoryItemResponse> {
    ensure_valid_uuid(id, "inventory item")?;

    let sku = normalize_required_text(&req.sku, "SKU")?;
    let name = normalize_required_text(&req.name, "Name")?;
    let unit = normalize_required_text(&req.unit, "Unit")?;
    let description = normalize_optional_text(req.description.as_deref());
    let category = normalize_optional_text(req.category.as_deref());
    let cost_price = normalize_decimal_input(&req.cost_price, 2, true, "Cost price")?;
    let sell_price = normalize_decimal_input(&req.sell_price, 2, true, "Sell price")?;

    repo::update(
        pool,
        id,
        &sku,
        &name,
        description.as_deref(),
        category.as_deref(),
        &unit,
        &cost_price,
        &sell_price,
        req.min_stock_level,
    )
    .await?
    .map(InventoryItemResponse::from)
    .ok_or_else(|| AppError::NotFound("Inventory item not found".into()))
}

pub async fn delete_inventory_item(pool: &PgPool, id: &str) -> AppResult<serde_json::Value> {
    ensure_valid_uuid(id, "inventory item")?;

    let rows_affected = repo::soft_delete(pool, id).await?;
    if rows_affected == 0 {
        return Err(AppError::NotFound("Inventory item not found".into()));
    }

    Ok(json!({ "deleted": true }))
}

pub async fn adjust_inventory_stock(
    pool: &PgPool,
    id: &str,
    req: &StockAdjustmentRequest,
    performed_by: &str,
) -> AppResult<InventoryAdjustmentResult> {
    ensure_valid_uuid(id, "inventory item")?;
    ensure_valid_uuid(performed_by, "user")?;

    let adjustment_type = normalize_adjustment_type(&req.adjustment_type)?;
    let allow_zero = matches!(adjustment_type.as_str(), "SET" | "COUNT");
    let quantity = normalize_decimal_input(&req.quantity, 3, allow_zero, "Quantity")?;
    let reason = normalize_optional_text(req.reason.as_deref());

    let adjustment_id = Uuid::now_v7().to_string();
    let (item, adjustment) = repo::adjust_stock(
        pool,
        &adjustment_id,
        id,
        &adjustment_type,
        &quantity,
        reason.as_deref(),
        performed_by,
    )
    .await?;

    Ok(InventoryAdjustmentResult {
        item: InventoryItemResponse::from(item),
        adjustment: StockAdjustmentResponse::from(adjustment),
    })
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

fn normalize_adjustment_type(value: &str) -> AppResult<String> {
    let normalized = value.trim().to_uppercase();
    if matches!(normalized.as_str(), "ADD" | "REMOVE" | "SET" | "COUNT") {
        Ok(normalized)
    } else {
        Err(AppError::Validation(
            "Adjustment type must be one of ADD, REMOVE, SET, or COUNT".into(),
        ))
    }
}

fn normalize_decimal_input(value: &str, scale: usize, allow_zero: bool, field_name: &str) -> AppResult<String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(AppError::Validation(format!("{} is required", field_name)));
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

    Ok(normalized.to_string())
}

fn ensure_valid_uuid(value: &str, resource_name: &str) -> AppResult<()> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| AppError::Validation(format!("Invalid {} identifier", resource_name)))
}
