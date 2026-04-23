use sqlx::PgPool;

use crate::errors::{AppError, AppResult};

use super::types::{FeatureFlagRow, IntakeTemplateRow, LocationRow, UserRow};

pub async fn get_setting(pool: &PgPool, key: &str) -> AppResult<Option<String>> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT value
        FROM tenant_settings
        WHERE key = $1
        "#,
    )
    .bind(key)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn upsert_setting(pool: &PgPool, key: &str, value: &str, is_encrypted: bool) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO tenant_settings (key, value, is_encrypted)
        VALUES ($1, $2, $3)
        ON CONFLICT (key) DO UPDATE
        SET value = EXCLUDED.value,
            is_encrypted = EXCLUDED.is_encrypted,
            updated_at = NOW()
        "#,
    )
    .bind(key)
    .bind(value)
    .bind(is_encrypted)
    .execute(pool)
    .await
    .map_err(map_db_error)?;

    Ok(())
}

pub async fn list_locations(pool: &PgPool) -> AppResult<Vec<LocationRow>> {
    sqlx::query_as::<_, LocationRow>(
        r#"
        SELECT
            id::text AS id,
            name,
            address,
            is_primary,
            is_active,
            created_at,
            updated_at
        FROM locations
        ORDER BY is_primary DESC, name, created_at
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

pub async fn find_location_by_id(pool: &PgPool, id: &str) -> AppResult<Option<LocationRow>> {
    sqlx::query_as::<_, LocationRow>(
        r#"
        SELECT
            id::text AS id,
            name,
            address,
            is_primary,
            is_active,
            created_at,
            updated_at
        FROM locations
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn count_active_primary_locations(pool: &PgPool, excluded_id: Option<&str>) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM locations
        WHERE is_active = true
          AND is_primary = true
          AND ($1::uuid IS NULL OR id <> $1::uuid)
        "#,
    )
    .bind(excluded_id)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn clear_primary_locations(pool: &PgPool, excluded_id: Option<&str>) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE locations
        SET is_primary = false,
            updated_at = NOW()
        WHERE is_primary = true
          AND ($1::uuid IS NULL OR id <> $1::uuid)
        "#,
    )
    .bind(excluded_id)
    .execute(pool)
    .await
    .map_err(map_db_error)?;

    Ok(())
}

pub async fn create_location(
    pool: &PgPool,
    id: &str,
    name: &str,
    address: Option<&str>,
    is_primary: bool,
    is_active: bool,
) -> AppResult<LocationRow> {
    sqlx::query_as::<_, LocationRow>(
        r#"
        INSERT INTO locations (id, name, address, is_primary, is_active)
        VALUES ($1::uuid, $2, $3, $4, $5)
        RETURNING
            id::text AS id,
            name,
            address,
            is_primary,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(address)
    .bind(is_primary)
    .bind(is_active)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn update_location(
    pool: &PgPool,
    id: &str,
    name: &str,
    address: Option<&str>,
    is_primary: bool,
    is_active: bool,
) -> AppResult<Option<LocationRow>> {
    sqlx::query_as::<_, LocationRow>(
        r#"
        UPDATE locations
        SET name = $2,
            address = $3,
            is_primary = $4,
            is_active = $5,
            updated_at = NOW()
        WHERE id = $1::uuid
        RETURNING
            id::text AS id,
            name,
            address,
            is_primary,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(address)
    .bind(is_primary)
    .bind(is_active)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_users(pool: &PgPool) -> AppResult<Vec<UserRow>> {
    sqlx::query_as::<_, UserRow>(
        r#"
        SELECT
            id::text AS id,
            email,
            name,
            role,
            is_active,
            last_login_at,
            created_at,
            updated_at
        FROM users
        ORDER BY is_active DESC, name, email
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

pub async fn find_user_by_id(pool: &PgPool, id: &str) -> AppResult<Option<UserRow>> {
    sqlx::query_as::<_, UserRow>(
        r#"
        SELECT
            id::text AS id,
            email,
            name,
            role,
            is_active,
            last_login_at,
            created_at,
            updated_at
        FROM users
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn count_active_owners(pool: &PgPool, excluded_id: Option<&str>) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM users
        WHERE is_active = true
          AND role = 'OWNER'
          AND ($1::uuid IS NULL OR id <> $1::uuid)
        "#,
    )
    .bind(excluded_id)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn create_user(
    pool: &PgPool,
    id: &str,
    email: &str,
    password_hash: &str,
    name: &str,
    role: &str,
    is_active: bool,
) -> AppResult<UserRow> {
    sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO users (id, email, password_hash, name, role, is_active)
        VALUES ($1::uuid, $2, $3, $4, $5, $6)
        RETURNING
            id::text AS id,
            email,
            name,
            role,
            is_active,
            last_login_at,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(name)
    .bind(role)
    .bind(is_active)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn update_user(
    pool: &PgPool,
    id: &str,
    email: &str,
    name: &str,
    role: &str,
    is_active: bool,
) -> AppResult<Option<UserRow>> {
    sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE users
        SET email = $2,
            name = $3,
            role = $4,
            is_active = $5,
            updated_at = NOW()
        WHERE id = $1::uuid
        RETURNING
            id::text AS id,
            email,
            name,
            role,
            is_active,
            last_login_at,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(name)
    .bind(role)
    .bind(is_active)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn update_user_with_password(
    pool: &PgPool,
    id: &str,
    email: &str,
    password_hash: &str,
    name: &str,
    role: &str,
    is_active: bool,
) -> AppResult<Option<UserRow>> {
    sqlx::query_as::<_, UserRow>(
        r#"
        UPDATE users
        SET email = $2,
            password_hash = $3,
            name = $4,
            role = $5,
            is_active = $6,
            updated_at = NOW()
        WHERE id = $1::uuid
        RETURNING
            id::text AS id,
            email,
            name,
            role,
            is_active,
            last_login_at,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(name)
    .bind(role)
    .bind(is_active)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn soft_delete_user(pool: &PgPool, id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE users
        SET is_active = false,
            updated_at = NOW()
        WHERE id = $1::uuid
          AND is_active = true
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .map_err(map_db_error)
}

pub async fn get_active_intake_template(pool: &PgPool) -> AppResult<Option<IntakeTemplateRow>> {
    sqlx::query_as::<_, IntakeTemplateRow>(
        r#"
        SELECT
            id::text AS id,
            name,
            items,
            is_active,
            created_at
        FROM intake_checklist_templates
        WHERE is_active = true
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn upsert_active_intake_template(
    pool: &PgPool,
    id: &str,
    name: &str,
    items: &serde_json::Value,
) -> AppResult<IntakeTemplateRow> {
    sqlx::query_as::<_, IntakeTemplateRow>(
        r#"
        WITH existing AS (
            SELECT id
            FROM intake_checklist_templates
            WHERE is_active = true
            ORDER BY created_at DESC, id DESC
            LIMIT 1
        ),
        reset AS (
            UPDATE intake_checklist_templates
            SET is_active = false
            WHERE is_active = true
              AND id <> COALESCE((SELECT id FROM existing), $1::uuid)
        ),
        updated AS (
            UPDATE intake_checklist_templates
            SET name = $2,
                items = $3::jsonb,
                is_active = true
            WHERE id = (SELECT id FROM existing)
            RETURNING
                id::text AS id,
                name,
                items,
                is_active,
                created_at
        ),
        inserted AS (
            INSERT INTO intake_checklist_templates (id, name, items, is_active)
            SELECT $1::uuid, $2, $3::jsonb, true
            WHERE NOT EXISTS (SELECT 1 FROM updated)
            RETURNING
                id::text AS id,
                name,
                items,
                is_active,
                created_at
        )
        SELECT * FROM updated
        UNION ALL
        SELECT * FROM inserted
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(items)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_feature_flags(control_pool: &PgPool, tenant_id: &str) -> AppResult<Vec<FeatureFlagRow>> {
    sqlx::query_as::<_, FeatureFlagRow>(
        r#"
        SELECT
            ff.key,
            ff.description,
            ff.default_enabled,
            COALESCE(tfo.is_enabled, ff.default_enabled, false) AS is_enabled,
            (tfo.id IS NOT NULL) AS has_override
        FROM feature_flags ff
        LEFT JOIN tenant_feature_flag_overrides tfo
            ON tfo.feature_flag_id = ff.id
           AND tfo.tenant_id = $1::uuid
        ORDER BY ff.key
        "#,
    )
    .bind(tenant_id)
    .fetch_all(control_pool)
    .await
    .map_err(map_db_error)
}

pub async fn upsert_feature_flag_override(
    control_pool: &PgPool,
    tenant_id: &str,
    key: &str,
    is_enabled: bool,
) -> AppResult<FeatureFlagRow> {
    sqlx::query_as::<_, FeatureFlagRow>(
        r#"
        WITH feature_flag AS (
            INSERT INTO feature_flags (id, key, description, default_enabled)
            VALUES (gen_random_uuid(), $2, NULL, false)
            ON CONFLICT (key) DO UPDATE
            SET key = EXCLUDED.key,
                updated_at = NOW()
            RETURNING id
        ),
        override_upsert AS (
            INSERT INTO tenant_feature_flag_overrides (tenant_id, feature_flag_id, is_enabled)
            SELECT $1::uuid, id, $3
            FROM feature_flag
            ON CONFLICT (tenant_id, feature_flag_id) DO UPDATE
            SET is_enabled = EXCLUDED.is_enabled,
                updated_at = NOW()
        )
        SELECT
            ff.key,
            ff.description,
            ff.default_enabled,
            COALESCE(tfo.is_enabled, ff.default_enabled, false) AS is_enabled,
            (tfo.id IS NOT NULL) AS has_override
        FROM feature_flags ff
        LEFT JOIN tenant_feature_flag_overrides tfo
            ON tfo.feature_flag_id = ff.id
           AND tfo.tenant_id = $1::uuid
        WHERE ff.key = $2
        "#,
    )
    .bind(tenant_id)
    .bind(key)
    .bind(is_enabled)
    .fetch_one(control_pool)
    .await
    .map_err(map_db_error)
}

fn map_db_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_error) = &error {
        match db_error.code().as_deref() {
            Some("23505") => {
                let constraint = db_error.constraint().unwrap_or_default();
                if constraint.contains("users_email_key") {
                    return AppError::Conflict("A user with this email already exists".into());
                }

                return AppError::Conflict("A record with this value already exists".into());
            }
            Some("23514") => {
                return AppError::Validation("A provided value does not satisfy a database rule".into());
            }
            Some("22P02") => {
                return AppError::Validation("Invalid identifier".into());
            }
            _ => {}
        }
    }

    AppError::Database(error)
}
