use axum::{
    extract::{Path, Query},
    routing::{get, post, put},
    Json, Router,
};
use crate::errors::AppError;
use crate::middleware::{auth::AuthUser, tenant::TenantDbPool};
use crate::AppState;

use super::service::HrService;
use super::types::*;
use crate::common::pagination::PaginationMeta;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/hr/employees", axum::routing::get(list_employees).post(create_employee))
        .route("/hr/employees/export", axum::routing::get(export_employees))
        .route("/hr/employees/:id", axum::routing::get(get_employee).put(update_employee).delete(delete_employee))
        .route("/hr/leave/types", axum::routing::get(list_leave_types).post(create_leave_type))
        .route("/hr/leave/requests", axum::routing::get(list_leave_requests).post(create_leave_request))
        .route("/hr/leave/requests/:id", axum::routing::put(update_leave_request))
        .route("/hr/payroll/periods", axum::routing::get(list_payroll_periods).post(create_payroll_period))
        .route("/hr/payroll/periods/:id/run", axum::routing::post(run_payroll))
        .route("/hr/payroll/periods/:id/entries", axum::routing::get(list_payroll_entries))
        .route("/hr/payroll/periods/:id/export", axum::routing::get(export_payroll))
        .route("/hr/attendance", axum::routing::get(list_attendance))
        .route("/hr/attendance/clock-in", axum::routing::post(clock_in))
        .route("/hr/attendance/clock-out", axum::routing::post(clock_out))
}

async fn list_employees(
    tenant_db: TenantDbPool,
    Query(query): Query<ListQuery>,
) -> Result<Json<(Vec<EmployeeResponse>, PaginationMeta)>, AppError> {
    let result = HrService::list_employees(&tenant_db.pool, query).await?;
    Ok(Json(result))
}

async fn get_employee(
    tenant_db: TenantDbPool,
    Path(id): Path<String>,
) -> Result<Json<EmployeeResponse>, AppError> {
    let employee = HrService::get_employee(&tenant_db.pool, &id).await?;
    Ok(Json(employee))
}

async fn create_employee(
    tenant_db: TenantDbPool,
    Json(req): Json<CreateEmployeeRequest>,
) -> Result<Json<EmployeeResponse>, AppError> {
    let employee = HrService::create_employee(&tenant_db.pool, req, None).await?;
    Ok(Json(employee))
}

async fn update_employee(
    tenant_db: TenantDbPool,
    Path(id): Path<String>,
    Json(req): Json<UpdateEmployeeRequest>,
) -> Result<Json<EmployeeResponse>, AppError> {
    let employee = HrService::update_employee(&tenant_db.pool, &id, req).await?;
    Ok(Json(employee))
}

