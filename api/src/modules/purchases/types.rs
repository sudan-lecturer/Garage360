use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;
use crate::common::pagination::PaginationMeta;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ListQuery {
    pub page: i64,
    pub limit: i64,
    pub search: Option<String>,
    pub status: Option<String>,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
            search: None,
            status: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderItemRequest {
    pub inventory_item_id: Option<String>,
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(range(min = 0.001))]
    pub quantity: f64,
    #[validate(range(min = 0.0))]
    pub unit_price: f64,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreatePurchaseOrderRequest {
    #[validate(length(min = 1))]
    pub supplier_id: String,
    pub expected_delivery: Option<String>,
    #[validate(range(min = 0.0, max = 100.0))]
    pub discount_pct: Option<f64>,
    #[validate(range(min = 0.0))]
    pub tax_amount: Option<f64>,
    pub notes: Option<String>,
    pub currency: Option<String>,
    #[validate(range(min = 0.0))]
    pub exchange_rate: Option<f64>,
    #[validate(length(min = 1), nested)]
    #[serde(default)]
    pub items: Vec<PurchaseOrderItemRequest>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseActionRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransitPurchaseRequest {
    pub notes: Option<String>,
    pub expected_delivery: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GrnItemRequest {
    #[validate(length(min = 1))]
    pub po_item_id: String,
    #[validate(range(min = 0.001))]
    pub received_qty: f64,
    #[validate(range(min = 0.0))]
    pub unit_cost: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateGrnRequest {
    pub received_at: Option<String>,
    pub notes: Option<String>,
    #[validate(length(min = 1), nested)]
    #[serde(default)]
    pub items: Vec<GrnItemRequest>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct QaInspectionItemRequest {
    #[validate(length(min = 1))]
    pub grn_item_id: String,
    #[validate(range(min = 0.0))]
    pub accepted_qty: Option<f64>,
    #[validate(range(min = 0.0))]
    pub rejected_qty: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateQaInspectionRequest {
    #[validate(length(min = 1))]
    pub status: String,
    pub notes: Option<String>,
    #[validate(nested)]
    #[serde(default)]
    pub items: Vec<QaInspectionItemRequest>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderSummaryResponse {
    pub id: String,
    pub po_no: Option<i32>,
    pub supplier_id: String,
    pub supplier_name: String,
    pub status: String,
    pub expected_delivery: Option<String>,
    pub subtotal: String,
    pub discount_pct: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub currency: String,
    pub exchange_rate: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderItemResponse {
    pub id: String,
    pub purchase_order_id: String,
    pub inventory_item_id: Option<String>,
    pub inventory_item_name: Option<String>,
    pub description: String,
    pub quantity: String,
    pub unit_price: String,
    pub received_qty: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodsReceiptSummaryResponse {
    pub id: String,
    pub grn_no: Option<i32>,
    pub purchase_order_id: String,
    pub status: String,
    pub received_by: Option<String>,
    pub received_by_name: Option<String>,
    pub received_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderResponse {
    pub id: String,
    pub po_no: Option<i32>,
    pub supplier_id: String,
    pub supplier_name: String,
    pub status: String,
    pub expected_delivery: Option<String>,
    pub subtotal: String,
    pub discount_pct: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub notes: Option<String>,
    pub currency: String,
    pub exchange_rate: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub items: Vec<PurchaseOrderItemResponse>,
    pub grns: Vec<GoodsReceiptSummaryResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseStatusHistoryResponse {
    pub id: String,
    pub from_status: Option<String>,
    pub to_status: String,
    pub notes: Option<String>,
    pub changed_by: Option<String>,
    pub changed_by_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseApprovalResponse {
    pub id: String,
    pub approved_by: Option<String>,
    pub approved_by_name: Option<String>,
    pub is_approved: bool,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseHistoryResponse {
    pub status_history: Vec<PurchaseStatusHistoryResponse>,
    pub approvals: Vec<PurchaseApprovalResponse>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodsReceiptItemResponse {
    pub id: String,
    pub grn_id: String,
    pub po_item_id: Option<String>,
    pub inventory_item_id: Option<String>,
    pub inventory_item_name: Option<String>,
    pub description: Option<String>,
    pub ordered_qty: Option<String>,
    pub received_qty: String,
    pub accepted_qty: Option<String>,
    pub rejected_qty: Option<String>,
    pub unit_cost: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QaInspectionResponse {
    pub id: String,
    pub grn_id: String,
    pub inspected_by: Option<String>,
    pub inspected_by_name: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodsReceiptNoteResponse {
    pub id: String,
    pub grn_no: Option<i32>,
    pub purchase_order_id: String,
    pub status: String,
    pub received_by: Option<String>,
    pub received_by_name: Option<String>,
    pub received_at: Option<String>,
    pub created_at: String,
    pub items: Vec<GoodsReceiptItemResponse>,
    pub inspections: Vec<QaInspectionResponse>,
}

#[derive(Debug, FromRow)]
pub struct PurchaseOrderSummaryRow {
    pub id: String,
    pub po_no: Option<i32>,
    pub supplier_id: String,
    pub supplier_name: String,
    pub status: String,
    pub expected_delivery: Option<DateTime<Utc>>,
    pub subtotal: String,
    pub discount_pct: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub currency: String,
    pub exchange_rate: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct PurchaseOrderDetailRow {
    pub id: String,
    pub po_no: Option<i32>,
    pub supplier_id: String,
    pub supplier_name: String,
    pub status: String,
    pub expected_delivery: Option<DateTime<Utc>>,
    pub subtotal: String,
    pub discount_pct: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub notes: Option<String>,
    pub currency: String,
    pub exchange_rate: Option<String>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct PurchaseOrderItemRow {
    pub id: String,
    pub purchase_order_id: String,
    pub inventory_item_id: Option<String>,
    pub inventory_item_name: Option<String>,
    pub description: String,
    pub quantity: String,
    pub unit_price: String,
    pub received_qty: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct PurchaseOrderStateRow {
    pub id: String,
    pub status: String,
}

#[derive(Debug, FromRow)]
pub struct PurchaseOrderItemCalcRow {
    pub id: String,
    pub inventory_item_id: Option<String>,
    pub description: String,
    pub quantity: f64,
    pub received_qty: f64,
}

#[derive(Debug, FromRow)]
pub struct PurchaseStatusHistoryRow {
    pub id: String,
    pub from_status: Option<String>,
    pub to_status: String,
    pub notes: Option<String>,
    pub changed_by: Option<String>,
    pub changed_by_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct PurchaseApprovalRow {
    pub id: String,
    pub approved_by: Option<String>,
    pub approved_by_name: Option<String>,
    pub is_approved: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct GoodsReceiptSummaryRow {
    pub id: String,
    pub grn_no: Option<i32>,
    pub purchase_order_id: String,
    pub status: String,
    pub received_by: Option<String>,
    pub received_by_name: Option<String>,
    pub received_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct GoodsReceiptStateRow {
    pub id: String,
    pub purchase_order_id: String,
    pub status: String,
}

#[derive(Debug, FromRow)]
pub struct GoodsReceiptItemRow {
    pub id: String,
    pub grn_id: String,
    pub po_item_id: Option<String>,
    pub inventory_item_id: Option<String>,
    pub inventory_item_name: Option<String>,
    pub description: Option<String>,
    pub ordered_qty: Option<String>,
    pub received_qty: String,
    pub accepted_qty: Option<String>,
    pub rejected_qty: Option<String>,
    pub unit_cost: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct GoodsReceiptItemCalcRow {
    pub id: String,
    pub po_item_id: Option<String>,
    pub inventory_item_id: Option<String>,
    pub received_qty: f64,
    pub unit_cost: Option<f64>,
}

#[derive(Debug, FromRow)]
pub struct QaInspectionRow {
    pub id: String,
    pub grn_id: String,
    pub inspected_by: Option<String>,
    pub inspected_by_name: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<PurchaseOrderSummaryRow> for PurchaseOrderSummaryResponse {
    fn from(row: PurchaseOrderSummaryRow) -> Self {
        Self {
            id: row.id,
            po_no: row.po_no,
            supplier_id: row.supplier_id,
            supplier_name: row.supplier_name,
            status: row.status,
            expected_delivery: row.expected_delivery.map(|value| value.to_rfc3339()),
            subtotal: row.subtotal,
            discount_pct: row.discount_pct,
            tax_amount: row.tax_amount,
            total_amount: row.total_amount,
            currency: row.currency,
            exchange_rate: row.exchange_rate,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl From<PurchaseOrderItemRow> for PurchaseOrderItemResponse {
    fn from(row: PurchaseOrderItemRow) -> Self {
        Self {
            id: row.id,
            purchase_order_id: row.purchase_order_id,
            inventory_item_id: row.inventory_item_id,
            inventory_item_name: row.inventory_item_name,
            description: row.description,
            quantity: row.quantity,
            unit_price: row.unit_price,
            received_qty: row.received_qty,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl From<GoodsReceiptSummaryRow> for GoodsReceiptSummaryResponse {
    fn from(row: GoodsReceiptSummaryRow) -> Self {
        Self {
            id: row.id,
            grn_no: row.grn_no,
            purchase_order_id: row.purchase_order_id,
            status: row.status,
            received_by: row.received_by,
            received_by_name: row.received_by_name,
            received_at: row.received_at.map(|value| value.to_rfc3339()),
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl PurchaseOrderResponse {
    pub fn from_parts(
        row: PurchaseOrderDetailRow,
        items: Vec<PurchaseOrderItemRow>,
        grns: Vec<GoodsReceiptSummaryRow>,
    ) -> Self {
        Self {
            id: row.id,
            po_no: row.po_no,
            supplier_id: row.supplier_id,
            supplier_name: row.supplier_name,
            status: row.status,
            expected_delivery: row.expected_delivery.map(|value| value.to_rfc3339()),
            subtotal: row.subtotal,
            discount_pct: row.discount_pct,
            tax_amount: row.tax_amount,
            total_amount: row.total_amount,
            notes: row.notes,
            currency: row.currency,
            exchange_rate: row.exchange_rate,
            created_by: row.created_by,
            created_at: row.created_at.to_rfc3339(),
            items: items.into_iter().map(PurchaseOrderItemResponse::from).collect(),
            grns: grns.into_iter().map(GoodsReceiptSummaryResponse::from).collect(),
        }
    }
}

impl From<PurchaseStatusHistoryRow> for PurchaseStatusHistoryResponse {
    fn from(row: PurchaseStatusHistoryRow) -> Self {
        Self {
            id: row.id,
            from_status: row.from_status,
            to_status: row.to_status,
            notes: row.notes,
            changed_by: row.changed_by,
            changed_by_name: row.changed_by_name,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl From<PurchaseApprovalRow> for PurchaseApprovalResponse {
    fn from(row: PurchaseApprovalRow) -> Self {
        Self {
            id: row.id,
            approved_by: row.approved_by,
            approved_by_name: row.approved_by_name,
            is_approved: row.is_approved,
            notes: row.notes,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl From<GoodsReceiptItemRow> for GoodsReceiptItemResponse {
    fn from(row: GoodsReceiptItemRow) -> Self {
        Self {
            id: row.id,
            grn_id: row.grn_id,
            po_item_id: row.po_item_id,
            inventory_item_id: row.inventory_item_id,
            inventory_item_name: row.inventory_item_name,
            description: row.description,
            ordered_qty: row.ordered_qty,
            received_qty: row.received_qty,
            accepted_qty: row.accepted_qty,
            rejected_qty: row.rejected_qty,
            unit_cost: row.unit_cost,
            notes: row.notes,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl From<QaInspectionRow> for QaInspectionResponse {
    fn from(row: QaInspectionRow) -> Self {
        Self {
            id: row.id,
            grn_id: row.grn_id,
            inspected_by: row.inspected_by,
            inspected_by_name: row.inspected_by_name,
            status: row.status,
            notes: row.notes,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl GoodsReceiptNoteResponse {
    pub fn from_parts(
        row: GoodsReceiptSummaryRow,
        items: Vec<GoodsReceiptItemRow>,
        inspections: Vec<QaInspectionRow>,
    ) -> Self {
        Self {
            id: row.id,
            grn_no: row.grn_no,
            purchase_order_id: row.purchase_order_id,
            status: row.status,
            received_by: row.received_by,
            received_by_name: row.received_by_name,
            received_at: row.received_at.map(|value| value.to_rfc3339()),
            created_at: row.created_at.to_rfc3339(),
            items: items.into_iter().map(GoodsReceiptItemResponse::from).collect(),
            inspections: inspections.into_iter().map(QaInspectionResponse::from).collect(),
        }
    }
}
