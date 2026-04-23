use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::errors::{AppError, AppResult};

use super::types::{
    CustomerActivityRow, JobStatusCountRow, JobSummaryRow, MechanicJobCountRow,
    RevenueStatusRow, RevenueTotalsRow, SavedReportRow,
};

pub async fn list_saved_reports(pool: &PgPool, created_by: &str) -> AppResult<Vec<SavedReportRow>> {
    sqlx::query_as::<_, SavedReportRow>(
        r#"
        SELECT id::text AS id, name, report_type, config, created_at
        FROM saved_reports
        WHERE created_by = $1::uuid
        ORDER BY created_at DESC
        "#,
    )
    .bind(created_by)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn create_saved_report(
    pool: &PgPool,
    name: &str,
    report_type: &str,
    config: &Value,
    created_by: &str,
) -> AppResult<SavedReportRow> {
    sqlx::query_as::<_, SavedReportRow>(
        r#"
        INSERT INTO saved_reports (name, report_type, config, created_by)
        VALUES ($1, $2, $3, $4::uuid)
        RETURNING id::text AS id, name, report_type, config, created_at
        "#,
    )
    .bind(name)
    .bind(report_type)
    .bind(config)
    .bind(created_by)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_saved_report(pool: &PgPool, id: &str, created_by: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        DELETE FROM saved_reports
        WHERE id = $1::uuid
          AND created_by = $2::uuid
        "#,
    )
    .bind(id)
    .bind(created_by)
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn fetch_revenue_totals(
    pool: &PgPool,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
) -> AppResult<RevenueTotalsRow> {
    sqlx::query_as::<_, RevenueTotalsRow>(
        r#"
        SELECT
            COUNT(*) AS invoice_count,
            COALESCE(SUM(total_amount), 0)::text AS total_invoiced,
            COALESCE(SUM(amount_paid), 0)::text AS total_collected,
            COALESCE(SUM(total_amount - amount_paid), 0)::text AS total_outstanding
        FROM invoices
        WHERE ($1::timestamptz IS NULL OR created_at >= $1)
          AND ($2::timestamptz IS NULL OR created_at <= $2)
        "#,
    )
    .bind(date_from)
    .bind(date_to)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn fetch_revenue_status_breakdown(
    pool: &PgPool,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
) -> AppResult<Vec<RevenueStatusRow>> {
    sqlx::query_as::<_, RevenueStatusRow>(
        r#"
        SELECT
            status,
            COUNT(*) AS invoice_count,
            COALESCE(SUM(total_amount), 0)::text AS total_amount,
            COALESCE(SUM(amount_paid), 0)::text AS amount_paid
        FROM invoices
        WHERE ($1::timestamptz IS NULL OR created_at >= $1)
          AND ($2::timestamptz IS NULL OR created_at <= $2)
        GROUP BY status
        ORDER BY status ASC
        "#,
    )
    .bind(date_from)
    .bind(date_to)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn fetch_job_summary(
    pool: &PgPool,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
) -> AppResult<JobSummaryRow> {
    sqlx::query_as::<_, JobSummaryRow>(
        r#"
        SELECT
            COUNT(*) AS total_jobs,
            COUNT(*) FILTER (WHERE status = 'COMPLETED') AS completed_jobs,
            COUNT(*) FILTER (WHERE status = 'CANCELLED') AS cancelled_jobs,
            COUNT(*) FILTER (WHERE status NOT IN ('COMPLETED', 'CANCELLED')) AS open_jobs
        FROM job_cards
        WHERE is_active = true
          AND ($1::timestamptz IS NULL OR created_at >= $1)
          AND ($2::timestamptz IS NULL OR created_at <= $2)
        "#,
    )
    .bind(date_from)
    .bind(date_to)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn fetch_job_status_counts(
    pool: &PgPool,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
) -> AppResult<Vec<JobStatusCountRow>> {
    sqlx::query_as::<_, JobStatusCountRow>(
        r#"
        SELECT
            status::text AS status,
            COUNT(*) AS job_count
        FROM job_cards
        WHERE is_active = true
          AND ($1::timestamptz IS NULL OR created_at >= $1)
          AND ($2::timestamptz IS NULL OR created_at <= $2)
        GROUP BY status
        ORDER BY status ASC
        "#,
    )
    .bind(date_from)
    .bind(date_to)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn fetch_mechanic_job_counts(
    pool: &PgPool,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
    limit: i64,
) -> AppResult<Vec<MechanicJobCountRow>> {
    sqlx::query_as::<_, MechanicJobCountRow>(
        r#"
        SELECT
            COALESCE(u.name, 'Unassigned') AS mechanic_name,
            COUNT(*) AS job_count
        FROM job_cards jc
        LEFT JOIN users u ON u.id = jc.mechanic_id
        WHERE jc.is_active = true
          AND ($1::timestamptz IS NULL OR jc.created_at >= $1)
          AND ($2::timestamptz IS NULL OR jc.created_at <= $2)
        GROUP BY COALESCE(u.name, 'Unassigned')
        ORDER BY job_count DESC, mechanic_name ASC
        LIMIT $3
        "#,
    )
    .bind(date_from)
    .bind(date_to)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn fetch_customer_activity(
    pool: &PgPool,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
    limit: i64,
) -> AppResult<Vec<CustomerActivityRow>> {
    sqlx::query_as::<_, CustomerActivityRow>(
        r#"
        SELECT
            c.id::text AS customer_id,
            COALESCE(
                NULLIF(TRIM(CONCAT_WS(' ', c.first_name, c.last_name)), ''),
                c.company_name,
                c.phone
            ) AS customer_name,
            COALESCE(job_stats.job_count, 0) AS job_count,
            COALESCE(invoice_stats.invoiced_amount, '0') AS invoiced_amount,
            COALESCE(invoice_stats.paid_amount, '0') AS paid_amount
        FROM customers c
        LEFT JOIN (
            SELECT customer_id, COUNT(*) AS job_count
            FROM job_cards
            WHERE is_active = true
              AND ($1::timestamptz IS NULL OR created_at >= $1)
              AND ($2::timestamptz IS NULL OR created_at <= $2)
            GROUP BY customer_id
        ) AS job_stats ON job_stats.customer_id = c.id
        LEFT JOIN (
            SELECT
                customer_id,
                COALESCE(SUM(total_amount), 0)::text AS invoiced_amount,
                COALESCE(SUM(amount_paid), 0)::text AS paid_amount
            FROM invoices
            WHERE ($1::timestamptz IS NULL OR created_at >= $1)
              AND ($2::timestamptz IS NULL OR created_at <= $2)
            GROUP BY customer_id
        ) AS invoice_stats ON invoice_stats.customer_id = c.id
        WHERE c.is_active = true
          AND (
            COALESCE(job_stats.job_count, 0) > 0
            OR COALESCE(invoice_stats.invoiced_amount, '0') <> '0'
            OR COALESCE(invoice_stats.paid_amount, '0') <> '0'
          )
        ORDER BY
            COALESCE(job_stats.job_count, 0) DESC,
            COALESCE(invoice_stats.paid_amount, '0')::numeric DESC,
            customer_name ASC
        LIMIT $3
        "#,
    )
    .bind(date_from)
    .bind(date_to)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}