async fn delete_employee(
    tenant_db: TenantDbPool,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    HrService::delete_employee(&tenant_db.pool, &id).await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

async fn export_employees(
    tenant_db: TenantDbPool,
) -> Result<Json<serde_json::Value>, AppError> {
    let query = ListQuery::default();
    let (employees, _) = HrService::list_employees(&tenant_db.pool, query).await?;
    let data: Vec<Vec<serde_json::Value>> = employees.iter().map(|e| {
        vec![
            serde_json::json!(e.employee_no),
            serde_json::json!(e.first_name),
            serde_json::json!(e.last_name),
            serde_json::json!(e.email),
            serde_json::json!(e.phone),
            serde_json::json!(e.employment_type),
            serde_json::json!(e.department),
            serde_json::json!(e.designation),
        ]
    }).collect();
    Ok(Json(serde_json::json!({
        "headers": ["Employee No", "First Name", "Last Name", "Email", "Phone", "Type", "Department", "Designation"],
        "data": data
    })))
}

async fn list_leave_types(
    tenant_db: TenantDbPool,
) -> Result<Json<Vec<LeaveTypeResponse>>, AppError> {
    let types = HrService::list_leave_types(&tenant_db.pool).await?;
    Ok(Json(types))
}

async fn create_leave_type(
    tenant_db: TenantDbPool,
    Json(req): Json<CreateLeaveTypeRequest>,
) -> Result<Json<LeaveTypeResponse>, AppError> {
    let leave_type = HrService::create_leave_type(&tenant_db.pool, req).await?;
    Ok(Json(leave_type))
}

async fn list_leave_requests(
    tenant_db: TenantDbPool,
    Query(query): Query<ListQuery>,
) -> Result<Json<(Vec<LeaveRequestResponse>, PaginationMeta)>, AppError> {
    let result = HrService::list_leave_requests(&tenant_db.pool, query).await?;
    Ok(Json(result))
}

async fn create_leave_request(
    tenant_db: TenantDbPool,
    Json(req): Json<CreateLeaveRequestRequest>,
) -> Result<Json<LeaveRequestResponse>, AppError> {
    let request = HrService::create_leave_request(&tenant_db.pool, req).await?;
    Ok(Json(request))
}

async fn update_leave_request(
    tenant_db: TenantDbPool,
    _auth: AuthUser,
    Path(id): Path<String>,
    Json(req): Json<UpdateLeaveRequestRequest>,
) -> Result<Json<LeaveRequestResponse>, AppError> {
    let status = req.status.unwrap_or_else(|| "APPROVED".to_string());
    let request = HrService::update_leave_request(&tenant_db.pool, &id, &status, "system").await?;
    Ok(Json(request))
}

async fn list_payroll_periods(
    tenant_db: TenantDbPool,
    Query(query): Query<ListQuery>,
) -> Result<Json<(Vec<PayrollPeriodResponse>, PaginationMeta)>, AppError> {
    let result = HrService::list_payroll_periods(&tenant_db.pool, query).await?;
    Ok(Json(result))
}

async fn create_payroll_period(
    tenant_db: TenantDbPool,
    Json(req): Json<CreatePayrollPeriodRequest>,
) -> Result<Json<PayrollPeriodResponse>, AppError> {
    let period = HrService::create_payroll_period(&tenant_db.pool, req).await?;
    Ok(Json(period))
}

async fn run_payroll(
    tenant_db: TenantDbPool,
    Path(id): Path<String>,
) -> Result<Json<Vec<PayrollEntryResponse>>, AppError> {
    let entries = HrService::run_payroll(&tenant_db.pool, &id).await?;
    Ok(Json(entries))
}

async fn list_payroll_entries(
    tenant_db: TenantDbPool,
    Path(id): Path<String>,
) -> Result<Json<Vec<PayrollEntryResponse>>, AppError> {
    let entries = HrService::list_payroll_entries(&tenant_db.pool, &id).await?;
    Ok(Json(entries))
}

async fn export_payroll(
    tenant_db: TenantDbPool,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let entries = HrService::list_payroll_entries(&tenant_db.pool, &id).await?;
    let data: Vec<Vec<serde_json::Value>> = entries.iter().map(|e| {
        vec![
            serde_json::json!(e.employee_id),
            serde_json::json!(e.basic_salary),
            serde_json::json!(e.gross_salary),
            serde_json::json!(e.net_salary),
        ]
    }).collect();
    Ok(Json(serde_json::json!({
        "headers": ["Employee ID", "Basic Salary", "Gross Salary", "Net Salary"],
        "data": data
    })))
}

async fn list_attendance(
    tenant_db: TenantDbPool,
    Query(query): Query<ListQuery>,
) -> Result<Json<(Vec<AttendanceRecordResponse>, PaginationMeta)>, AppError> {
    let result = HrService::list_attendance(&tenant_db.pool, query).await?;
    Ok(Json(result))
}

async fn clock_in(
    tenant_db: TenantDbPool,
    Json(req): Json<EmployeeIdRequest>,
) -> Result<Json<AttendanceRecordResponse>, AppError> {
    let record = HrService::clock_in(&tenant_db.pool, &req.employee_id).await?;
    Ok(Json(record))
}

async fn clock_out(
    tenant_db: TenantDbPool,
    Json(req): Json<EmployeeIdRequest>,
) -> Result<Json<AttendanceRecordResponse>, AppError> {
    let record = HrService::clock_out(&tenant_db.pool, &req.employee_id).await?;
    Ok(Json(record))
}

#[derive(serde::Deserialize)]
struct EmployeeIdRequest {
    employee_id: String,
}