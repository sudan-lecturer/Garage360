use sqlx::PgPool;

use crate::errors::{AppError, AppResult};

use super::types::{CreateDviTemplateRequest, DviResultRequest, DviResultRow, DviTemplateRow};

const DVI_TEMPLATE_SELECT: &str = r#"
SELECT
    id::text AS id,
    name,
    sections,
    created_at
FROM dvi_templates
"#;

const DVI_RESULT_SELECT: &str = r#"
SELECT
    r.id::text AS id,
    r.job_card_id::text AS job_card_id,
    r.template_id::text AS template_id,
    t.name AS template_name,
    r.data,
    r.submitted_by::text AS submitted_by,
    r.created_at
FROM dvi_results r
LEFT JOIN dvi_templates t ON t.id = r.template_id
"#;

pub async fn list_templates(pool: &PgPool) -> AppResult<Vec<DviTemplateRow>> {
    let query = format!(
        r#"{DVI_TEMPLATE_SELECT}
        WHERE is_active = true
        ORDER BY created_at DESC"#
    );

    sqlx::query_as::<_, DviTemplateRow>(&query)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn create_template(
    pool: &PgPool,
    req: &CreateDviTemplateRequest,
) -> AppResult<DviTemplateRow> {
    sqlx::query_as::<_, DviTemplateRow>(
        r#"
        INSERT INTO dvi_templates (name, sections)
        VALUES ($1, $2)
        RETURNING id::text AS id, name, sections, created_at
        "#,
    )
    .bind(&req.name)
    .bind(&req.sections)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn update_template(
    pool: &PgPool,
    id: &str,
    req: &CreateDviTemplateRequest,
) -> AppResult<Option<DviTemplateRow>> {
    sqlx::query_as::<_, DviTemplateRow>(
        r#"
        UPDATE dvi_templates
        SET name = $2,
            sections = $3
        WHERE id = $1::uuid
          AND is_active = true
        RETURNING id::text AS id, name, sections, created_at
        "#,
    )
    .bind(id)
    .bind(&req.name)
    .bind(&req.sections)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn soft_delete_template(pool: &PgPool, id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE dvi_templates
        SET is_active = false
        WHERE id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}

pub async fn template_exists(pool: &PgPool, id: &str) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM dvi_templates
            WHERE id = $1::uuid
              AND is_active = true
        )
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn job_card_exists(pool: &PgPool, id: &str) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM job_cards
            WHERE id = $1::uuid
              AND is_active = true
        )
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn create_result(
    pool: &PgPool,
    req: &DviResultRequest,
    submitted_by: &str,
) -> AppResult<String> {
    sqlx::query_scalar::<_, String>(
        r#"
        INSERT INTO dvi_results (job_card_id, template_id, data, submitted_by)
        VALUES ($1::uuid, $2::uuid, $3, $4::uuid)
        RETURNING id::text
        "#,
    )
    .bind(&req.job_card_id)
    .bind(req.template_id.as_deref())
    .bind(&req.data)
    .bind(submitted_by)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_result_by_id(pool: &PgPool, id: &str) -> AppResult<Option<DviResultRow>> {
    let query = format!(
        r#"{DVI_RESULT_SELECT}
        WHERE r.id = $1::uuid"#
    );

    sqlx::query_as::<_, DviResultRow>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
}

pub async fn update_result(
    pool: &PgPool,
    id: &str,
    req: &DviResultRequest,
    submitted_by: &str,
) -> AppResult<Option<String>> {
    sqlx::query_scalar::<_, String>(
        r#"
        UPDATE dvi_results
        SET job_card_id = $2::uuid,
            template_id = $3::uuid,
            data = $4,
            submitted_by = $5::uuid
        WHERE id = $1::uuid
        RETURNING id::text
        "#,
    )
    .bind(id)
    .bind(&req.job_card_id)
    .bind(req.template_id.as_deref())
    .bind(&req.data)
    .bind(submitted_by)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn delete_result(pool: &PgPool, id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        DELETE FROM dvi_results
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .map_err(AppError::Database)
}
