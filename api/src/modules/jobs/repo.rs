use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};

use crate::errors::{AppError, AppResult};

use super::types::{
    ActivityRow, ApprovalRow, ApprovalRequest, BaySummaryRow, ChangeRequestItemRequest,
    ChangeRequestItemRow, ChangeRequestRow, CreateChangeRequestRequest, CreateJobRequest,
    CustomerSignatureRow, IntakeChecklistRow, IntakePhotoRow, JobItemRequest, JobItemRow, JobRow,
    LockedJobRow, UserSummaryRow,
};

const JOB_SELECT: &str = r#"
SELECT
    jc.id::text AS id,
    jc.job_no,
    jc.vehicle_id::text AS vehicle_id,
    v.registration_no AS vehicle_registration_no,
    v.make AS vehicle_make,
    v.model AS vehicle_model,
    jc.customer_id::text AS customer_id,
    COALESCE(NULLIF(TRIM(CONCAT_WS(' ', c.first_name, c.last_name)), ''), c.company_name, c.phone) AS customer_name,
    jc.bay_id::text AS bay_id,
    sb.code AS bay_code,
    sb.name AS bay_name,
    jc.status::text AS status,
    jc.complaint,
    jc.diagnosis,
    jc.odometer_in,
    jc.odometer_out,
    jc.estimated_completion,
    jc.mechanic_id::text AS mechanic_id,
    mechanic.name AS mechanic_name,
    jc.account_manager_id::text AS account_manager_id,
    account_manager.name AS account_manager_name,
    jc.qa_by::text AS qa_by,
    qa_user.name AS qa_name,
    jc.qa_cycles,
    jc.created_at,
    jc.updated_at
FROM job_cards jc
JOIN vehicles v ON v.id = jc.vehicle_id
JOIN customers c ON c.id = jc.customer_id
LEFT JOIN service_bays sb ON sb.id = jc.bay_id
LEFT JOIN users mechanic ON mechanic.id = jc.mechanic_id
LEFT JOIN users account_manager ON account_manager.id = jc.account_manager_id
LEFT JOIN users qa_user ON qa_user.id = jc.qa_by
"#;

const JOB_ITEM_SELECT: &str = r#"
SELECT
    id::text AS id,
    job_card_id::text AS job_card_id,
    item_type,
    description,
    quantity::text AS quantity,
    unit_price::text AS unit_price,
    discount_pct::text AS discount_pct,
    (quantity * unit_price * ((100 - discount_pct) / 100))::text AS line_total,
    created_at
FROM job_card_items
"#;

const ACTIVITY_SELECT: &str = r#"
SELECT
    a.id::text AS id,
    a.job_card_id::text AS job_card_id,
    a.action,
    a.description,
    a.metadata,
    a.performed_by::text AS performed_by,
    u.name AS performed_by_name,
    a.created_at
FROM job_card_activities a
LEFT JOIN users u ON u.id = a.performed_by
"#;

const APPROVAL_SELECT: &str = r#"
SELECT
    a.id::text AS id,
    a.job_card_id::text AS job_card_id,
    a.approved_by::text AS approved_by,
    u.name AS approved_by_name,
    a.approval_type,
    a.channel,
    a.notes,
    a.created_at
FROM job_card_approvals a
LEFT JOIN users u ON u.id = a.approved_by
"#;

const CHANGE_REQUEST_SELECT: &str = r#"
SELECT
    id::text AS id,
    job_card_id::text AS job_card_id,
    status,
    requested_by::text AS requested_by,
    approved_by::text AS approved_by,
    notes,
    created_at,
    resolved_at
FROM job_change_requests
"#;

const CHANGE_REQUEST_ITEM_SELECT: &str = r#"
SELECT
    id::text AS id,
    change_request_id::text AS change_request_id,
    description,
    quantity::text AS quantity,
    unit_price::text AS unit_price,
    created_at
FROM job_change_request_items
"#;

