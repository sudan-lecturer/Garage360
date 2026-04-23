use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use crate::errors::{AppError, AppResult};

use super::types::{
    GoodsReceiptItemCalcRow, GoodsReceiptItemRow, GoodsReceiptStateRow, GoodsReceiptSummaryRow,
    PurchaseApprovalRow, PurchaseOrderDetailRow, PurchaseOrderItemCalcRow, PurchaseOrderItemRow,
    PurchaseOrderStateRow, PurchaseOrderSummaryRow, PurchaseStatusHistoryRow, QaInspectionRow,
};

pub async fn list(
    pool: &PgPool,
    search: &str,
    like: &str,
    status: &str,
    in_transit_only: bool,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<PurchaseOrderSummaryRow>> {
    sqlx::query_as::<_, PurchaseOrderSummaryRow>(
        r#"
        SELECT
            po.id::text AS id,
            po.po_no,
            po.supplier_id::text AS supplier_id,
            s.name AS supplier_name,
            po.status::text AS status,
            po.expected_delivery,
            po.subtotal::text AS subtotal,
            po.discount_pct::text AS discount_pct,
            po.tax_amount::text AS tax_amount,
            po.total_amount::text AS total_amount,
            COALESCE(po.currency, 'NPR') AS currency,
            po.exchange_rate::text AS exchange_rate,
            po.created_at
        FROM purchase_orders po
        JOIN suppliers s ON s.id = po.supplier_id
        WHERE (
            $1 = ''
            OR s.name ILIKE $2
            OR po.status::text ILIKE $2
            OR CAST(po.po_no AS TEXT) ILIKE $2
            OR COALESCE(po.notes, '') ILIKE $2
        )
          AND ($3 = '' OR po.status::text = $3)
          AND ($4 = false OR po.status = 'SENT')
        ORDER BY po.created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(status)
    .bind(in_transit_only)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn count(
    pool: &PgPool,
    search: &str,
    like: &str,
    status: &str,
    in_transit_only: bool,
) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM purchase_orders po
        JOIN suppliers s ON s.id = po.supplier_id
        WHERE (
            $1 = ''
            OR s.name ILIKE $2
            OR po.status::text ILIKE $2
            OR CAST(po.po_no AS TEXT) ILIKE $2
            OR COALESCE(po.notes, '') ILIKE $2
        )
          AND ($3 = '' OR po.status::text = $3)
          AND ($4 = false OR po.status = 'SENT')
        "#,
    )
    .bind(search)
    .bind(like)
    .bind(status)
    .bind(in_transit_only)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_order(pool: &PgPool, id: &str) -> AppResult<Option<PurchaseOrderDetailRow>> {
    sqlx::query_as::<_, PurchaseOrderDetailRow>(
        r#"
        SELECT
            po.id::text AS id,
            po.po_no,
            po.supplier_id::text AS supplier_id,
            s.name AS supplier_name,
            po.status::text AS status,
            po.expected_delivery,
            po.subtotal::text AS subtotal,
            po.discount_pct::text AS discount_pct,
            po.tax_amount::text AS tax_amount,
            po.total_amount::text AS total_amount,
            po.notes,
            COALESCE(po.currency, 'NPR') AS currency,
            po.exchange_rate::text AS exchange_rate,
            po.created_by::text AS created_by,
            po.created_at
        FROM purchase_orders po
        JOIN suppliers s ON s.id = po.supplier_id
        WHERE po.id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_order_state(pool: &PgPool, id: &str) -> AppResult<Option<PurchaseOrderStateRow>> {
    sqlx::query_as::<_, PurchaseOrderStateRow>(
        r#"
        SELECT id::text AS id, status::text AS status
        FROM purchase_orders
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_order_items(pool: &PgPool, id: &str) -> AppResult<Vec<PurchaseOrderItemRow>> {
    sqlx::query_as::<_, PurchaseOrderItemRow>(
        r#"
        SELECT
            poi.id::text AS id,
            poi.purchase_order_id::text AS purchase_order_id,
            poi.inventory_item_id::text AS inventory_item_id,
            inv.name AS inventory_item_name,
            poi.description,
            poi.quantity::text AS quantity,
            poi.unit_price::text AS unit_price,
            poi.received_qty::text AS received_qty,
            poi.created_at
        FROM purchase_order_items poi
        LEFT JOIN inventory_items inv ON inv.id = poi.inventory_item_id
        WHERE poi.purchase_order_id = $1::uuid
        ORDER BY poi.created_at ASC
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_order_item_calcs(
    pool: &PgPool,
    id: &str,
) -> AppResult<Vec<PurchaseOrderItemCalcRow>> {
    sqlx::query_as::<_, PurchaseOrderItemCalcRow>(
        r#"
        SELECT
            id::text AS id,
            inventory_item_id::text AS inventory_item_id,
            description,
            quantity::float8 AS quantity,
            received_qty::float8 AS received_qty
        FROM purchase_order_items
        WHERE purchase_order_id = $1::uuid
        ORDER BY created_at ASC
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_order_history(
    pool: &PgPool,
    id: &str,
) -> AppResult<Vec<PurchaseStatusHistoryRow>> {
    sqlx::query_as::<_, PurchaseStatusHistoryRow>(
        r#"
        SELECT
            h.id::text AS id,
            h.from_status,
            h.to_status,
            h.notes,
            h.changed_by::text AS changed_by,
            u.name AS changed_by_name,
            h.created_at
        FROM po_status_history h
        LEFT JOIN users u ON u.id = h.changed_by
        WHERE h.purchase_order_id = $1::uuid
        ORDER BY h.created_at DESC
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_order_approvals(pool: &PgPool, id: &str) -> AppResult<Vec<PurchaseApprovalRow>> {
    sqlx::query_as::<_, PurchaseApprovalRow>(
        r#"
        SELECT
            a.id::text AS id,
            a.approved_by::text AS approved_by,
            u.name AS approved_by_name,
            a.is_approved,
            a.notes,
            a.created_at
        FROM po_approvals a
        LEFT JOIN users u ON u.id = a.approved_by
        WHERE a.purchase_order_id = $1::uuid
        ORDER BY a.created_at DESC
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_order_grns(pool: &PgPool, id: &str) -> AppResult<Vec<GoodsReceiptSummaryRow>> {
    sqlx::query_as::<_, GoodsReceiptSummaryRow>(
        r#"
        SELECT
            grn.id::text AS id,
            grn.grn_no,
            grn.purchase_order_id::text AS purchase_order_id,
            grn.status::text AS status,
            grn.received_by::text AS received_by,
            u.name AS received_by_name,
            grn.received_at,
            grn.created_at
        FROM goods_receipt_notes grn
        LEFT JOIN users u ON u.id = grn.received_by
        WHERE grn.purchase_order_id = $1::uuid
        ORDER BY grn.created_at DESC
        "#,
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_grn(
    pool: &PgPool,
    purchase_order_id: &str,
    grn_id: &str,
) -> AppResult<Option<GoodsReceiptSummaryRow>> {
    sqlx::query_as::<_, GoodsReceiptSummaryRow>(
        r#"
        SELECT
            grn.id::text AS id,
            grn.grn_no,
            grn.purchase_order_id::text AS purchase_order_id,
            grn.status::text AS status,
            grn.received_by::text AS received_by,
            u.name AS received_by_name,
            grn.received_at,
            grn.created_at
        FROM goods_receipt_notes grn
        LEFT JOIN users u ON u.id = grn.received_by
        WHERE grn.id = $1::uuid
          AND grn.purchase_order_id = $2::uuid
        "#,
    )
    .bind(grn_id)
    .bind(purchase_order_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_grn_state(
    pool: &PgPool,
    purchase_order_id: &str,
    grn_id: &str,
) -> AppResult<Option<GoodsReceiptStateRow>> {
    sqlx::query_as::<_, GoodsReceiptStateRow>(
        r#"
        SELECT
            id::text AS id,
            purchase_order_id::text AS purchase_order_id,
            status::text AS status
        FROM goods_receipt_notes
        WHERE id = $1::uuid
          AND purchase_order_id = $2::uuid
        "#,
    )
    .bind(grn_id)
    .bind(purchase_order_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_grn_items(pool: &PgPool, grn_id: &str) -> AppResult<Vec<GoodsReceiptItemRow>> {
    sqlx::query_as::<_, GoodsReceiptItemRow>(
        r#"
        SELECT
            gi.id::text AS id,
            gi.grn_id::text AS grn_id,
            gi.po_item_id::text AS po_item_id,
            poi.inventory_item_id::text AS inventory_item_id,
            inv.name AS inventory_item_name,
            poi.description,
            poi.quantity::text AS ordered_qty,
            gi.received_qty::text AS received_qty,
            gi.accepted_qty::text AS accepted_qty,
            gi.rejected_qty::text AS rejected_qty,
            gi.unit_cost::text AS unit_cost,
            gi.notes,
            gi.created_at
        FROM grn_items gi
        LEFT JOIN purchase_order_items poi ON poi.id = gi.po_item_id
        LEFT JOIN inventory_items inv ON inv.id = poi.inventory_item_id
        WHERE gi.grn_id = $1::uuid
        ORDER BY gi.created_at ASC
        "#,
    )
    .bind(grn_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_grn_item_calcs(
    pool: &PgPool,
    grn_id: &str,
) -> AppResult<Vec<GoodsReceiptItemCalcRow>> {
    sqlx::query_as::<_, GoodsReceiptItemCalcRow>(
        r#"
        SELECT
            gi.id::text AS id,
            gi.po_item_id::text AS po_item_id,
            poi.inventory_item_id::text AS inventory_item_id,
            gi.received_qty::float8 AS received_qty,
            gi.unit_cost::float8 AS unit_cost
        FROM grn_items gi
        LEFT JOIN purchase_order_items poi ON poi.id = gi.po_item_id
        WHERE gi.grn_id = $1::uuid
        ORDER BY gi.created_at ASC
        "#,
    )
    .bind(grn_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn find_grn_inspections(pool: &PgPool, grn_id: &str) -> AppResult<Vec<QaInspectionRow>> {
    sqlx::query_as::<_, QaInspectionRow>(
        r#"
        SELECT
            qa.id::text AS id,
            qa.grn_id::text AS grn_id,
            qa.inspected_by::text AS inspected_by,
            u.name AS inspected_by_name,
            qa.status,
            qa.notes,
            qa.created_at
        FROM qa_inspections qa
        LEFT JOIN users u ON u.id = qa.inspected_by
        WHERE qa.grn_id = $1::uuid
        ORDER BY qa.created_at DESC
        "#,
    )
    .bind(grn_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn supplier_exists(pool: &PgPool, id: &str) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM suppliers
            WHERE id = $1::uuid AND is_active = true
        )
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn inventory_item_exists(pool: &PgPool, id: &str) -> AppResult<bool> {
    sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM inventory_items
            WHERE id = $1::uuid AND is_active = true
        )
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}

pub async fn insert_order(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    supplier_id: &str,
    status: &str,
    expected_delivery: Option<DateTime<Utc>>,
    subtotal: f64,
    discount_pct: f64,
    tax_amount: f64,
    total_amount: f64,
    notes: Option<&str>,
    currency: &str,
    exchange_rate: Option<f64>,
    created_by: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO purchase_orders (
            id,
            supplier_id,
            status,
            expected_delivery,
            subtotal,
            discount_pct,
            tax_amount,
            total_amount,
            notes,
            currency,
            exchange_rate,
            created_by
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            $3::po_status,
            $4,
            $5::numeric,
            $6::numeric,
            $7::numeric,
            $8::numeric,
            $9,
            $10,
            $11::numeric,
            $12::uuid
        )
        "#,
    )
    .bind(id)
    .bind(supplier_id)
    .bind(status)
    .bind(expected_delivery)
    .bind(subtotal)
    .bind(discount_pct)
    .bind(tax_amount)
    .bind(total_amount)
    .bind(notes)
    .bind(currency)
    .bind(exchange_rate)
    .bind(created_by)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn update_order_header(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    supplier_id: &str,
    status: &str,
    expected_delivery: Option<DateTime<Utc>>,
    subtotal: f64,
    discount_pct: f64,
    tax_amount: f64,
    total_amount: f64,
    notes: Option<&str>,
    currency: &str,
    exchange_rate: Option<f64>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE purchase_orders
        SET supplier_id = $2::uuid,
            status = $3::po_status,
            expected_delivery = $4,
            subtotal = $5::numeric,
            discount_pct = $6::numeric,
            tax_amount = $7::numeric,
            total_amount = $8::numeric,
            notes = $9,
            currency = $10,
            exchange_rate = $11::numeric,
            updated_at = NOW()
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .bind(supplier_id)
    .bind(status)
    .bind(expected_delivery)
    .bind(subtotal)
    .bind(discount_pct)
    .bind(tax_amount)
    .bind(total_amount)
    .bind(notes)
    .bind(currency)
    .bind(exchange_rate)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn delete_order_items(tx: &mut Transaction<'_, Postgres>, id: &str) -> AppResult<()> {
    sqlx::query(
        r#"
        DELETE FROM purchase_order_items
        WHERE purchase_order_id = $1::uuid
        "#,
    )
    .bind(id)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn insert_order_item(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    purchase_order_id: &str,
    inventory_item_id: Option<&str>,
    description: &str,
    quantity: f64,
    unit_price: f64,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO purchase_order_items (
            id,
            purchase_order_id,
            inventory_item_id,
            description,
            quantity,
            unit_price,
            received_qty
        )
        VALUES (
            $1::uuid,
            $2::uuid,
            $3::uuid,
            $4,
            $5::numeric,
            $6::numeric,
            0
        )
        "#,
    )
    .bind(id)
    .bind(purchase_order_id)
    .bind(inventory_item_id)
    .bind(description)
    .bind(quantity)
    .bind(unit_price)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn insert_status_history(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    purchase_order_id: &str,
    from_status: Option<&str>,
    to_status: &str,
    notes: Option<&str>,
    changed_by: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO po_status_history (
            id,
            purchase_order_id,
            from_status,
            to_status,
            notes,
            changed_by
        )
        VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6::uuid)
        "#,
    )
    .bind(id)
    .bind(purchase_order_id)
    .bind(from_status)
    .bind(to_status)
    .bind(notes)
    .bind(changed_by)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn insert_approval(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    purchase_order_id: &str,
    approved_by: &str,
    is_approved: bool,
    notes: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO po_approvals (id, purchase_order_id, approved_by, is_approved, notes)
        VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5)
        "#,
    )
    .bind(id)
    .bind(purchase_order_id)
    .bind(approved_by)
    .bind(is_approved)
    .bind(notes)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn update_order_status(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    status: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE purchase_orders
        SET status = $2::po_status,
            updated_at = NOW()
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .bind(status)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn update_order_expected_delivery(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    expected_delivery: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE purchase_orders
        SET expected_delivery = $2,
            updated_at = NOW()
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .bind(expected_delivery)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn insert_grn(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    purchase_order_id: &str,
    received_by: &str,
    received_at: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO goods_receipt_notes (
            id,
            purchase_order_id,
            status,
            received_by,
            received_at
        )
        VALUES ($1::uuid, $2::uuid, 'PENDING'::grn_status, $3::uuid, $4)
        "#,
    )
    .bind(id)
    .bind(purchase_order_id)
    .bind(received_by)
    .bind(received_at)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn insert_grn_item(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    grn_id: &str,
    po_item_id: &str,
    received_qty: f64,
    unit_cost: Option<f64>,
    notes: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO grn_items (
            id,
            grn_id,
            po_item_id,
            received_qty,
            unit_cost,
            notes
        )
        VALUES ($1::uuid, $2::uuid, $3::uuid, $4::numeric, $5::numeric, $6)
        "#,
    )
    .bind(id)
    .bind(grn_id)
    .bind(po_item_id)
    .bind(received_qty)
    .bind(unit_cost)
    .bind(notes)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn add_received_qty_to_po_item(
    tx: &mut Transaction<'_, Postgres>,
    po_item_id: &str,
    received_qty: f64,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE purchase_order_items
        SET received_qty = received_qty + $2::numeric
        WHERE id = $1::uuid
        "#,
    )
    .bind(po_item_id)
    .bind(received_qty)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn update_grn_status(
    tx: &mut Transaction<'_, Postgres>,
    grn_id: &str,
    status: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE goods_receipt_notes
        SET status = $2::grn_status
        WHERE id = $1::uuid
        "#,
    )
    .bind(grn_id)
    .bind(status)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn update_grn_item_outcome(
    tx: &mut Transaction<'_, Postgres>,
    grn_item_id: &str,
    accepted_qty: f64,
    rejected_qty: f64,
    notes: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE grn_items
        SET accepted_qty = $2::numeric,
            rejected_qty = $3::numeric,
            notes = $4
        WHERE id = $1::uuid
        "#,
    )
    .bind(grn_item_id)
    .bind(accepted_qty)
    .bind(rejected_qty)
    .bind(notes)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn insert_qa_inspection(
    tx: &mut Transaction<'_, Postgres>,
    id: &str,
    grn_id: &str,
    inspected_by: &str,
    status: &str,
    notes: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO qa_inspections (id, grn_id, inspected_by, status, notes)
        VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5)
        "#,
    )
    .bind(id)
    .bind(grn_id)
    .bind(inspected_by)
    .bind(status)
    .bind(notes)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}

pub async fn update_inventory_cost(
    tx: &mut Transaction<'_, Postgres>,
    inventory_item_id: &str,
    cost_price: f64,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE inventory_items
        SET cost_price = $2::numeric,
            updated_at = NOW()
        WHERE id = $1::uuid
        "#,
    )
    .bind(inventory_item_id)
    .bind(cost_price)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(AppError::Database)
}
