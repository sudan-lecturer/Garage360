use serde_json::json;
use sqlx::PgPool;

use crate::errors::{AppError, AppResult};
use crate::common::pagination::PaginationMeta;

use super::{
    repo,
    types::{
        CreateCustomerRequest, CustomerProfileResponse, CustomerResponse, FinancialSnapshotResponse,
        InvoiceSummaryResponse, JobSummaryResponse, ServiceChronicleEntry, VehicleSummaryResponse,
    },
};

pub async fn list_customers(
    pool: &PgPool,
    page: i64,
    limit: i64,
    search: String,
    customer_type: String,
) -> AppResult<serde_json::Value> {
    let offset = (page - 1) * limit;
    let like = format!("%{}%", search);
    let normalized_type = customer_type.trim().to_uppercase();

    let customers = repo::list(pool, &search, &like, &normalized_type, limit, offset).await?;
    let total = repo::count(pool, &search, &like, &normalized_type).await?;

    Ok(json!({
        "data": customers.into_iter().map(CustomerResponse::from).collect::<Vec<_>>(),
        "meta": crate::common::pagination::PaginationMeta { page, limit, total }
    }))
}

pub async fn search_customers(
    pool: &PgPool,
    search: String,
    customer_type: String,
) -> AppResult<Vec<CustomerResponse>> {
    let like = format!("%{}%", search);
    let normalized_type = customer_type.trim().to_uppercase();
    let customers = repo::list(pool, &search, &like, &normalized_type, 20, 0).await?;

    Ok(customers
        .into_iter()
        .map(CustomerResponse::from)
        .collect::<Vec<_>>())
}

pub async fn get_customer(pool: &PgPool, id: &str) -> AppResult<CustomerProfileResponse> {
    let row = repo::find_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Customer not found".into()))?;

    let base = CustomerResponse::from(row);
    let vehicles = repo::list_customer_vehicles(pool, id).await?;
    let jobs = repo::list_customer_jobs(pool, id).await?;
    let invoices = repo::list_customer_invoices(pool, id).await?;
    let snapshot = repo::get_customer_financial_snapshot(pool, id).await?;
    let chronicle = repo::list_customer_service_chronicle(pool, id).await?;

    let total_spend = snapshot.total_spend.parse::<f64>().unwrap_or(0.0);
    let tier = if total_spend >= 500_000.0 {
        "PLATINUM"
    } else if total_spend >= 200_000.0 {
        "GOLD"
    } else if total_spend >= 75_000.0 {
        "SILVER"
    } else {
        "BRONZE"
    };

    Ok(CustomerProfileResponse {
        id: base.id,
        customer_type: base.customer_type,
        first_name: base.first_name,
        last_name: base.last_name,
        company_name: base.company_name,
        email: base.email,
        phone: base.phone,
        address: base.address,
        created_at: base.created_at,
        name: base.name,
        tier: tier.to_string(),
        financial_snapshot: FinancialSnapshotResponse {
            total_invoices: snapshot.total_invoices,
            total_spend: snapshot.total_spend,
            outstanding_balance: snapshot.outstanding_balance,
            paid_invoices: snapshot.paid_invoices,
            last_invoice_at: snapshot.last_invoice_at.map(|d| d.to_rfc3339()),
        },
        service_chronicle: chronicle
            .into_iter()
            .map(|entry| ServiceChronicleEntry {
                id: entry.id,
                kind: entry.kind,
                reference_no: entry.reference_no,
                status: entry.status,
                occurred_at: entry.occurred_at.to_rfc3339(),
                summary: entry.summary,
            })
            .collect::<Vec<_>>(),
        vehicles: vehicles
            .into_iter()
            .map(|vehicle| VehicleSummaryResponse {
                id: vehicle.id,
                registration_no: vehicle.registration_no,
                brand: vehicle.make,
                model: vehicle.model,
                year: vehicle.year,
            })
            .collect::<Vec<_>>(),
        jobs: jobs
            .into_iter()
            .map(|job| JobSummaryResponse {
                id: job.id,
                job_no: job.job_no,
                status: job.status,
                created_at: job.created_at.to_rfc3339(),
            })
            .collect::<Vec<_>>(),
        invoices: invoices
            .into_iter()
            .map(|invoice| InvoiceSummaryResponse {
                id: invoice.id,
                invoice_no: invoice.invoice_no,
                status: invoice.status,
                total_amount: invoice.total_amount,
                created_at: invoice.created_at.to_rfc3339(),
            })
            .collect::<Vec<_>>(),
    })
}

pub async fn create_customer(
    pool: &PgPool,
    req: &CreateCustomerRequest,
    created_by: &str,
) -> AppResult<CustomerResponse> {
    repo::create(pool, req, created_by)
        .await
        .map(CustomerResponse::from)
}

pub async fn update_customer(
    pool: &PgPool,
    id: &str,
    req: &CreateCustomerRequest,
) -> AppResult<CustomerResponse> {
    repo::update(pool, id, req)
        .await?
        .map(CustomerResponse::from)
        .ok_or_else(|| AppError::NotFound("Customer not found".into()))
}

pub async fn delete_customer(pool: &PgPool, id: &str) -> AppResult<serde_json::Value> {
    let rows_affected = repo::soft_delete(pool, id).await?;
    if rows_affected == 0 {
        return Err(AppError::NotFound("Customer not found".into()));
    }

    Ok(json!({ "deleted": true }))
}
