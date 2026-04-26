use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::common::pagination::PaginationMeta;

use super::{
    repo,
    types::{
        CreateInvoiceRequest, InvoiceLineItemRequest, InvoiceResponse, InvoiceSummaryResponse,
        RecordPaymentRequest, VoidInvoiceRequest,
    },
};

const STATUS_PENDING: &str = "PENDING";
const STATUS_PARTIAL: &str = "PARTIAL";
const STATUS_PAID: &str = "PAID";
const STATUS_VOID: &str = "VOID";

const EPSILON: f64 = 0.000_001;

struct ResolvedInvoiceLine {
    description: String,
    quantity: f64,
    unit_price: f64,
    discount_pct: f64,
}

struct InvoiceTotals {
    subtotal: f64,
    discount_pct: f64,
    discount_amount: f64,
    tax_amount: f64,
    total_amount: f64,
}

pub async fn list_invoices(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
    status: String,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);

    let invoices = repo::list(pool, &search, &like, &status, limit, offset).await?;
    let total = repo::count(pool, &search, &like, &status).await?;

    Ok(json!({
        "data": invoices
            .into_iter()
            .map(InvoiceSummaryResponse::from)
            .collect::<Vec<_>>(),
        "meta": PaginationMeta { page, limit, total }
    }))
}

pub async fn export_invoices(pool: &PgPool, search: String, status: String) -> AppResult<serde_json::Value> {
    let like = format!("%{}%", search);
    let invoices = repo::list(pool, &search, &like, &status, 10_000, 0).await?;
    let rows = invoices
        .into_iter()
        .map(InvoiceSummaryResponse::from)
        .map(|invoice| {
            vec![
                json!(invoice.invoice_no),
                json!(invoice.job_no),
                json!(invoice.customer_name),
                json!(invoice.status),
                json!(invoice.subtotal),
                json!(invoice.discount_amount),
                json!(invoice.tax_amount),
                json!(invoice.total_amount),
                json!(invoice.amount_paid),
                json!(invoice.balance_due),
                json!(invoice.created_at),
            ]
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "headers": ["Invoice No", "Job No", "Customer", "Status", "Subtotal", "Discount", "Tax", "Total", "Amount Paid", "Balance Due", "Created At"],
        "data": rows
    }))
}

