use sqlx::PgPool;

use crate::errors::{AppError, AppResult};

use super::types::{BayBoardBayRow, BayBoardJobRow, BayRow};

pub async fn list_settings(pool: &PgPool) -> AppResult<Vec<BayRow>> {
    sqlx::query_as::<_, BayRow>(
        r#"
        SELECT
            sb.id::text AS id,
            sb.code,
            sb.name,
            sb.capacity,
            sb.is_active,
            COALESCE((
                SELECT COUNT(*)
                FROM job_cards jc
                WHERE jc.bay_id = sb.id
                  AND jc.is_active = true
                  AND jc.status NOT IN ('COMPLETED', 'CANCELLED')
            ), 0)::bigint AS active_job_count,
            sb.created_at,
            sb.updated_at
        FROM service_bays sb
        ORDER BY sb.code, sb.name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_board_bays(pool: &PgPool) -> AppResult<Vec<BayBoardBayRow>> {
    sqlx::query_as::<_, BayBoardBayRow>(
        r#"
        SELECT id::text AS id, code, name, capacity
        FROM service_bays
        WHERE is_active = true
        ORDER BY code, name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_board_jobs(pool: &PgPool) -> AppResult<Vec<BayBoardJobRow>> {
    sqlx::query_as::<_, BayBoardJobRow>(
        r#"
        SELECT
            jc.bay_id::text AS bay_id,
            jc.id::text AS job_id,
            jc.job_no,
            jc.status::text AS job_status,
            v.id::text AS vehicle_id,
            v.registration_no,
            c.id::text AS customer_id,
            COALESCE(
                NULLIF(TRIM(CONCAT_WS(' ', c.first_name, c.last_name)), ''),
                c.company_name,
                c.phone
            ) AS customer_name
        FROM job_cards jc
        JOIN vehicles v ON v.id = jc.vehicle_id
        JOIN customers c ON c.id = jc.customer_id
        WHERE jc.is_active = true
          AND jc.bay_id IS NOT NULL
          AND jc.status NOT IN ('COMPLETED', 'CANCELLED')
        ORDER BY jc.updated_at DESC, jc.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

pub async fn create(
    pool: &PgPool,
    id: &str,
    code: &str,
    name: &str,
    capacity: i32,
) -> AppResult<BayRow> {
    sqlx::query_as::<_, BayRow>(
        r#"
        INSERT INTO service_bays (id, code, name, capacity)
        VALUES ($1::uuid, $2, $3, $4)
        RETURNING
            id::text AS id,
            code,
            name,
            capacity,
            is_active,
            0::bigint AS active_job_count,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(code)
    .bind(name)
    .bind(capacity)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn update(
    pool: &PgPool,
    id: &str,
    code: &str,
    name: &str,
    capacity: i32,
) -> AppResult<Option<BayRow>> {
    sqlx::query_as::<_, BayRow>(
        r#"
        WITH updated AS (
            UPDATE service_bays
            SET code = $2,
                name = $3,
                capacity = $4,
                updated_at = NOW()
            WHERE id = $1::uuid
            RETURNING id, code, name, capacity, is_active, created_at, updated_at
        )
        SELECT
            updated.id::text AS id,
            updated.code,
            updated.name,
            updated.capacity,
            updated.is_active,
            COALESCE((
                SELECT COUNT(*)
                FROM job_cards jc
                WHERE jc.bay_id = updated.id
                  AND jc.is_active = true
                  AND jc.status NOT IN ('COMPLETED', 'CANCELLED')
            ), 0)::bigint AS active_job_count,
            updated.created_at,
            updated.updated_at
        FROM updated
        "#,
    )
    .bind(id)
    .bind(code)
    .bind(name)
    .bind(capacity)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn set_status(pool: &PgPool, id: &str, is_active: bool) -> AppResult<Option<BayRow>> {
    sqlx::query_as::<_, BayRow>(
        r#"
        WITH updated AS (
            UPDATE service_bays
            SET is_active = $2,
                updated_at = NOW()
            WHERE id = $1::uuid
            RETURNING id, code, name, capacity, is_active, created_at, updated_at
        )
        SELECT
            updated.id::text AS id,
            updated.code,
            updated.name,
            updated.capacity,
            updated.is_active,
            COALESCE((
                SELECT COUNT(*)
                FROM job_cards jc
                WHERE jc.bay_id = updated.id
                  AND jc.is_active = true
                  AND jc.status NOT IN ('COMPLETED', 'CANCELLED')
            ), 0)::bigint AS active_job_count,
            updated.created_at,
            updated.updated_at
        FROM updated
        "#,
    )
    .bind(id)
    .bind(is_active)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn active_job_count(pool: &PgPool, id: &str) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM job_cards
        WHERE bay_id = $1::uuid
          AND is_active = true
          AND status NOT IN ('COMPLETED', 'CANCELLED')
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn soft_delete(pool: &PgPool, id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE service_bays
        SET is_active = false,
            updated_at = NOW()
        WHERE id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .map_err(map_db_error)
}

fn map_db_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_error) = &error {
        match db_error.code().as_deref() {
            Some("23505") => {
                return AppError::Conflict("A bay with this code already exists".into());
            }
            Some("22P02") => {
                return AppError::Validation("Invalid bay identifier".into());
            }
            Some("22003") => {
                return AppError::Validation("Capacity is out of range".into());
            }
            _ => {}
        }
    }

    AppError::Database(error)
}
