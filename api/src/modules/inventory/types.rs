use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItemResponse {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub unit: String,
    pub cost_price: String,
    pub sell_price: String,
    pub min_stock_level: i32,
    pub current_quantity: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow)]
pub struct InventoryItemRow {
    pub id: String,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub unit: String,
    pub cost_price: String,
    pub sell_price: String,
    pub min_stock_level: i32,
    pub current_quantity: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<InventoryItemRow> for InventoryItemResponse {
    fn from(row: InventoryItemRow) -> Self {
        Self {
            id: row.id,
            sku: row.sku,
            name: row.name,
            description: row.description,
            category: row.category,
            unit: row.unit,
            cost_price: row.cost_price,
            sell_price: row.sell_price,
            min_stock_level: row.min_stock_level,
            current_quantity: row.current_quantity,
            is_active: row.is_active,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockAdjustmentResponse {
    pub id: String,
    pub inventory_item_id: String,
    pub adjustment_type: String,
    pub quantity: String,
    pub previous_quantity: Option<String>,
    pub new_quantity: Option<String>,
    pub reason: Option<String>,
    pub performed_by: Option<String>,
    pub created_at: String,
}

#[derive(Debug, FromRow)]
pub struct StockAdjustmentRow {
    pub id: String,
    pub inventory_item_id: String,
    pub adjustment_type: String,
    pub quantity: String,
    pub previous_quantity: Option<String>,
    pub new_quantity: Option<String>,
    pub reason: Option<String>,
    pub performed_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<StockAdjustmentRow> for StockAdjustmentResponse {
    fn from(row: StockAdjustmentRow) -> Self {
        Self {
            id: row.id,
            inventory_item_id: row.inventory_item_id,
            adjustment_type: row.adjustment_type,
            quantity: row.quantity,
            previous_quantity: row.previous_quantity,
            new_quantity: row.new_quantity,
            reason: row.reason,
            performed_by: row.performed_by,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItemDetailResponse {
    pub item: InventoryItemResponse,
    pub adjustments: Vec<StockAdjustmentResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryAdjustmentResult {
    pub item: InventoryItemResponse,
    pub adjustment: StockAdjustmentResponse,
}

#[derive(Debug, Deserialize, Validate)]
pub struct InventoryItemRequest {
    #[validate(length(min = 1, message = "SKU is required"))]
    pub sku: String,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    #[validate(length(min = 1, message = "Unit is required"))]
    pub unit: String,
    #[validate(length(min = 1, message = "Cost price is required"))]
    pub cost_price: String,
    #[validate(length(min = 1, message = "Sell price is required"))]
    pub sell_price: String,
    #[validate(range(min = 0, message = "Minimum stock level cannot be negative"))]
    pub min_stock_level: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct StockAdjustmentRequest {
    #[validate(length(min = 1, message = "Adjustment type is required"))]
    pub adjustment_type: String,
    #[validate(length(min = 1, message = "Quantity is required"))]
    pub quantity: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ListQuery {
    pub page: i64,
    pub limit: i64,
    pub search: Option<String>,
    pub category: Option<String>,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
            search: None,
            category: None,
        }
    }
}
