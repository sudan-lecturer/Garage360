use sqlx::PgPool;

use crate::errors::{AppError, AppResult};

use super::types::{VehicleRequest, VehicleResponse};

pub async fn ensure_vehicle_support_tables(pool: &PgPool) -> AppResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vehicle_photos (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            vehicle_id UUID NOT NULL REFERENCES vehicles(id) ON DELETE CASCADE,
            file_path TEXT NOT NULL,
            file_deleted_at TIMESTAMPTZ,
            created_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_vehicle_photos_vehicle ON vehicle_photos(vehicle_id)")
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_vehicle_photos_active ON vehicle_photos(file_deleted_at)")
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

    Ok(())
}

pub async fn list(
    pool: &PgPool,
    search: &str,
    like: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<VehicleResponse>> {
    sqlx::query_as::<_, VehicleResponse>(
        r#"
        SELECT
            v.id::text AS id,
            v.customer_id::text AS customer_id,
            COALESCE(NULLIF(TRIM(CONCAT_WS(' ', c.first_name, c.last_name)), ''), c.company_name, c.phone) AS customer_name,
            v.registration_no,
            v.make,
            v.model,
            v.year,
            (
                SELECT MAX(j.created_at)::text
                FROM job_cards j
                WHERE j.vehicle_id = v.id
            ) AS last_service_at,
            (
                SELECT vp.file_path
                FROM vehicle_photos vp
                WHERE vp.vehicle_id = v.id
                  AND vp.file_deleted_at IS NULL
                ORDER BY vp.created_at DESC
                LIMIT 1
            ) AS photo_path
        FROM vehicles v
        JOIN customers c ON c.id = v.customer_id
        WHERE v.is_active = true
          AND (
            $1 = ''
            OR v.registration_no ILIKE $2
            OR v.make ILIKE $2
            OR v.model ILIKE $2
            OR COALESCE(v.vin, '') ILIKE $2
            OR COALESCE(c.first_name, '') ILIKE $2
            OR COALESCE(c.last_name, '') ILIKE $2
            OR TRIM(CONCAT_WS(' ', COALESCE(c.first_name, ''), COALESCE(c.last_name, ''))) ILIKE $2
            OR COALESCE(c.company_name, '') ILIKE $2
            OR COALESCE(c.phone, '') ILIKE $2
            OR COALESCE(c.email, '') ILIKE $2
          )
        ORDER BY v.registration_no
        LIMIT $3 OFFSET $4
        "#,
    )
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
        FROM vehicles v
        JOIN customers c ON c.id = v.customer_id
        WHERE v.is_active = true
          AND (
            $1 = ''
            OR v.registration_no ILIKE $2
            OR v.make ILIKE $2
            OR v.model ILIKE $2
            OR COALESCE(v.vin, '') ILIKE $2
            OR COALESCE(c.first_name, '') ILIKE $2
            OR COALESCE(c.last_name, '') ILIKE $2
            OR TRIM(CONCAT_WS(' ', COALESCE(c.first_name, ''), COALESCE(c.last_name, ''))) ILIKE $2
            OR COALESCE(c.company_name, '') ILIKE $2
            OR COALESCE(c.phone, '') ILIKE $2
            OR COALESCE(c.email, '') ILIKE $2
          )
        "#,
    )
    .bind(search)
    .bind(like)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_by_id(pool: &PgPool, id: &str) -> AppResult<Option<VehicleResponse>> {
    sqlx::query_as::<_, VehicleResponse>(
        r#"
        SELECT
            v.id::text AS id,
            v.customer_id::text AS customer_id,
            COALESCE(NULLIF(TRIM(CONCAT_WS(' ', c.first_name, c.last_name)), ''), c.company_name, c.phone) AS customer_name,
            v.registration_no,
            v.make,
            v.model,
            v.year,
            (
                SELECT MAX(j.created_at)::text
                FROM job_cards j
                WHERE j.vehicle_id = v.id
            ) AS last_service_at,
            (
                SELECT vp.file_path
                FROM vehicle_photos vp
                WHERE vp.vehicle_id = v.id
                  AND vp.file_deleted_at IS NULL
                ORDER BY vp.created_at DESC
                LIMIT 1
            ) AS photo_path
        FROM vehicles v
        JOIN customers c ON c.id = v.customer_id
        WHERE v.id = $1::uuid AND v.is_active = true
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn create(
    pool: &PgPool,
    req: &VehicleRequest,
    created_by: &str,
) -> AppResult<String> {
    sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO vehicles (customer_id, registration_no, make, model, year, color, vin, created_by)
        VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8::uuid)
        RETURNING id::text
        "#,
    )
    .bind(&req.customer_id)
    .bind(&req.registration_no)
    .bind(&req.make)
    .bind(&req.model)
    .bind(req.year)
    .bind(&req.color)
    .bind(&req.vin)
    .bind(created_by)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update(
    pool: &PgPool,
    id: &str,
    req: &VehicleRequest,
) -> AppResult<Option<VehicleResponse>> {
    let vehicle_id = sqlx::query_scalar::<_, String>(
        r#"
        UPDATE vehicles
        SET customer_id = $2::uuid,
            registration_no = $3,
            make = $4,
            model = $5,
            year = $6,
            color = $7,
            vin = $8,
            updated_at = NOW()
        WHERE id = $1::uuid AND is_active = true
        RETURNING id::text
        "#,
    )
    .bind(id)
    .bind(&req.customer_id)
    .bind(&req.registration_no)
    .bind(&req.make)
    .bind(&req.model)
    .bind(req.year)
    .bind(&req.color)
    .bind(&req.vin)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    if let Some(vehicle_id) = vehicle_id {
        find_by_id(pool, &vehicle_id).await
    } else {
        Ok(None)
    }
}

pub async fn soft_delete(pool: &PgPool, id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE vehicles
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

pub async fn upsert_vehicle_photo(
    pool: &PgPool,
    vehicle_id: &str,
    file_path: &str,
    created_by: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE vehicle_photos
        SET file_deleted_at = NOW()
        WHERE vehicle_id = $1::uuid
          AND file_deleted_at IS NULL
        "#,
    )
    .bind(vehicle_id)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    sqlx::query(
        r#"
        INSERT INTO vehicle_photos (vehicle_id, file_path, created_by)
        VALUES ($1::uuid, $2, $3::uuid)
        "#,
    )
    .bind(vehicle_id)
    .bind(file_path)
    .bind(created_by)
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(())
}