pub async fn list(
    pool: &PgPool,
    search: &str,
    like: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<JobRow>> {
    let query = format!(
        r#"{JOB_SELECT}
        WHERE jc.is_active = true
          AND (
            $1 = ''
            OR jc.job_no::text ILIKE $2
            OR v.registration_no ILIKE $2
            OR v.make ILIKE $2
            OR v.model ILIKE $2
            OR COALESCE(NULLIF(TRIM(CONCAT_WS(' ', c.first_name, c.last_name)), ''), c.company_name, c.phone) ILIKE $2
            OR COALESCE(jc.complaint, '') ILIKE $2
            OR jc.status::text ILIKE $2
          )
        ORDER BY jc.created_at DESC, jc.job_no DESC
        LIMIT $3 OFFSET $4"#
    );

    sqlx::query_as::<_, JobRow>(&query)
        .bind(search)
        .bind(like)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn count(pool: &PgPool, search: &str, like: &str) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM job_cards jc
        JOIN vehicles v ON v.id = jc.vehicle_id
        JOIN customers c ON c.id = jc.customer_id
        WHERE jc.is_active = true
          AND (
            $1 = ''
            OR jc.job_no::text ILIKE $2
            OR v.registration_no ILIKE $2
            OR v.make ILIKE $2
            OR v.model ILIKE $2
            OR COALESCE(NULLIF(TRIM(CONCAT_WS(' ', c.first_name, c.last_name)), ''), c.company_name, c.phone) ILIKE $2
            OR COALESCE(jc.complaint, '') ILIKE $2
            OR jc.status::text ILIKE $2
          )
        "#,
    )
    .bind(search)
    .bind(like)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_by_id(pool: &PgPool, id: &str) -> AppResult<Option<JobRow>> {
    let query = format!(
        r#"{JOB_SELECT}
        WHERE jc.id = $1::uuid
          AND jc.is_active = true"#
    );

    sqlx::query_as::<_, JobRow>(&query)
        .bind(id)
        .fetch_optional(pool)
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

pub async fn vehicle_exists_for_customer(
    pool: &PgPool,
    vehicle_id: &str,
    customer_id: &str,
) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM vehicles
            WHERE id = $1::uuid
              AND customer_id = $2::uuid
              AND is_active = true
        )
        "#,
    )
    .bind(vehicle_id)
    .bind(customer_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_user_by_id(pool: &PgPool, id: &str) -> AppResult<Option<UserSummaryRow>> {
    sqlx::query_as::<_, UserSummaryRow>(
        r#"
        SELECT id::text AS id, name, role
        FROM users
        WHERE id = $1::uuid AND is_active = true
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn active_item_count(pool: &PgPool, job_id: &str) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM job_card_items
        WHERE job_card_id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(job_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn has_completed_intake_checklist(pool: &PgPool, job_id: &str) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM intake_checklists
            WHERE job_card_id = $1::uuid
              AND completed_at IS NOT NULL
        )
        "#,
    )
    .bind(job_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn has_customer_signature(pool: &PgPool, job_id: &str) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM customer_signatures
            WHERE job_card_id = $1::uuid
              AND signed_at IS NOT NULL
        )
        "#,
    )
    .bind(job_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn has_approval(pool: &PgPool, job_id: &str) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM job_card_approvals
            WHERE job_card_id = $1::uuid
        )
        "#,
    )
    .bind(job_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn has_paid_invoice(pool: &PgPool, job_id: &str) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM invoices
            WHERE job_card_id = $1::uuid
              AND status = 'PAID'
        )
        "#,
    )
    .bind(job_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn list_items(pool: &PgPool, job_id: &str) -> AppResult<Vec<JobItemRow>> {
    let query = format!(
        r#"{JOB_ITEM_SELECT}
        WHERE job_card_id = $1::uuid
          AND is_active = true
        ORDER BY created_at ASC"#
    );

    sqlx::query_as::<_, JobItemRow>(&query)
        .bind(job_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn find_item(pool: &PgPool, job_id: &str, item_id: &str) -> AppResult<Option<JobItemRow>> {
    let query = format!(
        r#"{JOB_ITEM_SELECT}
        WHERE job_card_id = $1::uuid
          AND id = $2::uuid
          AND is_active = true"#
    );

    sqlx::query_as::<_, JobItemRow>(&query)
        .bind(job_id)
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn list_activities(pool: &PgPool, job_id: &str) -> AppResult<Vec<ActivityRow>> {
    let query = format!(
        r#"{ACTIVITY_SELECT}
        WHERE a.job_card_id = $1::uuid
        ORDER BY a.created_at DESC"#
    );

    sqlx::query_as::<_, ActivityRow>(&query)
        .bind(job_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn find_activity_by_id(
    pool: &PgPool,
    job_id: &str,
    activity_id: &str,
) -> AppResult<Option<ActivityRow>> {
    let query = format!(
        r#"{ACTIVITY_SELECT}
        WHERE a.job_card_id = $1::uuid
          AND a.id = $2::uuid"#
    );

    sqlx::query_as::<_, ActivityRow>(&query)
        .bind(job_id)
        .bind(activity_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn list_qa_activities(pool: &PgPool, job_id: &str) -> AppResult<Vec<ActivityRow>> {
    let query = format!(
        r#"{ACTIVITY_SELECT}
        WHERE a.job_card_id = $1::uuid
          AND a.action IN ('qa.passed', 'qa.failed')
        ORDER BY a.created_at DESC"#
    );

    sqlx::query_as::<_, ActivityRow>(&query)
        .bind(job_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn list_approvals(pool: &PgPool, job_id: &str) -> AppResult<Vec<ApprovalRow>> {
    let query = format!(
        r#"{APPROVAL_SELECT}
        WHERE a.job_card_id = $1::uuid
        ORDER BY a.created_at DESC"#
    );

    sqlx::query_as::<_, ApprovalRow>(&query)
        .bind(job_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn find_approval_by_id(
    pool: &PgPool,
    job_id: &str,
    approval_id: &str,
) -> AppResult<Option<ApprovalRow>> {
    let query = format!(
        r#"{APPROVAL_SELECT}
        WHERE a.job_card_id = $1::uuid
          AND a.id = $2::uuid"#
    );

    sqlx::query_as::<_, ApprovalRow>(&query)
        .bind(job_id)
        .bind(approval_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn list_change_requests(pool: &PgPool, job_id: &str) -> AppResult<Vec<ChangeRequestRow>> {
    let query = format!(
        r#"{CHANGE_REQUEST_SELECT}
        WHERE job_card_id = $1::uuid
        ORDER BY created_at DESC"#
    );

    sqlx::query_as::<_, ChangeRequestRow>(&query)
        .bind(job_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn find_change_request(
    pool: &PgPool,
    job_id: &str,
    change_request_id: &str,
) -> AppResult<Option<ChangeRequestRow>> {
    let query = format!(
        r#"{CHANGE_REQUEST_SELECT}
        WHERE job_card_id = $1::uuid
          AND id = $2::uuid"#
    );

    sqlx::query_as::<_, ChangeRequestRow>(&query)
        .bind(job_id)
        .bind(change_request_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn list_change_request_items(
    pool: &PgPool,
    change_request_id: &str,
) -> AppResult<Vec<ChangeRequestItemRow>> {
    let query = format!(
        r#"{CHANGE_REQUEST_ITEM_SELECT}
        WHERE change_request_id = $1::uuid
        ORDER BY created_at ASC"#
    );

    sqlx::query_as::<_, ChangeRequestItemRow>(&query)
        .bind(change_request_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create_job(
    tx: &mut Transaction<'_, Postgres>,
    req: &CreateJobRequest,
    created_by: &str,
    estimated_completion: Option<DateTime<Utc>>,
) -> AppResult<String> {
    sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO job_cards (
            vehicle_id,
            customer_id,
            complaint,
            diagnosis,
            odometer_in,
            estimated_completion,
            created_by
        )
        VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7::uuid)
        RETURNING id::text
        "#,
    )
    .bind(&req.vehicle_id)
    .bind(&req.customer_id)
    .bind(&req.complaint)
    .bind(&req.diagnosis)
    .bind(req.odometer_in)
    .bind(estimated_completion)
    .bind(created_by)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::Database)
}

pub async fn update_job(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    customer_id: &str,
    vehicle_id: &str,
    complaint: Option<&str>,
    diagnosis: Option<&str>,
    odometer_in: Option<i32>,
    odometer_out: Option<i32>,
    estimated_completion: Option<DateTime<Utc>>,
) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE job_cards
        SET customer_id = $2::uuid,
            vehicle_id = $3::uuid,
            complaint = $4,
            diagnosis = $5,
            odometer_in = $6,
            odometer_out = $7,
            estimated_completion = $8,
            updated_at = NOW()
        WHERE id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(id)
    .bind(customer_id)
    .bind(vehicle_id)
    .bind(complaint)
    .bind(diagnosis)
    .bind(odometer_in)
    .bind(odometer_out)
    .bind(estimated_completion)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn insert_activity(
    tx: &mut Transaction<'_, Postgres>,
    job_id: &str,
    action: &str,
    description: Option<String>,
    metadata: Option<Value>,
    performed_by: Option<&str>,
) -> AppResult<String> {
    sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO job_card_activities (
            job_card_id,
            action,
            description,
            metadata,
            performed_by
        )
        VALUES ($1::uuid, $2, $3, $4, $5::uuid)
        RETURNING id::text
        "#,
    )
    .bind(job_id)
    .bind(action)
    .bind(description)
    .bind(metadata)
    .bind(performed_by)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::Database)
}

pub async fn lock_job_for_update(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
) -> AppResult<Option<LockedJobRow>> {
    sqlx::query_as::<_, LockedJobRow>(
        r#"
        SELECT
            id::text AS id,
            status::text AS status,
            complaint,
            odometer_out,
            mechanic_id::text AS mechanic_id,
            bay_id::text AS bay_id,
            qa_by::text AS qa_by
        FROM job_cards
        WHERE id = $1::uuid
          AND is_active = true
        FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::Database)
}

pub async fn update_job_status(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    status: &str,
) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE job_cards
        SET status = $2::job_status,
            updated_at = NOW()
        WHERE id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(id)
    .bind(status)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn apply_qa_result(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    status: &str,
    qa_by: &str,
    increment_qa_cycles: bool,
) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE job_cards
        SET status = $2::job_status,
            qa_by = $3::uuid,
            qa_cycles = qa_cycles + CASE WHEN $4 THEN 1 ELSE 0 END,
            updated_at = NOW()
        WHERE id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(qa_by)
    .bind(increment_qa_cycles)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn set_mechanic(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    mechanic_id: &str,
) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE job_cards
        SET mechanic_id = $2::uuid,
            updated_at = NOW()
        WHERE id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(id)
    .bind(mechanic_id)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn set_account_manager(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    account_manager_id: &str,
) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE job_cards
        SET account_manager_id = $2::uuid,
            updated_at = NOW()
        WHERE id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(id)
    .bind(account_manager_id)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn set_bay(tx: &mut Transaction<'_, Postgres>, id: &str, bay_id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE job_cards
        SET bay_id = $2::uuid,
            updated_at = NOW()
        WHERE id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(id)
    .bind(bay_id)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn set_estimated_completion(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    estimated_completion: DateTime<Utc>,
) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE job_cards
        SET estimated_completion = $2,
            updated_at = NOW()
        WHERE id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(id)
    .bind(estimated_completion)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn create_item(pool: &PgPool, job_id: &str, req: &JobItemRequest) -> AppResult<JobItemRow> {
    sqlx::query_as::<_, JobItemRow>(
        r#"
        INSERT INTO job_card_items (
            job_card_id,
            item_type,
            description,
            quantity,
            unit_price,
            discount_pct
        )
        VALUES ($1::uuid, $2, $3, $4::numeric, $5::numeric, $6::numeric)
        RETURNING
            id::text AS id,
            job_card_id::text AS job_card_id,
            item_type,
            description,
            quantity::text AS quantity,
            unit_price::text AS unit_price,
            discount_pct::text AS discount_pct,
            (quantity * unit_price * ((100 - discount_pct) / 100))::text AS line_total,
            created_at
        "#,
    )
    .bind(job_id)
    .bind(&req.item_type)
    .bind(&req.description)
    .bind(&req.quantity)
    .bind(&req.unit_price)
    .bind(req.discount_pct.as_deref().unwrap_or("0"))
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_item(
    pool: &PgPool,
    job_id: &str,
    item_id: &str,
    req: &JobItemRequest,
) -> AppResult<Option<JobItemRow>> {
    sqlx::query_as::<_, JobItemRow>(
        r#"
        UPDATE job_card_items
        SET item_type = $3,
            description = $4,
            quantity = $5::numeric,
            unit_price = $6::numeric,
            discount_pct = $7::numeric
        WHERE job_card_id = $1::uuid
          AND id = $2::uuid
          AND is_active = true
        RETURNING
            id::text AS id,
            job_card_id::text AS job_card_id,
            item_type,
            description,
            quantity::text AS quantity,
            unit_price::text AS unit_price,
            discount_pct::text AS discount_pct,
            (quantity * unit_price * ((100 - discount_pct) / 100))::text AS line_total,
            created_at
        "#,
    )
    .bind(job_id)
    .bind(item_id)
    .bind(&req.item_type)
    .bind(&req.description)
    .bind(&req.quantity)
    .bind(&req.unit_price)
    .bind(req.discount_pct.as_deref().unwrap_or("0"))
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn soft_delete_item(pool: &PgPool, job_id: &str, item_id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE job_card_items
        SET is_active = false
        WHERE job_card_id = $1::uuid
          AND id = $2::uuid
          AND is_active = true
        "#,
    )
    .bind(job_id)
    .bind(item_id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn create_approval(
    tx: &mut Transaction<'_, Postgres>,
    job_id: &str,
    approved_by: &str,
    req: &ApprovalRequest,
) -> AppResult<String> {
    sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO job_card_approvals (
            job_card_id,
            approved_by,
            approval_type,
            channel,
            notes
        )
        VALUES ($1::uuid, $2::uuid, $3, $4, $5)
        RETURNING id::text
        "#,
    )
    .bind(job_id)
    .bind(approved_by)
    .bind(&req.approval_type)
    .bind(&req.channel)
    .bind(&req.notes)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::Database)
}

pub async fn create_change_request(
    tx: &mut Transaction<'_, Postgres>,
    job_id: &str,
    requested_by: &str,
    req: &CreateChangeRequestRequest,
) -> AppResult<String> {
    sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO job_change_requests (
            job_card_id,
            requested_by,
            notes
        )
        VALUES ($1::uuid, $2::uuid, $3)
        RETURNING id::text
        "#,
    )
    .bind(job_id)
    .bind(requested_by)
    .bind(&req.notes)
    .fetch_one(&mut **tx)
    .await
    .map_err(AppError::Database)
}

pub async fn create_change_request_item(
    tx: &mut Transaction<'_, Postgres>,
    change_request_id: &str,
    item: &ChangeRequestItemRequest,
) -> AppResult<u64> {
    sqlx::query(
        r#"
        INSERT INTO job_change_request_items (
            change_request_id,
            description,
            quantity,
            unit_price
        )
        VALUES ($1::uuid, $2, $3::numeric, $4::numeric)
        "#,
    )
    .bind(change_request_id)
    .bind(&item.description)
    .bind(&item.quantity)
    .bind(&item.unit_price)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn lock_change_request_for_update(
    tx: &mut Transaction<'_, Postgres>,
    job_id: &str,
    change_request_id: &str,
) -> AppResult<Option<ChangeRequestRow>> {
    let query = format!(
        r#"{CHANGE_REQUEST_SELECT}
        WHERE job_card_id = $1::uuid
          AND id = $2::uuid
        FOR UPDATE"#
    );

    sqlx::query_as::<_, ChangeRequestRow>(&query)
        .bind(job_id)
        .bind(change_request_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(AppError::Database)
}

pub async fn list_change_request_items_tx(
    tx: &mut Transaction<'_, Postgres>,
    change_request_id: &str,
) -> AppResult<Vec<ChangeRequestItemRow>> {
    let query = format!(
        r#"{CHANGE_REQUEST_ITEM_SELECT}
        WHERE change_request_id = $1::uuid
        ORDER BY created_at ASC"#
    );

    sqlx::query_as::<_, ChangeRequestItemRow>(&query)
        .bind(change_request_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(AppError::Database)
}

pub async fn update_change_request_status(
    tx: &mut Transaction<'_, Postgres>,
    change_request_id: &str,
    status: &str,
    approved_by: &str,
    notes: Option<&str>,
) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE job_change_requests
        SET status = $2,
            approved_by = $3::uuid,
            notes = $4,
            resolved_at = NOW()
        WHERE id = $1::uuid
        "#,
    )
    .bind(change_request_id)
    .bind(status)
    .bind(approved_by)
    .bind(notes)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn insert_item_from_change_request(
    tx: &mut Transaction<'_, Postgres>,
    job_id: &str,
    item: &ChangeRequestItemRow,
) -> AppResult<u64> {
    sqlx::query(
        r#"
        INSERT INTO job_card_items (
            job_card_id,
            item_type,
            description,
            quantity,
            unit_price,
            discount_pct
        )
        VALUES ($1::uuid, 'PART', $2, $3::numeric, $4::numeric, 0)
        "#,
    )
    .bind(job_id)
    .bind(&item.description)
    .bind(&item.quantity)
    .bind(&item.unit_price)
    .execute(&mut **tx)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn lock_bay_for_update(
    tx: &mut Transaction<'_, Postgres>,
    bay_id: &str,
) -> AppResult<Option<BaySummaryRow>> {
    sqlx::query_as::<_, BaySummaryRow>(
        r#"
        SELECT id::text AS id, code, name, capacity
        FROM service_bays
        WHERE id = $1::uuid
          AND is_active = true
        FOR UPDATE
        "#,
    )
    .bind(bay_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::Database)
}

pub async fn find_conflicting_job_in_bay(
    tx: &mut Transaction<'_, Postgres>,
    bay_id: &str,
    current_job_id: &str,
) -> AppResult<Option<String>> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT id::text
        FROM job_cards
        WHERE bay_id = $1::uuid
          AND id <> $2::uuid
          AND is_active = true
          AND status NOT IN ('COMPLETED', 'CANCELLED')
        LIMIT 1
        "#,
    )
    .bind(bay_id)
    .bind(current_job_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(AppError::Database)
}

pub async fn get_intake_checklist(pool: &PgPool, job_id: &str) -> AppResult<Option<IntakeChecklistRow>> {
    sqlx::query_as::<_, IntakeChecklistRow>(
        r#"
        SELECT
            id::text AS id,
            template_id::text AS template_id,
            data,
            completed_at,
            created_at
        FROM intake_checklists
        WHERE job_card_id = $1::uuid
        "#,
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn upsert_intake_checklist(
    pool: &PgPool,
    job_id: &str,
    template_id: Option<&str>,
    data: &Value,
    completed: bool,
) -> AppResult<IntakeChecklistRow> {
    sqlx::query_as::<_, IntakeChecklistRow>(
        r#"
        INSERT INTO intake_checklists (job_card_id, template_id, data, completed_at)
        VALUES (
            $1::uuid,
            CASE WHEN $2 = '' THEN NULL ELSE $2::uuid END,
            $3::jsonb,
            CASE WHEN $4 THEN NOW() ELSE NULL END
        )
        ON CONFLICT (job_card_id) DO UPDATE SET
            template_id = CASE WHEN $2 = '' THEN intake_checklists.template_id ELSE $2::uuid END,
            data = EXCLUDED.data,
            completed_at = CASE WHEN $4 THEN NOW() ELSE intake_checklists.completed_at END
        RETURNING
            id::text AS id,
            template_id::text AS template_id,
            data,
            completed_at,
            created_at
        "#,
    )
    .bind(job_id)
    .bind(template_id.unwrap_or(""))
    .bind(data)
    .bind(completed)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn insert_intake_photo(
    pool: &PgPool,
    job_id: &str,
    photo_type: &str,
    file_path: &str,
    thumbnail_path: Option<&str>,
    uploaded_by: &str,
) -> AppResult<IntakePhotoRow> {
    sqlx::query_as::<_, IntakePhotoRow>(
        r#"
        INSERT INTO intake_photos (job_card_id, photo_type, file_path, thumbnail_path, uploaded_by)
        VALUES ($1::uuid, $2, $3, $4, $5::uuid)
        RETURNING
            id::text AS id,
            photo_type,
            file_path,
            thumbnail_path,
            uploaded_by::text AS uploaded_by,
            created_at
        "#,
    )
    .bind(job_id)
    .bind(photo_type)
    .bind(file_path)
    .bind(thumbnail_path)
    .bind(uploaded_by)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn list_intake_photos(pool: &PgPool, job_id: &str) -> AppResult<Vec<IntakePhotoRow>> {
    sqlx::query_as::<_, IntakePhotoRow>(
        r#"
        SELECT
            id::text AS id,
            photo_type,
            file_path,
            thumbnail_path,
            uploaded_by::text AS uploaded_by,
            created_at
        FROM intake_photos
        WHERE job_card_id = $1::uuid
          AND file_deleted_at IS NULL
        ORDER BY created_at ASC
        "#,
    )
    .bind(job_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_intake_photo(pool: &PgPool, job_id: &str, photo_id: &str) -> AppResult<Option<IntakePhotoRow>> {
    sqlx::query_as::<_, IntakePhotoRow>(
        r#"
        SELECT
            id::text AS id,
            photo_type,
            file_path,
            thumbnail_path,
            uploaded_by::text AS uploaded_by,
            created_at
        FROM intake_photos
        WHERE job_card_id = $1::uuid
          AND id = $2::uuid
          AND file_deleted_at IS NULL
        "#,
    )
    .bind(job_id)
    .bind(photo_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn soft_delete_intake_photo(pool: &PgPool, job_id: &str, photo_id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE intake_photos
        SET file_deleted_at = NOW()
        WHERE job_card_id = $1::uuid
          AND id = $2::uuid
          AND file_deleted_at IS NULL
        "#,
    )
    .bind(job_id)
    .bind(photo_id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn upsert_customer_signature(
    pool: &PgPool,
    job_id: &str,
    signature_type: &str,
    file_path: &str,
    signed_by: &str,
) -> AppResult<CustomerSignatureRow> {
    sqlx::query_as::<_, CustomerSignatureRow>(
        r#"
        INSERT INTO customer_signatures (job_card_id, signature_type, file_path, signed_by, signed_at)
        VALUES ($1::uuid, $2, $3, $4, NOW())
        ON CONFLICT (job_card_id) DO UPDATE SET
            signature_type = EXCLUDED.signature_type,
            file_path = EXCLUDED.file_path,
            signed_by = EXCLUDED.signed_by,
            signed_at = NOW(),
            file_deleted_at = NULL
        RETURNING
            id::text AS id,
            signature_type,
            file_path,
            signed_by,
            signed_at,
            created_at
        "#,
    )
    .bind(job_id)
    .bind(signature_type)
    .bind(file_path)
    .bind(signed_by)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn get_customer_signature(pool: &PgPool, job_id: &str) -> AppResult<Option<CustomerSignatureRow>> {
    sqlx::query_as::<_, CustomerSignatureRow>(
        r#"
        SELECT
            id::text AS id,
            signature_type,
            file_path,
            signed_by,
            signed_at,
            created_at
        FROM customer_signatures
        WHERE job_card_id = $1::uuid
          AND file_deleted_at IS NULL
        "#,
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}
