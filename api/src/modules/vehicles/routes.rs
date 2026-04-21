use axum::{
    extract::{Query, State, Path, Extension},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::errors::AppResult;
use crate::middleware::auth::AuthUser;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/vehicles", get(list))
        .route("/vehicles/search", get(search))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleResponse {
    pub id: String,
    pub customer_id: String,
    pub registration_no: String,
    pub make: String,
    pub model: String,
    pub year: Option<i32>,
}

async fn list(
    Query(query): Query<ListQuery>,
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let search = query.search.unwrap_or_default();
    let offset = (query.page - 1) * query.limit;
    
    let vehicles = sqlx::query_as::<_, (String, String, String, String, String, Option<i32>)>(
        "SELECT id, customer_id, registration_no, make, model, year FROM vehicles 
         WHERE is_active = true AND (registration_no LIKE $1 OR make ILIKE $1 OR model ILIKE $1)
         ORDER BY registration_no LIMIT $2 OFFSET $3"
    )
    .bind(format!("%{}%", search))
    .bind(query.limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(crate::errors::AppError::Database)?;

    let data: Vec<VehicleResponse> = vehicles.into_iter().map(|v| VehicleResponse {
        id: v.0, customer_id: v.1, registration_no: v.2, make: v.3, model: v.4, year: v.5,
    }).collect();

    Ok(Json(serde_json::json!({ "data": data, "meta": { "page": query.page, "limit": query.limit } })))
}

async fn search(
    Query(query): Query<ListQuery>,
    State(state): State<AppState>,
    Extension(_auth): Extension<AuthUser>,
) -> AppResult<Json<Vec<VehicleResponse>>> {
    let search = query.search.unwrap_or_default();
    
    let vehicles = sqlx::query_as::<_, (String, String, String, String, String, Option<i32>)>(
        "SELECT id, customer_id, registration_no, make, model, year FROM vehicles 
         WHERE is_active = true AND (registration_no LIKE $1 OR vin LIKE $1)
         ORDER BY registration_no LIMIT 20"
    )
    .bind(format!("%{}%", search))
    .fetch_all(&state.db)
    .await
    .map_err(crate::errors::AppError::Database)?;

    let data: Vec<VehicleResponse> = vehicles.into_iter().map(|v| VehicleResponse {
        id: v.0, customer_id: v.1, registration_no: v.2, make: v.3, model: v.4, year: v.5,
    }).collect();

    Ok(Json(data))
}

#[derive(Default, Deserialize)]
pub struct ListQuery {
    pub page: i64,
    pub limit: i64,
    pub search: Option<String>,
}