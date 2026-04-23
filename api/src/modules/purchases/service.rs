use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::common::pagination::PaginationMeta;

use super::{
    repo,
    types::{
        CreateGrnRequest, CreatePurchaseOrderRequest, CreateQaInspectionRequest,
        GoodsReceiptItemCalcRow, GoodsReceiptNoteResponse,
        PurchaseHistoryResponse, PurchaseOrderItemRequest, PurchaseOrderResponse,
        PurchaseOrderSummaryResponse, TransitPurchaseRequest,
    },
};

const STATUS_DRAFT: &str = "DRAFT";
const STATUS_APPROVED: &str = "APPROVED";
const STATUS_REJECTED: &str = "REJECTED";
const STATUS_SENT: &str = "SENT";
const STATUS_RECEIVED: &str = "RECEIVED";
const STATUS_GRN_COMPLETED: &str = "GRN_COMPLETED";
const STATUS_COMPLETED: &str = "COMPLETED";
const STATUS_CANCELLED: &str = "CANCELLED";

const HISTORY_SUBMITTED: &str = "SUBMITTED";
const HISTORY_IN_TRANSIT: &str = "IN_TRANSIT";

const GRN_QA_PASSED: &str = "QA_PASSED";
const GRN_QA_FAILED: &str = "QA_FAILED";
const GRN_COMPLETED: &str = "COMPLETED";

const EPSILON: f64 = 0.000_001;

struct PurchaseTotals {
    subtotal: f64,
    discount_pct: f64,
    tax_amount: f64,
    total_amount: f64,
}

struct QaOutcome {
    accepted_qty: f64,
    rejected_qty: f64,
    notes: Option<String>,
}

pub async fn list_purchases(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
    status: String,
    in_transit_only: bool,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);

    let purchases = repo::list(pool, &search, &like, &status, in_transit_only, limit, offset).await?;
    let total = repo::count(pool, &search, &like, &status, in_transit_only).await?;

    Ok(json!({
        "data": purchases
            .into_iter()
            .map(PurchaseOrderSummaryResponse::from)
            .collect::<Vec<_>>(),
        "meta": crate::common::pagination::PaginationMeta { page, limit, total }
    }))
}

