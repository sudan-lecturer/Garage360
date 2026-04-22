use sqlx::PgPool;

use crate::errors::{AppError, AppResult};

use super::types::{VehicleRequest, VehicleResponse};

pub async fn list(
    pool: &PgPool,
    search: &str,
    like: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<VehicleResponse>> {
    sqlx::query_as::<_, VehicleResponse>(
        r#"
        SELECT id, customer_id, registration_no, make, model, year
        FROM vehicles
        WHERE is_active = true
          AND (
            $1 = ''
            OR registration_no ILIKE $2
            OR make ILIKE $2
            OR model ILIKE $2
            OR COALESCE(vin, '') ILIKE $2
          )
        ORDER BY registration_no
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
        FROM vehicles
        WHERE is_active = true
          AND (
            $1 = ''
            OR registration_no ILIKE $2
            OR make ILIKE $2
            OR model ILIKE $2
            OR COALESCE(vin, '') ILIKE $2
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
        SELECT id, customer_id, registration_no, make, model, year
        FROM vehicles
        WHERE id = $1 AND is_active = true
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
) -> AppResult<VehicleResponse> {
    sqlx::query_as::<_, VehicleResponse>(
        r#"
        INSERT INTO vehicles (customer_id, registration_no, make, model, year, created_by)
        VALUES ($1::uuid, $2, $3, $4, $5, $6::uuid)
        RETURNING id, customer_id, registration_no, make, model, year
        "#,
    )
    .bind(&req.customer_id)
    .bind(&req.registration_no)
    .bind(&req.make)
    .bind(&req.model)
    .bind(req.year)
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
    sqlx::query_as::<_, VehicleResponse>(
        r#"
        UPDATE vehicles
        SET customer_id = $2::uuid,
            registration_no = $3,
            make = $4,
            model = $5,
            year = $6,
            updated_at = NOW()
        WHERE id = $1 AND is_active = true
        RETURNING id, customer_id, registration_no, make, model, year
        "#,
    )
    .bind(id)
    .bind(&req.customer_id)
    .bind(&req.registration_no)
    .bind(&req.make)
    .bind(&req.model)
    .bind(req.year)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn soft_delete(pool: &PgPool, id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE vehicles
        SET is_active = false, updated_at = NOW()
        WHERE id = $1 AND is_active = true
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}
