use sqlx::PgPool;

use crate::errors::{AppError, AppResult};

use super::types::{
    CreateCustomerRequest, CustomerRow, FinancialSnapshotRow, InvoiceSummaryRow, JobSummaryRow,
    ServiceChronicleRow, VehicleSummaryRow,
};

pub async fn list(
    pool: &PgPool,
    search: &str,
    like: &str,
    customer_type: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<CustomerRow>> {
    sqlx::query_as::<_, CustomerRow>(
        r#"
        SELECT id::text AS id, customer_type, first_name, last_name, company_name, email, phone, address, created_at
        FROM customers
        WHERE is_active = true
          AND (
            $3 = ''
            OR $3 = 'BOTH'
            OR customer_type = $3
          )
          AND (
            $1 = ''
            OR first_name ILIKE $2
            OR last_name ILIKE $2
            OR company_name ILIKE $2
            OR phone ILIKE $2
            OR COALESCE(email, '') ILIKE $2
            OR EXISTS (
                SELECT 1
                FROM vehicles v
                WHERE v.customer_id = customers.id
                  AND v.is_active = true
                  AND (
                    v.registration_no ILIKE $2
                    OR v.make ILIKE $2
                    OR v.model ILIKE $2
                  )
            )
          )
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(customer_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count(pool: &PgPool, search: &str, like: &str, customer_type: &str) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM customers
        WHERE is_active = true
          AND (
            $3 = ''
            OR $3 = 'BOTH'
            OR customer_type = $3
          )
          AND (
            $1 = ''
            OR first_name ILIKE $2
            OR last_name ILIKE $2
            OR company_name ILIKE $2
            OR phone ILIKE $2
            OR COALESCE(email, '') ILIKE $2
            OR EXISTS (
                SELECT 1
                FROM vehicles v
                WHERE v.customer_id = customers.id
                  AND v.is_active = true
                  AND (
                    v.registration_no ILIKE $2
                    OR v.make ILIKE $2
                    OR v.model ILIKE $2
                  )
            )
          )
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(customer_type)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_by_id(pool: &PgPool, id: &str) -> AppResult<Option<CustomerRow>> {
    sqlx::query_as::<_, CustomerRow>(
        r#"
        SELECT id::text AS id, customer_type, first_name, last_name, company_name, email, phone, address, created_at
        FROM customers
        WHERE id = $1::uuid AND is_active = true
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn list_customer_vehicles(pool: &PgPool, customer_id: &str) -> AppResult<Vec<VehicleSummaryRow>> {
    sqlx::query_as::<_, VehicleSummaryRow>(
        r#"
        SELECT id::text AS id, registration_no, make, model, year
        FROM vehicles
        WHERE customer_id = $1::uuid
          AND is_active = true
        ORDER BY created_at DESC
        LIMIT 20
        "#,
    )
    .bind(customer_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn list_customer_jobs(pool: &PgPool, customer_id: &str) -> AppResult<Vec<JobSummaryRow>> {
    sqlx::query_as::<_, JobSummaryRow>(
        r#"
        SELECT id::text AS id, job_no, status::text AS status, created_at
        FROM job_cards
        WHERE customer_id = $1::uuid
          AND is_active = true
        ORDER BY created_at DESC, job_no DESC
        LIMIT 20
        "#,
    )
    .bind(customer_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn list_customer_invoices(pool: &PgPool, customer_id: &str) -> AppResult<Vec<InvoiceSummaryRow>> {
    sqlx::query_as::<_, InvoiceSummaryRow>(
        r#"
        SELECT id::text AS id, invoice_no, status, total_amount::text AS total_amount, created_at
        FROM invoices
        WHERE customer_id = $1::uuid
        ORDER BY created_at DESC, invoice_no DESC
        LIMIT 20
        "#,
    )
    .bind(customer_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn get_customer_financial_snapshot(
    pool: &PgPool,
    customer_id: &str,
) -> AppResult<FinancialSnapshotRow> {
    sqlx::query_as::<_, FinancialSnapshotRow>(
        r#"
        SELECT
            COUNT(*)::bigint AS total_invoices,
            COALESCE(SUM(total_amount), 0)::text AS total_spend,
            COALESCE(SUM(GREATEST(total_amount - amount_paid, 0)), 0)::text AS outstanding_balance,
            COUNT(*) FILTER (WHERE status = 'PAID')::bigint AS paid_invoices,
            MAX(created_at) AS last_invoice_at
        FROM invoices
        WHERE customer_id = $1::uuid
          AND status <> 'VOID'
        "#,
    )
    .bind(customer_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn list_customer_service_chronicle(
    pool: &PgPool,
    customer_id: &str,
) -> AppResult<Vec<ServiceChronicleRow>> {
    sqlx::query_as::<_, ServiceChronicleRow>(
        r#"
        SELECT
            ('job:' || jc.id::text) AS id,
            'JOB' AS kind,
            ('JOB-' || jc.job_no::text) AS reference_no,
            jc.status::text AS status,
            jc.created_at AS occurred_at,
            COALESCE(NULLIF(jc.complaint, ''), 'Service job opened') AS summary
        FROM job_cards jc
        WHERE jc.customer_id = $1::uuid
          AND jc.is_active = true
        UNION ALL
        SELECT
            ('invoice:' || i.id::text) AS id,
            'INVOICE' AS kind,
            ('INV-' || COALESCE(i.invoice_no::text, '-')) AS reference_no,
            i.status AS status,
            i.created_at AS occurred_at,
            COALESCE(NULLIF(i.notes, ''), 'Invoice generated') AS summary
        FROM invoices i
        WHERE i.customer_id = $1::uuid
        ORDER BY occurred_at DESC
        LIMIT 30
        "#,
    )
    .bind(customer_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn create(
    pool: &PgPool,
    req: &CreateCustomerRequest,
    created_by: &str,
) -> AppResult<CustomerRow> {
    sqlx::query_as::<_, CustomerRow>(
        r#"
        INSERT INTO customers (
            customer_type, first_name, last_name, company_name, email, phone, address, created_by
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8::uuid)
        RETURNING id::text AS id, customer_type, first_name, last_name, company_name, email, phone, address, created_at
        "#,
    )
    .bind(&req.customer_type)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&req.company_name)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.address)
    .bind(created_by)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update(
    pool: &PgPool,
    id: &str,
    req: &CreateCustomerRequest,
) -> AppResult<Option<CustomerRow>> {
    sqlx::query_as::<_, CustomerRow>(
        r#"
        UPDATE customers
        SET first_name = $2,
            last_name = $3,
            company_name = $4,
            email = $5,
            phone = $6,
            address = $7,
            updated_at = NOW()
        WHERE id = $1::uuid AND is_active = true
        RETURNING id::text AS id, customer_type, first_name, last_name, company_name, email, phone, address, created_at
        "#,
    )
    .bind(id)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&req.company_name)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.address)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn soft_delete(pool: &PgPool, id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE customers
        SET is_active = false, updated_at = NOW()
        WHERE id = $1::uuid AND is_active = true
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}
