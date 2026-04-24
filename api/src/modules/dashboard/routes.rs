use axum::{routing::get, Json, Router};

use crate::errors::AppResult;
use crate::AppState;
use crate::modules::dashboard::service;
use crate::middleware::auth::AuthUser;
use crate::middleware::tenant::TenantDbPool;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard/stats", get(get_stats))
        .route("/dashboard/recent-activities", get(get_recent_activities))
}

async fn get_stats(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
) -> AppResult<Json<service::DashboardStats>> {
    Ok(Json(service::get_dashboard_stats(&tenant_db.pool).await?))
}

async fn get_recent_activities(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
) -> AppResult<Json<Vec<service::RecentActivity>>> {
    Ok(Json(service::get_recent_activities(&tenant_db.pool).await?))
}