use axum::{
    extract::{Query, State, Path, Extension},
    routing::{get, post, put, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

use crate::errors::{AppError, AppResult};
use crate::middleware::auth::AuthUser;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/customers", get(list))
        .route("/customers", post(create))
        .route("/customers/:id", get(show))
        .route("/customers/:id", put(update))
        .route("/customers/:id", delete(delete))
        .route("/customers/search", get(search))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerResponse {
    pub id: String,
    pub customer_type: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: String,
    pub address: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct CustomerRow {
    id: String,
    customer_type: String,
    first_name: Option<String>,
    last_name: Option<String>,
    company_name: Option<String>,
    email: Option<String>,
    phone: String,
    address: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<CustomerRow> for CustomerResponse {
    fn from(row: CustomerRow) -> Self {
        Self {
            id: row.id,
            customer_type: row.customer_type,
            first_name: row.first_name,
            last_name: row.last_name,
            company_name: row.company_name,
            email: row.email,
            phone: row.phone,
            address: row.address,
            created_at: row.created_at.map(|d| d.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCustomerRequest {
    pub customer_type: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: String,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub page: i64,
    pub limit: i64,
    pub search: Option<String>,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self { page: 1, limit: 20, search: None }
    }
}

async fn list(
    Query(query): Query<ListQuery>,
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let offset = (query.page - 1) * query.limit;
    let search = query.search.unwrap_or_default();
    
    let customers = sqlx::query_as::<_, CustomerRow>(
        "SELECT id, customer_type, first_name, last_name, company_name, email, phone, address, created_at 
         FROM customers WHERE is_active = true AND (
             first_name ILIKE $1 OR last_name ILIKE $1 OR company_name ILIKE $1 OR phone LIKE $1
         ) ORDER BY created_at DESC LIMIT $2 OFFSET $3"
    )
    .bind(format!("%{}%", search))
    .bind(query.limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM customers WHERE is_active = true"
    )
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({
        "data": customers.into_iter().map(CustomerResponse::from).collect::<Vec<_>>(),
        "meta": { "page": query.page, "limit": query.limit, "total": total }
    })))
}

async fn search(
    Query(query): Query<ListQuery>,
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
) -> AppResult<Json<Vec<CustomerResponse>>> {
    let search = query.search.unwrap_or_default();
    
    let customers = sqlx::query_as::<_, CustomerRow>(
        "SELECT id, customer_type, first_name, last_name, company_name, email, phone, address, created_at 
         FROM customers WHERE is_active = true AND (
             first_name ILIKE $1 OR last_name ILIKE $1 OR company_name ILIKE $1 OR phone LIKE $1
         ) ORDER BY created_at DESC LIMIT 20"
    )
    .bind(format!("%{}%", search))
    .fetch_all(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(customers.into_iter().map(CustomerResponse::from).collect()))
}

async fn show(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
) -> AppResult<Json<CustomerResponse>> {
    let customer = sqlx::query_as::<_, CustomerRow>(
        "SELECT id, customer_type, first_name, last_name, company_name, email, phone, address, created_at 
         FROM customers WHERE id = $1 AND is_active = true"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Customer not found".into()))?;

    Ok(Json(CustomerResponse::from(customer)))
}

async fn create(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(req): Json<CreateCustomerRequest>,
) -> AppResult<Json<CustomerResponse>> {
    req.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let customer = sqlx::query_as::<_, CustomerRow>(
        "INSERT INTO customers (customer_type, first_name, last_name, company_name, email, phone, address, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id, customer_type, first_name, last_name, company_name, email, phone, address, created_at"
    )
    .bind(&req.customer_type)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&req.company_name)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.address)
    .bind(&auth.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(CustomerResponse::from(customer)))
}

async fn update(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<CreateCustomerRequest>,
) -> AppResult<Json<CustomerResponse>> {
    let customer = sqlx::query_as::<_, CustomerRow>(
        "UPDATE customers SET first_name = $2, last_name = $3, company_name = $4, email = $5, phone = $6, address = $7, updated_at = NOW()
         WHERE id = $1 AND is_active = true RETURNING id, customer_type, first_name, last_name, company_name, email, phone, address, created_at"
    )
    .bind(&id)
    .bind(&req.first_name)
    .bind(&req.last_name)
    .bind(&req.company_name)
    .bind(&req.email)
    .bind(&req.phone)
    .bind(&req.address)
    .fetch_optional(&state.db)
    .await
    .map_err(AppError::Database)?
    .ok_or_else(|| AppError::NotFound("Customer not found".into()))?;

    Ok(Json(CustomerResponse::from(customer)))
}

async fn delete(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query("UPDATE customers SET is_active = false WHERE id = $1")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Customer not found".into()));
    }

    Ok(Json(serde_json::json!({ "deleted": true })))
}