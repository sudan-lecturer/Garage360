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

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLineItemRequest {
    #[validate(length(min = 1))]
    pub description: String,
    #[validate(range(min = 0.001))]
    pub quantity: f64,
    #[validate(range(min = 0.0))]
    pub unit_price: f64,
    #[validate(range(min = 0.0, max = 100.0))]
    pub discount_pct: Option<f64>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceRequest {
    pub job_card_id: Option<String>,
    #[validate(length(min = 1))]
    pub customer_id: String,
    #[validate(range(min = 0.0, max = 100.0))]
    pub discount_pct: Option<f64>,
    #[validate(range(min = 0.0))]
    pub tax_amount: Option<f64>,
    pub notes: Option<String>,
    #[validate(nested)]
    #[serde(default)]
    pub line_items: Vec<InvoiceLineItemRequest>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RecordPaymentRequest {
    #[validate(range(min = 0.01))]
    pub amount: f64,
    pub payment_method: Option<String>,
    pub payment_ref: Option<String>,
    pub paid_at: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VoidInvoiceRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceSummaryResponse {
    pub id: String,
    pub invoice_no: Option<i32>,
    pub job_card_id: Option<String>,
    pub job_no: Option<i32>,
    pub customer_id: String,
    pub customer_name: String,
    pub status: String,
    pub subtotal: String,
    pub discount_pct: String,
    pub discount_amount: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub amount_paid: String,
    pub balance_due: String,
    pub payment_method: Option<String>,
    pub payment_ref: Option<String>,
    pub paid_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLineItemResponse {
    pub id: String,
    pub invoice_id: String,
    pub description: String,
    pub quantity: String,
    pub unit_price: String,
    pub discount_pct: String,
    pub line_total: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceResponse {
    pub id: String,
    pub invoice_no: Option<i32>,
    pub job_card_id: Option<String>,
    pub job_no: Option<i32>,
    pub customer_id: String,
    pub customer_name: String,
    pub status: String,
    pub subtotal: String,
    pub discount_pct: String,
    pub discount_amount: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub amount_paid: String,
    pub balance_due: String,
    pub payment_method: Option<String>,
    pub payment_ref: Option<String>,
    pub paid_at: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub line_items: Vec<InvoiceLineItemResponse>,
}

#[derive(Debug, FromRow)]
pub struct InvoiceSummaryRow {
    pub id: String,
    pub invoice_no: Option<i32>,
    pub job_card_id: Option<String>,
    pub job_no: Option<i32>,
    pub customer_id: String,
    pub customer_name: String,
    pub status: String,
    pub subtotal: String,
    pub discount_pct: String,
    pub discount_amount: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub amount_paid: String,
    pub balance_due: String,
    pub payment_method: Option<String>,
    pub payment_ref: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct InvoiceDetailRow {
    pub id: String,
    pub invoice_no: Option<i32>,
    pub job_card_id: Option<String>,
    pub job_no: Option<i32>,
    pub customer_id: String,
    pub customer_name: String,
    pub status: String,
    pub subtotal: String,
    pub discount_pct: String,
    pub discount_amount: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub amount_paid: String,
    pub balance_due: String,
    pub payment_method: Option<String>,
    pub payment_ref: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct InvoiceLineItemRow {
    pub id: String,
    pub invoice_id: String,
    pub description: String,
    pub quantity: String,
    pub unit_price: String,
    pub discount_pct: String,
    pub line_total: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct InvoiceStateRow {
    pub id: String,
    pub status: String,
    pub job_card_id: Option<String>,
    pub customer_id: String,
    pub subtotal: f64,
    pub discount_pct: f64,
    pub discount_amount: f64,
    pub tax_amount: f64,
    pub total_amount: f64,
    pub amount_paid: f64,
    pub payment_method: Option<String>,
    pub payment_ref: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct InvoiceLineCalcRow {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub discount_pct: f64,
}

#[derive(Debug, FromRow)]
pub struct JobCardInvoiceSeedRow {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub discount_pct: f64,
}

impl From<InvoiceSummaryRow> for InvoiceSummaryResponse {
    fn from(row: InvoiceSummaryRow) -> Self {
        Self {
            id: row.id,
            invoice_no: row.invoice_no,
            job_card_id: row.job_card_id,
            job_no: row.job_no,
            customer_id: row.customer_id,
            customer_name: row.customer_name,
            status: row.status,
            subtotal: row.subtotal,
            discount_pct: row.discount_pct,
            discount_amount: row.discount_amount,
            tax_amount: row.tax_amount,
            total_amount: row.total_amount,
            amount_paid: row.amount_paid,
            balance_due: row.balance_due,
            payment_method: row.payment_method,
            payment_ref: row.payment_ref,
            paid_at: row.paid_at.map(|value| value.to_rfc3339()),
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl From<InvoiceLineItemRow> for InvoiceLineItemResponse {
    fn from(row: InvoiceLineItemRow) -> Self {
        Self {
            id: row.id,
            invoice_id: row.invoice_id,
            description: row.description,
            quantity: row.quantity,
            unit_price: row.unit_price,
            discount_pct: row.discount_pct,
            line_total: row.line_total,
            created_at: row.created_at.to_rfc3339(),
        }
    }
}

impl InvoiceResponse {
    pub fn from_parts(row: InvoiceDetailRow, line_items: Vec<InvoiceLineItemRow>) -> Self {
        Self {
            id: row.id,
            invoice_no: row.invoice_no,
            job_card_id: row.job_card_id,
            job_no: row.job_no,
            customer_id: row.customer_id,
            customer_name: row.customer_name,
            status: row.status,
            subtotal: row.subtotal,
            discount_pct: row.discount_pct,
            discount_amount: row.discount_amount,
            tax_amount: row.tax_amount,
            total_amount: row.total_amount,
            amount_paid: row.amount_paid,
            balance_due: row.balance_due,
            payment_method: row.payment_method,
            payment_ref: row.payment_ref,
            paid_at: row.paid_at.map(|value| value.to_rfc3339()),
            notes: row.notes,
            created_at: row.created_at.to_rfc3339(),
            line_items: line_items.into_iter().map(InvoiceLineItemResponse::from).collect(),
        }
    }
}