pub async fn get_invoice(pool: &PgPool, id: &str) -> AppResult<InvoiceResponse> {
    ensure_uuid(id, "invoice id")?;

    let invoice = repo::find_invoice(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    let line_items = repo::find_invoice_lines(pool, id).await?;

    Ok(InvoiceResponse::from_parts(invoice, line_items))
}

pub async fn create_invoice(
    pool: &PgPool,
    req: &CreateInvoiceRequest,
) -> AppResult<InvoiceResponse> {
    validate_invoice_references(pool, req).await?;

    let resolved_lines = resolve_invoice_lines(pool, req).await?;
    let totals = calculate_invoice_totals(req.discount_pct, req.tax_amount, &resolved_lines)?;
    let invoice_id = Uuid::now_v7().to_string();
    let status = derive_invoice_status(0.0, totals.total_amount);

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::insert_invoice(
        &mut tx,
        &invoice_id,
        req.job_card_id.as_deref(),
        &req.customer_id,
        totals.subtotal,
        totals.discount_pct,
        totals.discount_amount,
        totals.tax_amount,
        totals.total_amount,
        0.0,
        status,
        req.notes.as_deref(),
    )
    .await?;
    insert_invoice_lines(&mut tx, &invoice_id, &resolved_lines).await?;
    tx.commit().await.map_err(AppError::Database)?;

    get_invoice(pool, &invoice_id).await
}

pub async fn update_invoice(
    pool: &PgPool,
    id: &str,
    req: &CreateInvoiceRequest,
) -> AppResult<InvoiceResponse> {
    ensure_uuid(id, "invoice id")?;

    let state = repo::find_invoice_state(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    if state.status == STATUS_VOID || state.status == STATUS_PAID {
        return Err(AppError::Conflict(
            "Paid or void invoices cannot be updated".into(),
        ));
    }

    validate_invoice_references(pool, req).await?;

    let resolved_lines = resolve_invoice_lines(pool, req).await?;
    let totals = calculate_invoice_totals(req.discount_pct, req.tax_amount, &resolved_lines)?;
    if totals.total_amount + EPSILON < state.amount_paid {
        return Err(AppError::Conflict(
            "Updated invoice total cannot be less than the amount already paid".into(),
        ));
    }

    let next_status = derive_invoice_status(state.amount_paid, totals.total_amount);

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::update_invoice_header(
        &mut tx,
        id,
        req.job_card_id.as_deref(),
        &req.customer_id,
        totals.subtotal,
        totals.discount_pct,
        totals.discount_amount,
        totals.tax_amount,
        totals.total_amount,
        next_status,
        req.notes.as_deref(),
    )
    .await?;
    repo::delete_invoice_lines(&mut tx, id).await?;
    insert_invoice_lines(&mut tx, id, &resolved_lines).await?;
    tx.commit().await.map_err(AppError::Database)?;

    get_invoice(pool, id).await
}

pub async fn issue_invoice(pool: &PgPool, id: &str) -> AppResult<InvoiceResponse> {
    ensure_uuid(id, "invoice id")?;

    let state = repo::find_invoice_state(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    if state.status == STATUS_VOID {
        return Err(AppError::Conflict("Void invoices cannot be issued".into()));
    }

    let line_items = repo::find_invoice_line_calcs(pool, id).await?;
    if line_items.is_empty() {
        return Err(AppError::Validation(
            "Invoice must contain at least one line item".into(),
        ));
    }

    let resolved_lines = line_items
        .into_iter()
        .map(|line| ResolvedInvoiceLine {
            description: line.description,
            quantity: line.quantity,
            unit_price: line.unit_price,
            discount_pct: line.discount_pct,
        })
        .collect::<Vec<_>>();
    let totals = calculate_invoice_totals(Some(state.discount_pct), Some(state.tax_amount), &resolved_lines)?;
    if totals.total_amount + EPSILON < state.amount_paid {
        return Err(AppError::Conflict(
            "Invoice total cannot be less than the amount already paid".into(),
        ));
    }

    repo::refresh_invoice_totals(
        pool,
        id,
        totals.subtotal,
        totals.discount_amount,
        totals.tax_amount,
        totals.total_amount,
        derive_invoice_status(state.amount_paid, totals.total_amount),
    )
    .await?;

    get_invoice(pool, id).await
}

pub async fn record_payment(
    pool: &PgPool,
    id: &str,
    req: &RecordPaymentRequest,
) -> AppResult<InvoiceResponse> {
    ensure_uuid(id, "invoice id")?;

    let state = repo::find_invoice_state(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    if state.status == STATUS_VOID {
        return Err(AppError::Conflict("Void invoices cannot accept payments".into()));
    }
    if state.total_amount <= EPSILON {
        return Err(AppError::Conflict(
            "Invoice total must be greater than zero before recording payment".into(),
        ));
    }

    let amount = round_money(req.amount);
    if amount <= 0.0 {
        return Err(AppError::Validation(
            "Payment amount must be greater than zero".into(),
        ));
    }

    let new_amount_paid = round_money(state.amount_paid + amount);
    if new_amount_paid - state.total_amount > EPSILON {
        return Err(AppError::Conflict(
            "Payment exceeds the outstanding invoice balance".into(),
        ));
    }

    let payment_method = normalize_payment_method(req.payment_method.as_deref())?;
    let payment_ref = normalize_payment_ref(req.payment_ref.as_deref());
    let paid_at = parse_optional_datetime(&req.paid_at, "paidAt")?.unwrap_or_else(Utc::now);
    let next_status = derive_invoice_status(new_amount_paid, state.total_amount);
    let next_notes = merge_action_note(state.notes.as_deref(), req.notes.as_deref(), "PAYMENT");

    repo::update_payment(
        pool,
        id,
        new_amount_paid,
        next_status,
        payment_method.as_deref().or(state.payment_method.as_deref()),
        payment_ref.as_deref().or(state.payment_ref.as_deref()),
        Some(paid_at),
        next_notes.as_deref(),
    )
    .await?;

    get_invoice(pool, id).await
}

pub async fn void_invoice(
    pool: &PgPool,
    id: &str,
    req: &VoidInvoiceRequest,
) -> AppResult<InvoiceResponse> {
    ensure_uuid(id, "invoice id")?;

    let state = repo::find_invoice_state(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invoice not found".into()))?;
    if state.status == STATUS_VOID {
        return Err(AppError::Conflict("Invoice is already void".into()));
    }
    if state.amount_paid > EPSILON {
        return Err(AppError::Conflict(
            "Invoices with recorded payments cannot be voided".into(),
        ));
    }

    let next_notes = merge_action_note(state.notes.as_deref(), req.notes.as_deref(), "VOID");
    repo::update_status_and_notes(pool, id, STATUS_VOID, next_notes.as_deref()).await?;

    get_invoice(pool, id).await
}

async fn validate_invoice_references(pool: &PgPool, req: &CreateInvoiceRequest) -> AppResult<()> {
    ensure_uuid(&req.customer_id, "customerId")?;
    if !repo::customer_exists(pool, &req.customer_id).await? {
        return Err(AppError::NotFound("Customer not found".into()));
    }

    if let Some(job_card_id) = &req.job_card_id {
        ensure_uuid(job_card_id, "jobCardId")?;
        let job_customer_id = repo::find_job_card_customer_id(pool, job_card_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Job card not found".into()))?;
        if job_customer_id != req.customer_id {
            return Err(AppError::Conflict(
                "Invoice customer must match the linked job card customer".into(),
            ));
        }
    }

    Ok(())
}

async fn resolve_invoice_lines(
    pool: &PgPool,
    req: &CreateInvoiceRequest,
) -> AppResult<Vec<ResolvedInvoiceLine>> {
    if !req.line_items.is_empty() {
        let mut lines = Vec::with_capacity(req.line_items.len());
        for item in &req.line_items {
            lines.push(resolve_invoice_line(item)?);
        }
        return Ok(lines);
    }

    let job_card_id = req.job_card_id.as_ref().ok_or_else(|| {
        AppError::Validation(
            "Invoice requires either lineItems or a linked jobCardId with billable items".into(),
        )
    })?;

    let job_lines = repo::fetch_job_card_invoice_seed(pool, job_card_id).await?;
    if job_lines.is_empty() {
        return Err(AppError::Validation(
            "Linked job card does not contain any active billable items".into(),
        ));
    }

    Ok(job_lines
        .into_iter()
        .map(|item| resolve_line_values(item.description, item.quantity, item.unit_price, item.discount_pct))
        .collect::<AppResult<Vec<_>>>()?)
}

async fn insert_invoice_lines(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    invoice_id: &str,
    lines: &[ResolvedInvoiceLine],
) -> AppResult<()> {
    for line in lines {
        repo::insert_invoice_line(
            tx,
            &Uuid::now_v7().to_string(),
            invoice_id,
            &line.description,
            round_qty(line.quantity),
            round_money(line.unit_price),
            round_money(line.discount_pct),
            calculate_line_total(line.quantity, line.unit_price, line.discount_pct),
        )
        .await?;
    }

    Ok(())
}

fn resolve_invoice_line(item: &InvoiceLineItemRequest) -> AppResult<ResolvedInvoiceLine> {
    if item.description.trim().is_empty() {
        return Err(AppError::Validation(
            "Invoice line item description is required".into(),
        ));
    }
    if item.quantity <= 0.0 {
        return Err(AppError::Validation(
            "Invoice line item quantity must be greater than zero".into(),
        ));
    }
    if item.unit_price < 0.0 {
        return Err(AppError::Validation(
            "Invoice line item unit price cannot be negative".into(),
        ));
    }

    let discount_pct = round_money(item.discount_pct.unwrap_or(0.0));
    resolve_line_values(
        item.description.trim().to_string(),
        item.quantity,
        item.unit_price,
        discount_pct,
    )
}

fn resolve_line_values(
    description: String,
    quantity: f64,
    unit_price: f64,
    discount_pct: f64,
) -> AppResult<ResolvedInvoiceLine> {
    if description.trim().is_empty() {
        return Err(AppError::Validation(
            "Invoice line item description is required".into(),
        ));
    }
    if quantity <= 0.0 {
        return Err(AppError::Validation(
            "Invoice line item quantity must be greater than zero".into(),
        ));
    }
    if unit_price < 0.0 {
        return Err(AppError::Validation(
            "Invoice line item unit price cannot be negative".into(),
        ));
    }
    if !(0.0..=100.0).contains(&discount_pct) {
        return Err(AppError::Validation(
            "Invoice line item discountPct must be between 0 and 100".into(),
        ));
    }

    Ok(ResolvedInvoiceLine {
        description: description.trim().to_string(),
        quantity: round_qty(quantity),
        unit_price: round_money(unit_price),
        discount_pct,
    })
}

fn calculate_invoice_totals(
    discount_pct: Option<f64>,
    tax_amount: Option<f64>,
    lines: &[ResolvedInvoiceLine],
) -> AppResult<InvoiceTotals> {
    if lines.is_empty() {
        return Err(AppError::Validation(
            "Invoice must contain at least one line item".into(),
        ));
    }

    let normalized_discount_pct = round_money(discount_pct.unwrap_or(0.0));
    let normalized_tax_amount = round_money(tax_amount.unwrap_or(0.0));
    if normalized_discount_pct < 0.0 || normalized_discount_pct > 100.0 {
        return Err(AppError::Validation(
            "discountPct must be between 0 and 100".into(),
        ));
    }
    if normalized_tax_amount < 0.0 {
        return Err(AppError::Validation("taxAmount cannot be negative".into()));
    }

    let mut subtotal = 0.0;
    let mut line_discount_total = 0.0;
    for line in lines {
        let line_subtotal = round_money(line.quantity * line.unit_price);
        let line_discount = round_money(line_subtotal * line.discount_pct / 100.0);
        subtotal = round_money(subtotal + line_subtotal);
        line_discount_total = round_money(line_discount_total + line_discount);
    }

    let header_discount_base = round_money(subtotal - line_discount_total);
    let header_discount = round_money(header_discount_base * normalized_discount_pct / 100.0);
    let discount_amount = round_money(line_discount_total + header_discount);
    let total_amount = round_money(subtotal - discount_amount + normalized_tax_amount);
    if total_amount < 0.0 {
        return Err(AppError::Validation(
            "Invoice total cannot be negative".into(),
        ));
    }

    Ok(InvoiceTotals {
        subtotal,
        discount_pct: normalized_discount_pct,
        discount_amount,
        tax_amount: normalized_tax_amount,
        total_amount,
    })
}

fn calculate_line_total(quantity: f64, unit_price: f64, discount_pct: f64) -> f64 {
    let subtotal = round_money(round_qty(quantity) * round_money(unit_price));
    let discount = round_money(subtotal * round_money(discount_pct) / 100.0);
    round_money(subtotal - discount)
}

fn derive_invoice_status(amount_paid: f64, total_amount: f64) -> &'static str {
    if amount_paid >= total_amount - EPSILON && total_amount > EPSILON {
        STATUS_PAID
    } else if amount_paid > EPSILON {
        STATUS_PARTIAL
    } else {
        STATUS_PENDING
    }
}

fn parse_optional_datetime(
    value: &Option<String>,
    field_name: &str,
) -> AppResult<Option<DateTime<Utc>>> {
    match value {
        Some(raw) => {
            if raw.trim().is_empty() {
                return Ok(None);
            }
            let parsed = DateTime::parse_from_rfc3339(raw).map_err(|_| {
                AppError::Validation(format!("{} must be a valid RFC3339 datetime", field_name))
            })?;
            Ok(Some(parsed.with_timezone(&Utc)))
        }
        None => Ok(None),
    }
}

fn normalize_payment_method(value: Option<&str>) -> AppResult<Option<String>> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Ok(None);
    }
    let upper = normalized.to_uppercase();
    if matches!(
        upper.as_str(),
        "CASH" | "CARD" | "BANK_TRANSFER" | "WALLET" | "UPI" | "CHEQUE"
    ) {
        Ok(Some(upper))
    } else {
        Err(AppError::Validation(
            "paymentMethod must be one of CASH, CARD, BANK_TRANSFER, WALLET, UPI, or CHEQUE"
                .into(),
        ))
    }
}

fn normalize_payment_ref(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn merge_action_note(existing: Option<&str>, note: Option<&str>, prefix: &str) -> Option<String> {
    let existing_clean = existing
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    match note.map(str::trim).filter(|value| !value.is_empty()) {
        Some(addition) => match existing_clean {
            Some(current) => Some(format!("{}\n{}: {}", current, prefix, addition)),
            None => Some(format!("{}: {}", prefix, addition)),
        },
        None => existing_clean,
    }
}

fn ensure_uuid(value: &str, field_name: &str) -> AppResult<()> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| AppError::Validation(format!("{} must be a valid UUID", field_name)))
}

fn round_money(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round_qty(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_optional_datetime_accepts_blank_string() {
        let value = Some("  ".to_string());
        let parsed = parse_optional_datetime(&value, "paidAt").expect("blank paidAt should be valid");
        assert!(parsed.is_none());
    }

    #[test]
    fn normalize_payment_method_accepts_known_value_case_insensitive() {
        let value = normalize_payment_method(Some("card")).expect("card should be accepted");
        assert_eq!(value.as_deref(), Some("CARD"));
    }

    #[test]
    fn normalize_payment_method_rejects_unknown_value() {
        let err = normalize_payment_method(Some("crypto")).expect_err("unknown method should fail");
        match err {
            AppError::Validation(message) => assert!(message.contains("paymentMethod")),
            other => panic!("unexpected error type: {:?}", other),
        }
    }
}
