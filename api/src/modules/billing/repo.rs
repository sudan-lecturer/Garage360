use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use crate::errors::{AppError, AppResult};

use super::types::{
    InvoiceDetailRow, InvoiceLineCalcRow, InvoiceLineItemRow, InvoiceStateRow, InvoiceSummaryRow,
    JobCardInvoiceSeedRow,
};

pub async fn list(
    pool: &PgPool,
    search: &str,
    like: &str,
    status: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<InvoiceSummaryRow>> {
    sqlx::query_as::<_, InvoiceSummaryRow>(
        r#"
        SELECT
            i.id::text AS id,
            i.invoice_no,
            i.job_card_id::text AS job_card_id,
            jc.job_no,
            i.customer_id::text AS customer_id,
            COALESCE(
                NULLIF(BTRIM(CONCAT(COALESCE(c.first_name, ''), ' ', COALESCE(c.last_name, ''))), ''),
                c.company_name,
                c.phone
            ) AS customer_name,
            i.status,
            i.subtotal::text AS subtotal,
            i.discount_pct::text AS discount_pct,
            i.discount_amount::text AS discount_amount,
            i.tax_amount::text AS tax_amount,
            i.total_amount::text AS total_amount,
            i.amount_paid::text AS amount_paid,
            GREATEST(i.total_amount - i.amount_paid, 0)::text AS balance_due,
            i.payment_method,
            i.payment_ref,
            i.paid_at,
            i.created_at
        FROM invoices i
        JOIN customers c ON c.id = i.customer_id
        LEFT JOIN job_cards jc ON jc.id = i.job_card_id
        WHERE (
            $1 = ''
            OR COALESCE(c.first_name, '') ILIKE $2
            OR COALESCE(c.last_name, '') ILIKE $2
            OR COALESCE(c.company_name, '') ILIKE $2
            OR CAST(i.invoice_no AS TEXT) ILIKE $2
            OR i.status ILIKE $2
            OR COALESCE(i.payment_ref, '') ILIKE $2
        )
          AND ($3 = '' OR i.status = $3)
        ORDER BY i.created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count(pool: &PgPool, search: &str, like: &str, status: &str) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM invoices i
        JOIN customers c ON c.id = i.customer_id
        WHERE (
            $1 = ''
            OR COALESCE(c.first_name, '') ILIKE $2
            OR COALESCE(c.last_name, '') ILIKE $2
            OR COALESCE(c.company_name, '') ILIKE $2
            OR CAST(i.invoice_no AS TEXT) ILIKE $2
            OR i.status ILIKE $2
            OR COALESCE(i.payment_ref, '') ILIKE $2
        )
          AND ($3 = '' OR i.status = $3)
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(status)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_invoice(pool: &PgPool, id: &str) -> AppResult<Option<InvoiceDetailRow>> {
    sqlx::query_as::<_, InvoiceDetailRow>(
        r#"
        SELECT
            i.id::text AS id,
            i.invoice_no,
            i.job_card_id::text AS job_card_id,
            jc.job_no,
            i.customer_id::text AS customer_id,
            COALESCE(
                NULLIF(BTRIM(CONCAT(COALESCE(c.first_name, ''), ' ', COALESCE(c.last_name, ''))), ''),
                c.company_name,
                c.phone
            ) AS customer_name,
            i.status,
            i.subtotal::text AS subtotal,
            i.discount_pct::text AS discount_pct,
            i.discount_amount::text AS discount_amount,
            i.tax_amount::text AS tax_amount,
            i.total_amount::text AS total_amount,
            i.amount_paid::text AS amount_paid,
            GREATEST(i.total_amount - i.amount_paid, 0)::text AS balance_due,
            i.payment_method,
            i.payment_ref,
            i.paid_at,
            i.notes,
            i.created_at
        FROM invoices i
        JOIN customers c ON c.id = i.customer_id
        LEFT JOIN job_cards jc ON jc.id = i.job_card_id
        WHERE i.id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_invoice_lines(pool: &PgPool, id: &str) -> AppResult<Vec<InvoiceLineItemRow>> {
    sqlx::query_as::<_, InvoiceLineItemRow>(
        r#"
        SELECT
            id::text AS id,
            invoice_id::text AS invoice_id,
            description,
            quantity::text AS quantity,
            unit_price::text AS unit_price,
            discount_pct::text AS discount_pct,
            line_total::text AS line_total,
            created_at
        FROM invoice_line_items
        WHERE invoice_id = $1::uuid
        ORDER BY created_at ASC
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_invoice_state(pool: &PgPool, id: &str) -> AppResult<Option<InvoiceStateRow>> {
    sqlx::query_as::<_, InvoiceStateRow>(
        r#"
        SELECT
            id::text AS id,
            status,
            job_card_id::text AS job_card_id,
            customer_id::text AS customer_id,
            subtotal::float8 AS subtotal,
            discount_pct::float8 AS discount_pct,
            discount_amount::float8 AS discount_amount,
            tax_amount::float8 AS tax_amount,
            total_amount::float8 AS total_amount,
            amount_paid::float8 AS amount_paid,
            payment_method,
            payment_ref,
            paid_at,
            notes
        FROM invoices
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_invoice_line_calcs(
    pool: &PgPool,
    id: &str,
) -> AppResult<Vec<InvoiceLineCalcRow>> {
    sqlx::query_as::<_, InvoiceLineCalcRow>(
        r#"
        SELECT
            description,
            quantity::float8 AS quantity,
            unit_price::float8 AS unit_price,
            discount_pct::float8 AS discount_pct
        FROM invoice_line_items
        WHERE invoice_id = $1::uuid
        ORDER BY created_at ASC
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn customer_exists(pool: &PgPool, id: &str) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM customers
            WHERE id = $1::uuid AND is_active = true
        )
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_job_card_customer_id(
    pool: &PgPool,
    job_card_id: &str,
) -> AppResult<Option<String>> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT customer_id::text
        FROM job_cards
        WHERE id = $1::uuid AND is_active = true
        "#,
    )
    .bind(job_card_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn fetch_job_card_invoice_seed(
    pool: &PgPool,
    job_card_id: &str,
) -> AppResult<Vec<JobCardInvoiceSeedRow>> {
    sqlx::query_as::<_, JobCardInvoiceSeedRow>(
        r#"
        SELECT
            description,
            quantity::float8 AS quantity,
            unit_price::float8 AS unit_price,
            discount_pct::float8 AS discount_pct
        FROM job_card_items
        WHERE job_card_id = $1::uuid
          AND is_active = true
        ORDER BY created_at ASC
        "#,
    )
    .bind(job_card_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn insert_invoice(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    job_card_id: Option<&str>,
    customer_id: &str,
    subtotal: f64,
    discount_pct: f64,
    discount_amount: f64,
    tax_amount: f64,
    total_amount: f64,
    amount_paid: f64,
    status: &str,
    notes: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO invoices (
            id,
            job_card_id,
            customer_id,
            subtotal,
            discount_pct,
            discount_amount,
            tax_amount,
            total_amount,
            amount_paid,
            status,
            notes
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            $3::uuid,
            $4::numeric,
            $5::numeric,
            $6::numeric,
            $7::numeric,
            $8::numeric,
            $9::numeric,
            $10,
            $11
        )
        "#,
    )
    .bind(id)
    .bind(job_card_id)
    .bind(customer_id)
    .bind(subtotal)
    .bind(discount_pct)
    .bind(discount_amount)
    .bind(tax_amount)
    .bind(total_amount)
    .bind(amount_paid)
    .bind(status)
    .bind(notes)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn update_invoice_header(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    job_card_id: Option<&str>,
    customer_id: &str,
    subtotal: f64,
    discount_pct: f64,
    discount_amount: f64,
    tax_amount: f64,
    total_amount: f64,
    status: &str,
    notes: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE invoices
        SET job_card_id = $2::uuid,
            customer_id = $3::uuid,
            subtotal = $4::numeric,
            discount_pct = $5::numeric,
            discount_amount = $6::numeric,
            tax_amount = $7::numeric,
            total_amount = $8::numeric,
            status = $9,
            notes = $10
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .bind(job_card_id)
    .bind(customer_id)
    .bind(subtotal)
    .bind(discount_pct)
    .bind(discount_amount)
    .bind(tax_amount)
    .bind(total_amount)
    .bind(status)
    .bind(notes)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn delete_invoice_lines(tx: &mut Transaction<'_, Postgres>, id: &str) -> AppResult<()> {
    sqlx::query(
        r#"
        DELETE FROM invoice_line_items
        WHERE invoice_id = $1::uuid
        "#,
    )
    .bind(id)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn insert_invoice_line(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    invoice_id: &str,
    description: &str,
    quantity: f64,
    unit_price: f64,
    discount_pct: f64,
    line_total: f64,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO invoice_line_items (
            id,
            invoice_id,
            description,
            quantity,
            unit_price,
            discount_pct,
            line_total
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            $3,
            $4::numeric,
            $5::numeric,
            $6::numeric,
            $7::numeric
        )
        "#,
    )
    .bind(id)
    .bind(invoice_id)
    .bind(description)
    .bind(quantity)
    .bind(unit_price)
    .bind(discount_pct)
    .bind(line_total)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn refresh_invoice_totals(
    pool: &PgPool,
    id: &str,
    subtotal: f64,
    discount_amount: f64,
    tax_amount: f64,
    total_amount: f64,
    status: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE invoices
        SET subtotal = $2::numeric,
            discount_amount = $3::numeric,
            tax_amount = $4::numeric,
            total_amount = $5::numeric,
            status = $6
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .bind(subtotal)
    .bind(discount_amount)
    .bind(tax_amount)
    .bind(total_amount)
    .bind(status)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn update_payment(
    pool: &PgPool,
    id: &str,
    amount_paid: f64,
    status: &str,
    payment_method: Option<&str>,
    payment_ref: Option<&str>,
    paid_at: Option<DateTime<Utc>>,
    notes: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE invoices
        SET amount_paid = $2::numeric,
            status = $3,
            payment_method = $4,
            payment_ref = $5,
            paid_at = $6,
            notes = $7
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .bind(amount_paid)
    .bind(status)
    .bind(payment_method)
    .bind(payment_ref)
    .bind(paid_at)
    .bind(notes)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn update_status_and_notes(
    pool: &PgPool,
    id: &str,
    status: &str,
    notes: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE invoices
        SET status = $2,
            notes = $3
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(notes)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}
