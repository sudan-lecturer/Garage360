use sqlx::PgPool;

use crate::errors::{AppError, AppResult};

use super::types::{InventoryItemRow, StockAdjustmentRow};

pub async fn list(
    pool: &PgPool,
    search: &str,
    like: &str,
    category: &str,
    category_like: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<InventoryItemRow>> {
    sqlx::query_as::<_, InventoryItemRow>(
        r#"
        SELECT
            i.id::text AS id,
            i.sku,
            i.name,
            i.description,
            i.category,
            i.unit,
            i.cost_price::text AS cost_price,
            i.sell_price::text AS sell_price,
            i.min_stock_level,
            COALESCE(stock.new_quantity, 0)::text AS current_quantity,
            i.is_active,
            i.created_at,
            i.updated_at
        FROM inventory_items i
        LEFT JOIN LATERAL (
            SELECT sa.new_quantity
            FROM stock_adjustments sa
            WHERE sa.inventory_item_id = i.id
            ORDER BY sa.created_at DESC, sa.id DESC
            LIMIT 1
        ) stock ON true
        WHERE i.is_active = true
          AND (
            $1 = ''
            OR i.sku ILIKE $2
            OR i.name ILIKE $2
            OR COALESCE(i.description, '') ILIKE $2
            OR COALESCE(i.category, '') ILIKE $2
          )
          AND (
            $3 = ''
            OR COALESCE(i.category, '') ILIKE $4
          )
        ORDER BY i.name, i.sku
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(category)
    .bind(category_like)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

pub async fn count(pool: &PgPool, search: &str, like: &str, category: &str, category_like: &str) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM inventory_items i
        WHERE i.is_active = true
          AND (
            $1 = ''
            OR i.sku ILIKE $2
            OR i.name ILIKE $2
            OR COALESCE(i.description, '') ILIKE $2
            OR COALESCE(i.category, '') ILIKE $2
          )
          AND (
            $3 = ''
            OR COALESCE(i.category, '') ILIKE $4
          )
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(category)
    .bind(category_like)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_low_stock(
    pool: &PgPool,
    search: &str,
    like: &str,
    category: &str,
    category_like: &str,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<InventoryItemRow>> {
    sqlx::query_as::<_, InventoryItemRow>(
        r#"
        SELECT
            i.id::text AS id,
            i.sku,
            i.name,
            i.description,
            i.category,
            i.unit,
            i.cost_price::text AS cost_price,
            i.sell_price::text AS sell_price,
            i.min_stock_level,
            COALESCE(stock.new_quantity, 0)::text AS current_quantity,
            i.is_active,
            i.created_at,
            i.updated_at
        FROM inventory_items i
        LEFT JOIN LATERAL (
            SELECT sa.new_quantity
            FROM stock_adjustments sa
            WHERE sa.inventory_item_id = i.id
            ORDER BY sa.created_at DESC, sa.id DESC
            LIMIT 1
        ) stock ON true
        WHERE i.is_active = true
          AND COALESCE(stock.new_quantity, 0) <= i.min_stock_level
          AND (
            $1 = ''
            OR i.sku ILIKE $2
            OR i.name ILIKE $2
            OR COALESCE(i.description, '') ILIKE $2
            OR COALESCE(i.category, '') ILIKE $2
          )
          AND (
            $3 = ''
            OR COALESCE(i.category, '') ILIKE $4
          )
        ORDER BY COALESCE(stock.new_quantity, 0), i.name, i.sku
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(category)
    .bind(category_like)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

pub async fn count_low_stock(
    pool: &PgPool,
    search: &str,
    like: &str,
    category: &str,
    category_like: &str,
) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM inventory_items i
        LEFT JOIN LATERAL (
            SELECT sa.new_quantity
            FROM stock_adjustments sa
            WHERE sa.inventory_item_id = i.id
            ORDER BY sa.created_at DESC, sa.id DESC
            LIMIT 1
        ) stock ON true
        WHERE i.is_active = true
          AND COALESCE(stock.new_quantity, 0) <= i.min_stock_level
          AND (
            $1 = ''
            OR i.sku ILIKE $2
            OR i.name ILIKE $2
            OR COALESCE(i.description, '') ILIKE $2
            OR COALESCE(i.category, '') ILIKE $2
          )
          AND (
            $3 = ''
            OR COALESCE(i.category, '') ILIKE $4
          )
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(category)
    .bind(category_like)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn find_by_id(pool: &PgPool, id: &str) -> AppResult<Option<InventoryItemRow>> {
    sqlx::query_as::<_, InventoryItemRow>(
        r#"
        SELECT
            i.id::text AS id,
            i.sku,
            i.name,
            i.description,
            i.category,
            i.unit,
            i.cost_price::text AS cost_price,
            i.sell_price::text AS sell_price,
            i.min_stock_level,
            COALESCE(stock.new_quantity, 0)::text AS current_quantity,
            i.is_active,
            i.created_at,
            i.updated_at
        FROM inventory_items i
        LEFT JOIN LATERAL (
            SELECT sa.new_quantity
            FROM stock_adjustments sa
            WHERE sa.inventory_item_id = i.id
            ORDER BY sa.created_at DESC, sa.id DESC
            LIMIT 1
        ) stock ON true
        WHERE i.id = $1::uuid
          AND i.is_active = true
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn list_adjustments(pool: &PgPool, inventory_item_id: &str) -> AppResult<Vec<StockAdjustmentRow>> {
    sqlx::query_as::<_, StockAdjustmentRow>(
        r#"
        SELECT
            id::text AS id,
            inventory_item_id::text AS inventory_item_id,
            adjustment_type,
            quantity::text AS quantity,
            previous_quantity::text AS previous_quantity,
            new_quantity::text AS new_quantity,
            reason,
            performed_by::text AS performed_by,
            created_at
        FROM stock_adjustments
        WHERE inventory_item_id = $1::uuid
        ORDER BY created_at DESC, id DESC
        LIMIT 100
        "#,
    )
    .bind(inventory_item_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

pub async fn create(
    pool: &PgPool,
    id: &str,
    sku: &str,
    name: &str,
    description: Option<&str>,
    category: Option<&str>,
    unit: &str,
    cost_price: &str,
    sell_price: &str,
    min_stock_level: i32,
    created_by: &str,
) -> AppResult<InventoryItemRow> {
    sqlx::query_as::<_, InventoryItemRow>(
        r#"
        INSERT INTO inventory_items (
            id,
            sku,
            name,
            description,
            category,
            unit,
            cost_price,
            sell_price,
            min_stock_level,
            created_by
        )
        VALUES (
            $1::uuid,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7::numeric(10,2),
            $8::numeric(10,2),
            $9,
            $10::uuid
        )
        RETURNING
            id::text AS id,
            sku,
            name,
            description,
            category,
            unit,
            cost_price::text AS cost_price,
            sell_price::text AS sell_price,
            min_stock_level,
            '0'::text AS current_quantity,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(id)
    .bind(sku)
    .bind(name)
    .bind(description)
    .bind(category)
    .bind(unit)
    .bind(cost_price)
    .bind(sell_price)
    .bind(min_stock_level)
    .bind(created_by)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

pub async fn update(
    pool: &PgPool,
    id: &str,
    sku: &str,
    name: &str,
    description: Option<&str>,
    category: Option<&str>,
    unit: &str,
    cost_price: &str,
    sell_price: &str,
    min_stock_level: i32,
) -> AppResult<Option<InventoryItemRow>> {
    sqlx::query_as::<_, InventoryItemRow>(
        r#"
        WITH updated AS (
            UPDATE inventory_items
            SET sku = $2,
                name = $3,
                description = $4,
                category = $5,
                unit = $6,
                cost_price = $7::numeric(10,2),
                sell_price = $8::numeric(10,2),
                min_stock_level = $9,
                updated_at = NOW()
            WHERE id = $1::uuid
              AND is_active = true
            RETURNING id, sku, name, description, category, unit, cost_price, sell_price, min_stock_level, is_active, created_at, updated_at
        )
        SELECT
            updated.id::text AS id,
            updated.sku,
            updated.name,
            updated.description,
            updated.category,
            updated.unit,
            updated.cost_price::text AS cost_price,
            updated.sell_price::text AS sell_price,
            updated.min_stock_level,
            COALESCE(stock.new_quantity, 0)::text AS current_quantity,
            updated.is_active,
            updated.created_at,
            updated.updated_at
        FROM updated
        LEFT JOIN LATERAL (
            SELECT sa.new_quantity
            FROM stock_adjustments sa
            WHERE sa.inventory_item_id = updated.id
            ORDER BY sa.created_at DESC, sa.id DESC
            LIMIT 1
        ) stock ON true
        "#,
    )
    .bind(id)
    .bind(sku)
    .bind(name)
    .bind(description)
    .bind(category)
    .bind(unit)
    .bind(cost_price)
    .bind(sell_price)
    .bind(min_stock_level)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

pub async fn soft_delete(pool: &PgPool, id: &str) -> AppResult<u64> {
    sqlx::query(
        r#"
        UPDATE inventory_items
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

pub async fn adjust_stock(
    pool: &PgPool,
    adjustment_id: &str,
    inventory_item_id: &str,
    adjustment_type: &str,
    quantity: &str,
    reason: Option<&str>,
    performed_by: &str,
) -> AppResult<(InventoryItemRow, StockAdjustmentRow)> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let locked_item = sqlx::query_scalar::<_, String>(
        r#"
        SELECT id::text AS id
        FROM inventory_items
        WHERE id = $1::uuid
          AND is_active = true
        FOR UPDATE
        "#,
    )
    .bind(inventory_item_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    if locked_item.is_none() {
        return Err(AppError::NotFound("Inventory item not found".into()));
    }

    let adjustment = sqlx::query_as::<_, StockAdjustmentRow>(
        r#"
        WITH current_stock AS (
            SELECT COALESCE((
                SELECT sa.new_quantity
                FROM stock_adjustments sa
                WHERE sa.inventory_item_id = $2::uuid
                ORDER BY sa.created_at DESC, sa.id DESC
                LIMIT 1
            ), 0) AS quantity
        )
        INSERT INTO stock_adjustments (
            id,
            inventory_item_id,
            adjustment_type,
            quantity,
            previous_quantity,
            new_quantity,
            reason,
            performed_by
        )
        SELECT
            $1::uuid,
            $2::uuid,
            $3,
            $4::numeric(10,3),
            current_stock.quantity,
            CASE
                WHEN $3 = 'ADD' THEN current_stock.quantity + $4::numeric(10,3)
                WHEN $3 = 'REMOVE' THEN current_stock.quantity - $4::numeric(10,3)
                ELSE $4::numeric(10,3)
            END,
            $5,
            $6::uuid
        FROM current_stock
        RETURNING
            id::text AS id,
            inventory_item_id::text AS inventory_item_id,
            adjustment_type,
            quantity::text AS quantity,
            previous_quantity::text AS previous_quantity,
            new_quantity::text AS new_quantity,
            reason,
            performed_by::text AS performed_by,
            created_at
        "#,
    )
    .bind(adjustment_id)
    .bind(inventory_item_id)
    .bind(adjustment_type)
    .bind(quantity)
    .bind(reason)
    .bind(performed_by)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    if adjustment
        .new_quantity
        .as_deref()
        .map_or(false, |value| value.starts_with('-'))
    {
        return Err(AppError::Conflict(
            "Adjustment would result in negative stock".into(),
        ));
    }

    let item = sqlx::query_as::<_, InventoryItemRow>(
        r#"
        SELECT
            i.id::text AS id,
            i.sku,
            i.name,
            i.description,
            i.category,
            i.unit,
            i.cost_price::text AS cost_price,
            i.sell_price::text AS sell_price,
            i.min_stock_level,
            COALESCE(stock.new_quantity, 0)::text AS current_quantity,
            i.is_active,
            i.created_at,
            i.updated_at
        FROM inventory_items i
        LEFT JOIN LATERAL (
            SELECT sa.new_quantity
            FROM stock_adjustments sa
            WHERE sa.inventory_item_id = i.id
            ORDER BY sa.created_at DESC, sa.id DESC
            LIMIT 1
        ) stock ON true
        WHERE i.id = $1::uuid
          AND i.is_active = true
        "#,
    )
    .bind(inventory_item_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;

    Ok((item, adjustment))
}

fn map_db_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_error) = &error {
        match db_error.code().as_deref() {
            Some("23505") => {
                return AppError::Conflict("An inventory item with this SKU already exists".into());
            }
            Some("23503") => {
                return AppError::Validation("Referenced record was not found".into());
            }
            Some("22P02") => {
                return AppError::Validation("Invalid inventory identifier or numeric value".into());
            }
            Some("22003") => {
                return AppError::Validation("Numeric value is out of range".into());
            }
            Some("23514") => {
                return AppError::Validation("Invalid inventory adjustment type".into());
            }
            _ => {}
        }
    }

    AppError::Database(error)
}
