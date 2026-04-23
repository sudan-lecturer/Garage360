use serde_json::Value;
use sqlx::PgPool;

use crate::errors::{AppError, AppResult};

use super::types::{AssetDefectRow, AssetInspectionRow, AssetRow};

const ASSET_SELECT: &str = r#"
SELECT
    a.id::text AS id,
    a.asset_tag,
    a.name,
    a.category,
    a.location_id::text AS location_id,
    a.purchase_date,
    a.purchase_cost::text AS purchase_cost,
    a.useful_life_years,
    a.status,
    a.notes,
    a.created_at,
    a.updated_at,
    last_inspection.created_at AS last_inspection_at,
    COALESCE(open_defects.open_defect_count, 0)::bigint AS open_defect_count
FROM assets a
LEFT JOIN LATERAL (
    SELECT ai.created_at
    FROM asset_inspections ai
    WHERE ai.asset_id = a.id
    ORDER BY ai.created_at DESC, ai.id DESC
    LIMIT 1
) last_inspection ON true
LEFT JOIN LATERAL (
    SELECT COUNT(*) AS open_defect_count
    FROM asset_defects ad
    WHERE ad.asset_id = a.id
      AND ad.status <> 'RESOLVED'
) open_defects ON true
"#;

const INSPECTION_SELECT: &str = r#"
SELECT
    ai.id::text AS id,
    ai.asset_id::text AS asset_id,
    ai.template_id::text AS template_id,
    ait.name AS template_name,
    ai.data,
    ai.submitted_by::text AS submitted_by,
    ai.created_at
FROM asset_inspections ai
LEFT JOIN asset_inspection_templates ait ON ait.id = ai.template_id
"#;

const DEFECT_SELECT: &str = r#"
SELECT
    ad.id::text AS id,
    ad.asset_id::text AS asset_id,
    a.asset_tag,
    a.name AS asset_name,
    ad.description,
    ad.severity,
    ad.status,
    ad.reported_by::text AS reported_by,
    ad.resolved_by::text AS resolved_by,
    ad.resolved_at,
    ad.created_at
FROM asset_defects ad
JOIN assets a ON a.id = ad.asset_id
"#;

