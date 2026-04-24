use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::errors::AppResult;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub open_jobs: i64,
    pub jobs_change: i64,
    pub stock_alerts: i64,
    pub alerts_change: i64,
    pub bays_occupied: i64,
    pub total_bays: i64,
    pub goods_in_transit: i64,
    pub transit_change: i64,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecentActivity {
    pub id: String,
    pub action: String,
    pub description: String,
    pub performed_by: String,
    pub created_at: DateTime<Utc>,
}

pub async fn get_dashboard_stats(pool: &PgPool) -> AppResult<DashboardStats> {
    let stats = sqlx::query_as::<_, DashboardStats>(
        r#"
        SELECT
            COALESCE(SUM(CASE WHEN j.status IN ('IN_SERVICE', 'QA', 'BILLING') THEN 1 ELSE 0 END), 0) as open_jobs,
            0 as jobs_change,
            COALESCE(SUM(CASE WHEN i.quantity <= i.min_stock_level THEN 1 ELSE 0 END), 0) as stock_alerts,
            0 as alerts_change,
            (SELECT COUNT(*) FROM service_bays WHERE id IN (SELECT bay_id FROM job_cards WHERE status IN ('IN_SERVICE', 'QA', 'BILLING') AND bay_id IS NOT NULL)) as bays_occupied,
            (SELECT COUNT(*) FROM service_bays) as total_bays,
            0 as goods_in_transit,
            0 as transit_change
        FROM job_cards j
        CROSS JOIN inventory_items i
        LIMIT 1
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(stats)
}

pub async fn get_recent_activities(pool: &PgPool) -> AppResult<Vec<RecentActivity>> {
    let activities = sqlx::query_as::<_, RecentActivity>(
        r#"
        SELECT
            jca.id::text as id,
            jca.action,
            jca.description,
            u.name as performed_by,
            jca.created_at
        FROM job_card_activities jca
        JOIN users u ON jca.performed_by = u.id
        ORDER BY jca.created_at DESC
        LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(activities)
}