pub async fn get_purchase(pool: &PgPool, id: &str) -> AppResult<PurchaseOrderResponse> {
    ensure_uuid(id, "purchase order id")?;

    let order = repo::find_order(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".into()))?;
    let items = repo::find_order_items(pool, id).await?;
    let grns = repo::find_order_grns(pool, id).await?;

    Ok(PurchaseOrderResponse::from_parts(order, items, grns))
}

pub async fn create_purchase(
    pool: &PgPool,
    req: &CreatePurchaseOrderRequest,
    created_by: &str,
) -> AppResult<PurchaseOrderResponse> {
    ensure_uuid(created_by, "user id")?;
    ensure_uuid(&req.supplier_id, "supplierId")?;
    ensure_supplier_exists(pool, &req.supplier_id).await?;
    validate_inventory_refs(pool, &req.items).await?;

    let expected_delivery = parse_optional_datetime(&req.expected_delivery, "expectedDelivery")?;
    let totals = calculate_purchase_totals(req.discount_pct, req.tax_amount, &req.items)?;
    let currency = normalized_currency(req.currency.as_deref());
    let purchase_id = Uuid::now_v7().to_string();

    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    repo::insert_order(
        &mut tx,
        &purchase_id,
        &req.supplier_id,
        STATUS_DRAFT,
        expected_delivery,
        totals.subtotal,
        totals.discount_pct,
        totals.tax_amount,
        totals.total_amount,
        req.notes.as_deref(),
        &currency,
        req.exchange_rate.map(round_exchange_rate),
        created_by,
    )
    .await?;

    insert_purchase_items(&mut tx, &purchase_id, &req.items).await?;

    repo::insert_status_history(
        &mut tx,
        &Uuid::now_v7().to_string(),
        &purchase_id,
        None,
        STATUS_DRAFT,
        Some("Purchase order created"),
        created_by,
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;

    get_purchase(pool, &purchase_id).await
}

pub async fn update_purchase(
    pool: &PgPool,
    id: &str,
    req: &CreatePurchaseOrderRequest,
    updated_by: &str,
) -> AppResult<PurchaseOrderResponse> {
    ensure_uuid(id, "purchase order id")?;
    ensure_uuid(updated_by, "user id")?;
    ensure_uuid(&req.supplier_id, "supplierId")?;
    ensure_supplier_exists(pool, &req.supplier_id).await?;
    validate_inventory_refs(pool, &req.items).await?;

    let state = repo::find_order_state(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".into()))?;

    if state.status != STATUS_DRAFT && state.status != STATUS_REJECTED {
        return Err(AppError::Conflict(
            "Only draft or rejected purchase orders can be updated".into(),
        ));
    }

    let next_status = STATUS_DRAFT;

    let expected_delivery = parse_optional_datetime(&req.expected_delivery, "expectedDelivery")?;
    let totals = calculate_purchase_totals(req.discount_pct, req.tax_amount, &req.items)?;
    let currency = normalized_currency(req.currency.as_deref());

    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    repo::update_order_header(
        &mut tx,
        id,
        &req.supplier_id,
        next_status,
        expected_delivery,
        totals.subtotal,
        totals.discount_pct,
        totals.tax_amount,
        totals.total_amount,
        req.notes.as_deref(),
        &currency,
        req.exchange_rate.map(round_exchange_rate),
    )
    .await?;

    repo::delete_order_items(&mut tx, id).await?;
    insert_purchase_items(&mut tx, id, &req.items).await?;

    if state.status == STATUS_REJECTED {
        repo::insert_status_history(
            &mut tx,
            &Uuid::now_v7().to_string(),
            id,
            Some(STATUS_REJECTED),
            STATUS_DRAFT,
            Some("Purchase order revised after rejection"),
            updated_by,
        )
        .await?;
    }

    tx.commit().await.map_err(AppError::Database)?;

    get_purchase(pool, id).await
}

pub async fn get_purchase_history(pool: &PgPool, id: &str) -> AppResult<PurchaseHistoryResponse> {
    ensure_uuid(id, "purchase order id")?;
    ensure_purchase_exists(pool, id).await?;

    let status_history = repo::find_order_history(pool, id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();
    let approvals = repo::find_order_approvals(pool, id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();

    Ok(PurchaseHistoryResponse {
        status_history,
        approvals,
    })
}

pub async fn submit_purchase(
    pool: &PgPool,
    id: &str,
    user_id: &str,
    notes: Option<String>,
) -> AppResult<PurchaseOrderResponse> {
    ensure_uuid(id, "purchase order id")?;
    ensure_uuid(user_id, "user id")?;

    let state = repo::find_order_state(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".into()))?;
    if state.status != STATUS_DRAFT {
        return Err(AppError::Conflict(
            "Only draft purchase orders can be submitted".into(),
        ));
    }

    let items = repo::find_order_item_calcs(pool, id).await?;
    if items.is_empty() {
        return Err(AppError::Validation(
            "Purchase order must contain at least one item".into(),
        ));
    }

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::insert_status_history(
        &mut tx,
        &Uuid::now_v7().to_string(),
        id,
        Some(STATUS_DRAFT),
        HISTORY_SUBMITTED,
        action_note(notes.as_deref(), "Purchase order submitted"),
        user_id,
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    get_purchase(pool, id).await
}

pub async fn approve_purchase(
    pool: &PgPool,
    id: &str,
    user_id: &str,
    notes: Option<String>,
) -> AppResult<PurchaseOrderResponse> {
    ensure_uuid(id, "purchase order id")?;
    ensure_uuid(user_id, "user id")?;

    let state = repo::find_order_state(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".into()))?;
    if state.status != STATUS_DRAFT {
        return Err(AppError::Conflict(
            "Only draft purchase orders can be approved".into(),
        ));
    }

    let items = repo::find_order_item_calcs(pool, id).await?;
    if items.is_empty() {
        return Err(AppError::Validation(
            "Purchase order must contain at least one item".into(),
        ));
    }

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::insert_approval(
        &mut tx,
        &Uuid::now_v7().to_string(),
        id,
        user_id,
        true,
        notes.as_deref(),
    )
    .await?;
    repo::update_order_status(&mut tx, id, STATUS_APPROVED).await?;
    repo::insert_status_history(
        &mut tx,
        &Uuid::now_v7().to_string(),
        id,
        Some(STATUS_DRAFT),
        STATUS_APPROVED,
        action_note(notes.as_deref(), "Purchase order approved"),
        user_id,
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    get_purchase(pool, id).await
}

pub async fn reject_purchase(
    pool: &PgPool,
    id: &str,
    user_id: &str,
    notes: Option<String>,
) -> AppResult<PurchaseOrderResponse> {
    ensure_uuid(id, "purchase order id")?;
    ensure_uuid(user_id, "user id")?;

    let state = repo::find_order_state(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".into()))?;
    if state.status != STATUS_DRAFT && state.status != STATUS_APPROVED {
        return Err(AppError::Conflict(
            "Only draft or approved purchase orders can be rejected".into(),
        ));
    }

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::insert_approval(
        &mut tx,
        &Uuid::now_v7().to_string(),
        id,
        user_id,
        false,
        notes.as_deref(),
    )
    .await?;
    repo::update_order_status(&mut tx, id, STATUS_REJECTED).await?;
    repo::insert_status_history(
        &mut tx,
        &Uuid::now_v7().to_string(),
        id,
        Some(state.status.as_str()),
        STATUS_REJECTED,
        action_note(notes.as_deref(), "Purchase order rejected"),
        user_id,
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    get_purchase(pool, id).await
}

pub async fn send_purchase(
    pool: &PgPool,
    id: &str,
    user_id: &str,
    notes: Option<String>,
) -> AppResult<PurchaseOrderResponse> {
    ensure_uuid(id, "purchase order id")?;
    ensure_uuid(user_id, "user id")?;

    let state = repo::find_order_state(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".into()))?;
    if state.status != STATUS_DRAFT && state.status != STATUS_APPROVED {
        return Err(AppError::Conflict(
            "Only draft or approved purchase orders can be sent".into(),
        ));
    }

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::update_order_status(&mut tx, id, STATUS_SENT).await?;
    repo::insert_status_history(
        &mut tx,
        &Uuid::now_v7().to_string(),
        id,
        Some(state.status.as_str()),
        STATUS_SENT,
        action_note(notes.as_deref(), "Purchase order sent to supplier"),
        user_id,
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    get_purchase(pool, id).await
}

pub async fn mark_purchase_in_transit(
    pool: &PgPool,
    id: &str,
    user_id: &str,
    req: &TransitPurchaseRequest,
) -> AppResult<PurchaseOrderResponse> {
    ensure_uuid(id, "purchase order id")?;
    ensure_uuid(user_id, "user id")?;

    let state = repo::find_order_state(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".into()))?;
    if state.status != STATUS_SENT {
        return Err(AppError::Conflict(
            "Only sent purchase orders can be marked in transit".into(),
        ));
    }

    let expected_delivery = parse_optional_datetime(&req.expected_delivery, "expectedDelivery")?;

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    if let Some(expected_delivery) = expected_delivery {
        repo::update_order_expected_delivery(&mut tx, id, expected_delivery).await?;
    }
    repo::insert_status_history(
        &mut tx,
        &Uuid::now_v7().to_string(),
        id,
        Some(STATUS_SENT),
        HISTORY_IN_TRANSIT,
        action_note(req.notes.as_deref(), "Shipment marked in transit"),
        user_id,
    )
    .await?;
    tx.commit().await.map_err(AppError::Database)?;

    get_purchase(pool, id).await
}

pub async fn create_grn(
    pool: &PgPool,
    purchase_order_id: &str,
    req: &CreateGrnRequest,
    user_id: &str,
) -> AppResult<GoodsReceiptNoteResponse> {
    ensure_uuid(purchase_order_id, "purchase order id")?;
    ensure_uuid(user_id, "user id")?;

    let order_state = repo::find_order_state(pool, purchase_order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".into()))?;
    if order_state.status == STATUS_REJECTED
        || order_state.status == STATUS_CANCELLED
        || order_state.status == STATUS_COMPLETED
    {
        return Err(AppError::Conflict(
            "Cannot receive goods for this purchase order".into(),
        ));
    }

    let po_items = repo::find_order_item_calcs(pool, purchase_order_id).await?;
    if po_items.is_empty() {
        return Err(AppError::Validation(
            "Purchase order must contain at least one item before receiving goods".into(),
        ));
    }

    let mut po_item_map = HashMap::new();
    for item in po_items {
        po_item_map.insert(item.id.clone(), item);
    }

    let mut receipt_totals: HashMap<String, f64> = HashMap::new();
    for item in &req.items {
        ensure_uuid(&item.po_item_id, "poItemId")?;
        let po_item = po_item_map.get(&item.po_item_id).ok_or_else(|| {
            AppError::Validation("GRN items must belong to the purchase order".into())
        })?;
        let remaining_qty = po_item.quantity - po_item.received_qty;
        if remaining_qty <= EPSILON {
            return Err(AppError::Conflict(format!(
                "Item '{}' has already been fully received",
                po_item.description
            )));
        }

        let total_for_item = receipt_totals.entry(item.po_item_id.clone()).or_insert(0.0);
        *total_for_item = round_qty(*total_for_item + item.received_qty);
        if *total_for_item - remaining_qty > EPSILON {
            return Err(AppError::Conflict(format!(
                "Received quantity for '{}' exceeds the remaining quantity",
                po_item.description
            )));
        }
    }

    let received_at = parse_optional_datetime(&req.received_at, "receivedAt")?
        .unwrap_or_else(Utc::now);
    let grn_id = Uuid::now_v7().to_string();

    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    repo::insert_grn(&mut tx, &grn_id, purchase_order_id, user_id, received_at).await?;

    for item in &req.items {
        repo::insert_grn_item(
            &mut tx,
            &Uuid::now_v7().to_string(),
            &grn_id,
            &item.po_item_id,
            round_qty(item.received_qty),
            item.unit_cost.map(round_money),
            item.notes.as_deref(),
        )
        .await?;
        repo::add_received_qty_to_po_item(&mut tx, &item.po_item_id, round_qty(item.received_qty)).await?;
    }

    let next_status = if purchase_fully_received(&po_item_map, &receipt_totals) {
        STATUS_GRN_COMPLETED
    } else {
        STATUS_RECEIVED
    };

    repo::update_order_status(&mut tx, purchase_order_id, next_status).await?;
    repo::insert_status_history(
        &mut tx,
        &Uuid::now_v7().to_string(),
        purchase_order_id,
        Some(order_state.status.as_str()),
        next_status,
        action_note(req.notes.as_deref(), "Goods receipt note created"),
        user_id,
    )
    .await?;

    tx.commit().await.map_err(AppError::Database)?;

    get_grn(pool, purchase_order_id, &grn_id).await
}

pub async fn get_grn(
    pool: &PgPool,
    purchase_order_id: &str,
    grn_id: &str,
) -> AppResult<GoodsReceiptNoteResponse> {
    ensure_uuid(purchase_order_id, "purchase order id")?;
    ensure_uuid(grn_id, "grn id")?;

    let grn = repo::find_grn(pool, purchase_order_id, grn_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Goods receipt note not found".into()))?;
    let items = repo::find_grn_items(pool, grn_id).await?;
    let inspections = repo::find_grn_inspections(pool, grn_id).await?;

    Ok(GoodsReceiptNoteResponse::from_parts(grn, items, inspections))
}

pub async fn record_qa(
    pool: &PgPool,
    purchase_order_id: &str,
    grn_id: &str,
    req: &CreateQaInspectionRequest,
    user_id: &str,
) -> AppResult<GoodsReceiptNoteResponse> {
    ensure_uuid(purchase_order_id, "purchase order id")?;
    ensure_uuid(grn_id, "grn id")?;
    ensure_uuid(user_id, "user id")?;

    let qa_status = normalize_grn_status(&req.status)?;
    let grn_state = repo::find_grn_state(pool, purchase_order_id, grn_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Goods receipt note not found".into()))?;
    if grn_state.status == GRN_COMPLETED {
        return Err(AppError::Conflict(
            "QA is already completed for this GRN".into(),
        ));
    }

    let grn_items = repo::find_grn_item_calcs(pool, grn_id).await?;
    if grn_items.is_empty() {
        return Err(AppError::Validation(
            "GRN must contain at least one item".into(),
        ));
    }

    let requested_updates = build_requested_qa_updates(req, &grn_items)?;
    let order_state = repo::find_order_state(pool, purchase_order_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Purchase order not found".into()))?;
    let po_items = repo::find_order_item_calcs(pool, purchase_order_id).await?;
    let fully_received = po_items
        .iter()
        .all(|item| item.received_qty + EPSILON >= item.quantity);

    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    for grn_item in &grn_items {
        if let Some(update) = requested_updates.get(&grn_item.id) {
            repo::update_grn_item_outcome(
                &mut tx,
                &grn_item.id,
                update.accepted_qty,
                update.rejected_qty,
                update.notes.as_deref(),
            )
            .await?;

            if qa_status != GRN_QA_FAILED && update.accepted_qty > EPSILON {
                if let (Some(inventory_item_id), Some(unit_cost)) =
                    (&grn_item.inventory_item_id, grn_item.unit_cost)
                {
                    repo::update_inventory_cost(&mut tx, inventory_item_id, round_money(unit_cost)).await?;
                }
            }
        }
    }

    repo::insert_qa_inspection(
        &mut tx,
        &Uuid::now_v7().to_string(),
        grn_id,
        user_id,
        qa_status,
        req.notes.as_deref(),
    )
    .await?;
    repo::update_grn_status(&mut tx, grn_id, qa_status).await?;

    if qa_status != GRN_QA_FAILED && fully_received {
        repo::update_order_status(&mut tx, purchase_order_id, STATUS_COMPLETED).await?;
        repo::insert_status_history(
            &mut tx,
            &Uuid::now_v7().to_string(),
            purchase_order_id,
            Some(order_state.status.as_str()),
            STATUS_COMPLETED,
            action_note(req.notes.as_deref(), "Purchase order completed after QA"),
            user_id,
        )
        .await?;
    }

    tx.commit().await.map_err(AppError::Database)?;

    get_grn(pool, purchase_order_id, grn_id).await
}

async fn ensure_supplier_exists(pool: &PgPool, supplier_id: &str) -> AppResult<()> {
    if repo::supplier_exists(pool, supplier_id).await? {
        Ok(())
    } else {
        Err(AppError::NotFound("Supplier not found".into()))
    }
}

async fn ensure_purchase_exists(pool: &PgPool, id: &str) -> AppResult<()> {
    if repo::find_order_state(pool, id).await?.is_some() {
        Ok(())
    } else {
        Err(AppError::NotFound("Purchase order not found".into()))
    }
}

async fn validate_inventory_refs(
    pool: &PgPool,
    items: &[PurchaseOrderItemRequest],
) -> AppResult<()> {
    for item in items {
        if let Some(inventory_item_id) = &item.inventory_item_id {
            ensure_uuid(inventory_item_id, "inventoryItemId")?;
            if !repo::inventory_item_exists(pool, inventory_item_id).await? {
                return Err(AppError::NotFound("Inventory item not found".into()));
            }
        }
    }

    Ok(())
}

async fn insert_purchase_items(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    purchase_order_id: &str,
    items: &[PurchaseOrderItemRequest],
) -> AppResult<()> {
    for item in items {
        repo::insert_order_item(
            tx,
            &Uuid::now_v7().to_string(),
            purchase_order_id,
            item.inventory_item_id.as_deref(),
            item.description.trim(),
            round_qty(item.quantity),
            round_money(item.unit_price),
        )
        .await?;
    }

    Ok(())
}

fn calculate_purchase_totals(
    discount_pct: Option<f64>,
    tax_amount: Option<f64>,
    items: &[PurchaseOrderItemRequest],
) -> AppResult<PurchaseTotals> {
    if items.is_empty() {
        return Err(AppError::Validation(
            "Purchase order must contain at least one item".into(),
        ));
    }

    let normalized_discount_pct = round_money(discount_pct.unwrap_or(0.0));
    let normalized_tax_amount = round_money(tax_amount.unwrap_or(0.0));
    if normalized_discount_pct < 0.0 || normalized_discount_pct > 100.0 {
        return Err(AppError::Validation(
            "discountPct must be between 0 and 100".into(),
        ));
    }
    if normalized_tax_amount < 0.0 {
        return Err(AppError::Validation("taxAmount cannot be negative".into()));
    }

    let mut subtotal = 0.0;
    for item in items {
        if item.description.trim().is_empty() {
            return Err(AppError::Validation(
                "Purchase item description is required".into(),
            ));
        }
        if item.quantity <= 0.0 {
            return Err(AppError::Validation(
                "Purchase item quantity must be greater than zero".into(),
            ));
        }
        if item.unit_price < 0.0 {
            return Err(AppError::Validation(
                "Purchase item unit price cannot be negative".into(),
            ));
        }

        subtotal = round_money(subtotal + round_qty(item.quantity) * round_money(item.unit_price));
    }

    let discount_amount = round_money(subtotal * normalized_discount_pct / 100.0);
    let total_amount = round_money(subtotal - discount_amount + normalized_tax_amount);
    if total_amount < 0.0 {
        return Err(AppError::Validation(
            "Purchase order total cannot be negative".into(),
        ));
    }

    Ok(PurchaseTotals {
        subtotal,
        discount_pct: normalized_discount_pct,
        tax_amount: normalized_tax_amount,
        total_amount,
    })
}

fn build_requested_qa_updates(
    req: &CreateQaInspectionRequest,
    grn_items: &[GoodsReceiptItemCalcRow],
) -> AppResult<HashMap<String, QaOutcome>> {
    let mut requested = HashMap::new();
    let mut request_map = HashMap::new();
    let valid_ids = grn_items
        .iter()
        .map(|item| item.id.as_str())
        .collect::<std::collections::HashSet<_>>();

    for item in &req.items {
        ensure_uuid(&item.grn_item_id, "grnItemId")?;
        if !valid_ids.contains(item.grn_item_id.as_str()) {
            return Err(AppError::Validation(
                "QA items must belong to the goods receipt note".into(),
            ));
        }
        request_map.insert(item.grn_item_id.clone(), item);
    }

    for grn_item in grn_items {
        let default_pass = req.status.eq_ignore_ascii_case(GRN_QA_PASSED)
            || req.status.eq_ignore_ascii_case(GRN_COMPLETED);

        let (accepted_qty, rejected_qty, notes) = if let Some(item) = request_map.get(&grn_item.id) {
            let accepted = round_qty(item.accepted_qty.unwrap_or(if default_pass {
                grn_item.received_qty
            } else {
                0.0
            }));
            let rejected = round_qty(item.rejected_qty.unwrap_or(if default_pass {
                0.0
            } else {
                grn_item.received_qty
            }));
            (accepted, rejected, item.notes.clone())
        } else if default_pass {
            (round_qty(grn_item.received_qty), 0.0, None)
        } else {
            (0.0, round_qty(grn_item.received_qty), None)
        };

        if accepted_qty < 0.0 || rejected_qty < 0.0 {
            return Err(AppError::Validation(
                "Accepted and rejected quantities cannot be negative".into(),
            ));
        }

        let total = round_qty(accepted_qty + rejected_qty);
        if (total - round_qty(grn_item.received_qty)).abs() > EPSILON {
            return Err(AppError::Validation(
                "Accepted and rejected quantities must match the received quantity".into(),
            ));
        }

        requested.insert(
            grn_item.id.clone(),
            QaOutcome {
                accepted_qty,
                rejected_qty,
                notes,
            },
        );
    }

    Ok(requested)
}

fn purchase_fully_received(
    po_item_map: &HashMap<String, super::types::PurchaseOrderItemCalcRow>,
    receipt_totals: &HashMap<String, f64>,
) -> bool {
    po_item_map.iter().all(|(item_id, item)| {
        let just_received = receipt_totals.get(item_id).copied().unwrap_or(0.0);
        item.received_qty + just_received + EPSILON >= item.quantity
    })
}

fn parse_optional_datetime(
    value: &Option<String>,
    field_name: &str,
) -> AppResult<Option<DateTime<Utc>>> {
    match value {
        Some(raw) => {
            let parsed = DateTime::parse_from_rfc3339(raw).map_err(|_| {
                AppError::Validation(format!("{} must be a valid RFC3339 datetime", field_name))
            })?;
            Ok(Some(parsed.with_timezone(&Utc)))
        }
        None => Ok(None),
    }
}

fn normalize_grn_status(status: &str) -> AppResult<&'static str> {
    if status.eq_ignore_ascii_case(GRN_QA_PASSED) {
        Ok(GRN_QA_PASSED)
    } else if status.eq_ignore_ascii_case(GRN_QA_FAILED) {
        Ok(GRN_QA_FAILED)
    } else if status.eq_ignore_ascii_case(GRN_COMPLETED) {
        Ok(GRN_COMPLETED)
    } else {
        Err(AppError::Validation(
            "status must be one of QA_PASSED, QA_FAILED, or COMPLETED".into(),
        ))
    }
}

fn normalized_currency(currency: Option<&str>) -> String {
    let value = currency.unwrap_or("NPR").trim();
    if value.is_empty() {
        "NPR".to_string()
    } else {
        value.to_uppercase()
    }
}

fn action_note<'a>(notes: Option<&'a str>, default: &'a str) -> Option<&'a str> {
    match notes {
        Some(value) if !value.trim().is_empty() => Some(value),
        _ => Some(default),
    }
}

fn ensure_uuid(value: &str, field_name: &str) -> AppResult<()> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| AppError::Validation(format!("{} must be a valid UUID", field_name)))
}

fn round_money(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round_exchange_rate(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn round_qty(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}
