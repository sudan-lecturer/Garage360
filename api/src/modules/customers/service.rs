use serde_json::json;
use sqlx::PgPool;

use crate::errors::{AppError, AppResult};
use crate::common::pagination::PaginationMeta;

use super::{repo, types::{CreateCustomerRequest, CustomerResponse}};

pub async fn list_customers(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);

    let customers = repo::list(pool, &search, &like, limit, offset).await?;
    let total = repo::count(pool, &search, &like).await?;

    Ok(json!({
        "data": customers.into_iter().map(CustomerResponse::from).collect::<Vec<_>>(),
        "meta": crate::common::pagination::PaginationMeta { page, limit, total }
    }))
}

pub async fn search_customers(
    pool: &PgPool,
    search: String,
) -> AppResult<Vec<CustomerResponse>> {
    let like = format!("%{}%", search);
    let customers = repo::list(pool, &search, &like, 20, 0).await?;

    Ok(customers
        .into_iter()
        .map(CustomerResponse::from)
        .collect::<Vec<_>>())
}

pub async fn get_customer(pool: &PgPool, id: &str) -> AppResult<CustomerResponse> {
    repo::find_by_id(pool, id)
        .await?
        .map(CustomerResponse::from)
        .ok_or_else(|| AppError::NotFound("Customer not found".into()))
}

pub async fn create_customer(
    pool: &PgPool,
    req: &CreateCustomerRequest,
    created_by: &str,
) -> AppResult<CustomerResponse> {
    repo::create(pool, req, created_by)
        .await
        .map(CustomerResponse::from)
}

pub async fn update_customer(
    pool: &PgPool,
    id: &str,
    req: &CreateCustomerRequest,
) -> AppResult<CustomerResponse> {
    repo::update(pool, id, req)
        .await?
        .map(CustomerResponse::from)
        .ok_or_else(|| AppError::NotFound("Customer not found".into()))
}

pub async fn delete_customer(pool: &PgPool, id: &str) -> AppResult<serde_json::Value> {
    let rows_affected = repo::soft_delete(pool, id).await?;
    if rows_affected == 0 {
        return Err(AppError::NotFound("Customer not found".into()));
    }

    Ok(json!({ "deleted": true }))
}