pub async fn list(
    pool: &PgPool,
    search: &str,
    like: &str,
    category: &str,
    category_like: &str,
    status: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<AssetRow>> {
    let query = format!(
        r#"{ASSET_SELECT}
        WHERE (
            $1 = ''
            OR a.asset_tag ILIKE $2
            OR a.name ILIKE $2
            OR COALESCE(a.category, '') ILIKE $2
            OR COALESCE(a.notes, '') ILIKE $2
            OR a.status ILIKE $2
        )
          AND ($3 = '' OR COALESCE(a.category, '') ILIKE $4)
          AND ($5 = '' OR a.status = $5)
        ORDER BY a.name, a.asset_tag
        LIMIT $6 OFFSET $7"#
    );

    sqlx::query_as::<_, AssetRow>(&query)
        .bind(search)
        .bind(like)
        .bind(category)
        .bind(category_like)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

pub async fn count(
    pool: &PgPool,
    search: &str,
    like: &str,
    category: &str,
    category_like: &str,
    status: &str,
) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM assets a
        WHERE (
            $1 = ''
            OR a.asset_tag ILIKE $2
            OR a.name ILIKE $2
            OR COALESCE(a.category, '') ILIKE $2
            OR COALESCE(a.notes, '') ILIKE $2
            OR a.status ILIKE $2
        )
          AND ($3 = '' OR COALESCE(a.category, '') ILIKE $4)
          AND ($5 = '' OR a.status = $5)
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(category)
    .bind(category_like)
    .bind(status)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_due_inspection(
    pool: &PgPool,
    search: &str,
    like: &str,
    category: &str,
    category_like: &str,
    status: &str,
    days_since_last: i32,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<AssetRow>> {
    let query = format!(
        r#"{ASSET_SELECT}
        WHERE (
            $1 = ''
            OR a.asset_tag ILIKE $2
            OR a.name ILIKE $2
            OR COALESCE(a.category, '') ILIKE $2
            OR COALESCE(a.notes, '') ILIKE $2
            OR a.status ILIKE $2
        )
          AND ($3 = '' OR COALESCE(a.category, '') ILIKE $4)
          AND ($5 = '' OR a.status = $5)
          AND (
              last_inspection.created_at IS NULL
              OR last_inspection.created_at <= NOW() - ($6::int * INTERVAL '1 day')
          )
        ORDER BY COALESCE(last_inspection.created_at, TIMESTAMPTZ '1970-01-01 00:00:00+00') ASC, a.name, a.asset_tag
        LIMIT $7 OFFSET $8"#
    );

    sqlx::query_as::<_, AssetRow>(&query)
        .bind(search)
        .bind(like)
        .bind(category)
        .bind(category_like)
        .bind(status)
        .bind(days_since_last)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

pub async fn count_due_inspection(
    pool: &PgPool,
    search: &str,
    like: &str,
    category: &str,
    category_like: &str,
    status: &str,
    days_since_last: i32,
) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM assets a
        LEFT JOIN LATERAL (
            SELECT ai.created_at
            FROM asset_inspections ai
            WHERE ai.asset_id = a.id
            ORDER BY ai.created_at DESC, ai.id DESC
            LIMIT 1
        ) last_inspection ON true
        WHERE (
            $1 = ''
            OR a.asset_tag ILIKE $2
            OR a.name ILIKE $2
            OR COALESCE(a.category, '') ILIKE $2
            OR COALESCE(a.notes, '') ILIKE $2
            OR a.status ILIKE $2
        )
          AND ($3 = '' OR COALESCE(a.category, '') ILIKE $4)
          AND ($5 = '' OR a.status = $5)
          AND (
              last_inspection.created_at IS NULL
              OR last_inspection.created_at <= NOW() - ($6::int * INTERVAL '1 day')
          )
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(category)
    .bind(category_like)
    .bind(status)
    .bind(days_since_last)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn find_asset(pool: &PgPool, id: &str) -> AppResult<Option<AssetRow>> {
    let query = format!(
        r#"{ASSET_SELECT}
        WHERE a.id = $1::uuid"#
    );

    sqlx::query_as::<_, AssetRow>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

pub async fn create_asset(
    pool: &PgPool,
    id: &str,
    asset_tag: &str,
    name: &str,
    category: Option<&str>,
    location_id: Option<&str>,
    purchase_date: Option<chrono::NaiveDate>,
    purchase_cost: Option<&str>,
    useful_life_years: Option<i32>,
    status: &str,
    notes: Option<&str>,
) -> AppResult<AssetRow> {
    sqlx::query_as::<_, AssetRow>(
        r#"
        INSERT INTO assets (
            id,
            asset_tag,
            name,
            category,
            location_id,
            purchase_date,
            purchase_cost,
            useful_life_years,
            status,
            notes
        )
        VALUES (
            $1::uuid,
            $2,
            $3,
            $4,
            $5::uuid,
            $6,
            $7::numeric(10,2),
            $8,
            $9,
            $10
        )
        RETURNING
            id::text AS id,
            asset_tag,
            name,
            category,
            location_id::text AS location_id,
            purchase_date,
            purchase_cost::text AS purchase_cost,
            useful_life_years,
            status,
            notes,
            created_at,
            updated_at,
            NULL::timestamptz AS last_inspection_at,
            0::bigint AS open_defect_count
        "#,
    )
    .bind(id)
    .bind(asset_tag)
    .bind(name)
    .bind(category)
    .bind(location_id)
    .bind(purchase_date)
    .bind(purchase_cost)
    .bind(useful_life_years)
    .bind(status)
    .bind(notes)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn update_asset(
    pool: &PgPool,
    id: &str,
    asset_tag: &str,
    name: &str,
    category: Option<&str>,
    location_id: Option<&str>,
    purchase_date: Option<chrono::NaiveDate>,
    purchase_cost: Option<&str>,
    useful_life_years: Option<i32>,
    status: &str,
    notes: Option<&str>,
) -> AppResult<Option<AssetRow>> {
    sqlx::query_as::<_, AssetRow>(
        r#"
        WITH updated AS (
            UPDATE assets
            SET asset_tag = $2,
                name = $3,
                category = $4,
                location_id = $5::uuid,
                purchase_date = $6,
                purchase_cost = $7::numeric(10,2),
                useful_life_years = $8,
                status = $9,
                notes = $10,
                updated_at = NOW()
            WHERE id = $1::uuid
            RETURNING *
        )
        SELECT
            updated.id::text AS id,
            updated.asset_tag,
            updated.name,
            updated.category,
            updated.location_id::text AS location_id,
            updated.purchase_date,
            updated.purchase_cost::text AS purchase_cost,
            updated.useful_life_years,
            updated.status,
            updated.notes,
            updated.created_at,
            updated.updated_at,
            last_inspection.created_at AS last_inspection_at,
            COALESCE(open_defects.open_defect_count, 0)::bigint AS open_defect_count
        FROM updated
        LEFT JOIN LATERAL (
            SELECT ai.created_at
            FROM asset_inspections ai
            WHERE ai.asset_id = updated.id
            ORDER BY ai.created_at DESC, ai.id DESC
            LIMIT 1
        ) last_inspection ON true
        LEFT JOIN LATERAL (
            SELECT COUNT(*) AS open_defect_count
            FROM asset_defects ad
            WHERE ad.asset_id = updated.id
              AND ad.status <> 'RESOLVED'
        ) open_defects ON true
        "#,
    )
    .bind(id)
    .bind(asset_tag)
    .bind(name)
    .bind(category)
    .bind(location_id)
    .bind(purchase_date)
    .bind(purchase_cost)
    .bind(useful_life_years)
    .bind(status)
    .bind(notes)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_asset_inspections(
    pool: &PgPool,
    asset_id: &str,
    limit: i64,
) -> AppResult<Vec<AssetInspectionRow>> {
    let query = format!(
        r#"{INSPECTION_SELECT}
        WHERE ai.asset_id = $1::uuid
        ORDER BY ai.created_at DESC, ai.id DESC
        LIMIT $2"#
    );

    sqlx::query_as::<_, AssetInspectionRow>(&query)
        .bind(asset_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

pub async fn active_template_exists(pool: &PgPool, id: &str) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM asset_inspection_templates
            WHERE id = $1::uuid
              AND is_active = true
        )
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn insert_inspection(
    pool: &PgPool,
    id: &str,
    asset_id: &str,
    template_id: Option<&str>,
    data: &Value,
    submitted_by: &str,
) -> AppResult<AssetInspectionRow> {
    sqlx::query_as::<_, AssetInspectionRow>(
        r#"
        WITH inserted AS (
            INSERT INTO asset_inspections (
                id,
                asset_id,
                template_id,
                data,
                submitted_by
            )
            VALUES (
                $1::uuid,
                $2::uuid,
                $3::uuid,
                $4::jsonb,
                $5::uuid
            )
            RETURNING *
        )
        SELECT
            inserted.id::text AS id,
            inserted.asset_id::text AS asset_id,
            inserted.template_id::text AS template_id,
            ait.name AS template_name,
            inserted.data,
            inserted.submitted_by::text AS submitted_by,
            inserted.created_at
        FROM inserted
        LEFT JOIN asset_inspection_templates ait ON ait.id = inserted.template_id
        "#,
    )
    .bind(id)
    .bind(asset_id)
    .bind(template_id)
    .bind(data)
    .bind(submitted_by)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_open_defects(
    pool: &PgPool,
    search: &str,
    like: &str,
    severity: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<AssetDefectRow>> {
    let query = format!(
        r#"{DEFECT_SELECT}
        WHERE ad.status <> 'RESOLVED'
          AND (
              $1 = ''
              OR a.asset_tag ILIKE $2
              OR a.name ILIKE $2
              OR ad.description ILIKE $2
              OR ad.severity ILIKE $2
              OR ad.status ILIKE $2
          )
          AND ($3 = '' OR ad.severity = $3)
        ORDER BY ad.created_at DESC, ad.id DESC
        LIMIT $4 OFFSET $5"#
    );

    sqlx::query_as::<_, AssetDefectRow>(&query)
        .bind(search)
        .bind(like)
        .bind(severity)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

pub async fn count_open_defects(
    pool: &PgPool,
    search: &str,
    like: &str,
    severity: &str,
) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM asset_defects ad
        JOIN assets a ON a.id = ad.asset_id
        WHERE ad.status <> 'RESOLVED'
          AND (
              $1 = ''
              OR a.asset_tag ILIKE $2
              OR a.name ILIKE $2
              OR ad.description ILIKE $2
              OR ad.severity ILIKE $2
              OR ad.status ILIKE $2
          )
          AND ($3 = '' OR ad.severity = $3)
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(severity)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_asset_open_defects(pool: &PgPool, asset_id: &str) -> AppResult<Vec<AssetDefectRow>> {
    let query = format!(
        r#"{DEFECT_SELECT}
        WHERE ad.asset_id = $1::uuid
          AND ad.status <> 'RESOLVED'
        ORDER BY ad.created_at DESC, ad.id DESC"#
    );

    sqlx::query_as::<_, AssetDefectRow>(&query)
        .bind(asset_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

pub async fn find_defect(pool: &PgPool, asset_id: &str, defect_id: &str) -> AppResult<Option<AssetDefectRow>> {
    let query = format!(
        r#"{DEFECT_SELECT}
        WHERE ad.asset_id = $1::uuid
          AND ad.id = $2::uuid"#
    );

    sqlx::query_as::<_, AssetDefectRow>(&query)
        .bind(asset_id)
        .bind(defect_id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

pub async fn insert_defect(
    pool: &PgPool,
    id: &str,
    asset_id: &str,
    description: &str,
    severity: &str,
    reported_by: &str,
) -> AppResult<AssetDefectRow> {
    sqlx::query_as::<_, AssetDefectRow>(
        r#"
        WITH inserted AS (
            INSERT INTO asset_defects (
                id,
                asset_id,
                description,
                severity,
                reported_by
            )
            VALUES (
                $1::uuid,
                $2::uuid,
                $3,
                $4,
                $5::uuid
            )
            RETURNING *
        )
        SELECT
            inserted.id::text AS id,
            inserted.asset_id::text AS asset_id,
            a.asset_tag,
            a.name AS asset_name,
            inserted.description,
            inserted.severity,
            inserted.status,
            inserted.reported_by::text AS reported_by,
            inserted.resolved_by::text AS resolved_by,
            inserted.resolved_at,
            inserted.created_at
        FROM inserted
        JOIN assets a ON a.id = inserted.asset_id
        "#,
    )
    .bind(id)
    .bind(asset_id)
    .bind(description)
    .bind(severity)
    .bind(reported_by)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn update_defect(
    pool: &PgPool,
    defect_id: &str,
    asset_id: &str,
    description: &str,
    severity: &str,
    status: &str,
    resolved_by: Option<&str>,
) -> AppResult<Option<AssetDefectRow>> {
    sqlx::query_as::<_, AssetDefectRow>(
        r#"
        WITH updated AS (
            UPDATE asset_defects
            SET description = $3,
                severity = $4,
                status = $5,
                resolved_by = CASE WHEN $5 = 'RESOLVED' THEN $6::uuid ELSE NULL END,
                resolved_at = CASE WHEN $5 = 'RESOLVED' THEN NOW() ELSE NULL END
            WHERE id = $1::uuid
              AND asset_id = $2::uuid
            RETURNING *
        )
        SELECT
            updated.id::text AS id,
            updated.asset_id::text AS asset_id,
            a.asset_tag,
            a.name AS asset_name,
            updated.description,
            updated.severity,
            updated.status,
            updated.reported_by::text AS reported_by,
            updated.resolved_by::text AS resolved_by,
            updated.resolved_at,
            updated.created_at
        FROM updated
        JOIN assets a ON a.id = updated.asset_id
        "#,
    )
    .bind(defect_id)
    .bind(asset_id)
    .bind(description)
    .bind(severity)
    .bind(status)
    .bind(resolved_by)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

fn map_db_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_error) = &error {
        match db_error.code().as_deref() {
            Some("23505") => {
                return AppError::Conflict("An asset with this asset tag already exists".into());
            }
            Some("23503") => {
                return AppError::Validation("Referenced asset record was not found".into());
            }
            Some("22P02") => {
                return AppError::Validation("Invalid asset identifier or JSON value".into());
            }
            Some("22007") => {
                return AppError::Validation("Invalid asset date value".into());
            }
            Some("22003") => {
                return AppError::Validation("Numeric value is out of range".into());
            }
            Some("23514") => {
                return AppError::Validation("Invalid asset defect status".into());
            }
            _ => {}
        }
    }

    AppError::Database(error)
}
