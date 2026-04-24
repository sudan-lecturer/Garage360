use sqlx::PgPool;

use crate::errors::{AppError, AppResult};

use super::types::{CreateCustomerRequest, CustomerRow};

pub async fn list(
    pool: &PgPool,
    search: &str,
    like: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<CustomerRow>> {
    sqlx::query_as::<_, CustomerRow>(
        r#"
        SELECT id::text AS id, customer_type, first_name, last_name, company_name, email, phone, address, created_at
        FROM customers
        WHERE is_active = true
          AND (
            $1 = ''
            OR first_name ILIKE $2
            OR last_name ILIKE $2
            OR company_name ILIKE $2
            OR phone ILIKE $2
          )
        ORDER BY created_at DESC
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
        FROM customers
        WHERE is_active = true
          AND (
            $1 = ''
            OR first_name ILIKE $2
            OR last_name ILIKE $2
            OR company_name ILIKE $2
            OR phone ILIKE $2
          )
        "#,
    )
    .bind(search)
    .bind(like)
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
