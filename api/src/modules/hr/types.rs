use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;
use crate::common::pagination::PaginationMeta;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeResponse {
    pub id: String,
    pub employee_no: String,
    pub user_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: String,
    pub employment_type: String,
    pub department: Option<String>,
    pub designation: Option<String>,
    pub join_date: Option<String>,
    pub salary: Option<String>,
    pub is_active: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct EmployeeRow {
    pub id: String,
    pub employee_no: String,
    pub user_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: String,
    pub employment_type: String,
    pub department: Option<String>,
    pub designation: Option<String>,
    pub join_date: Option<chrono::NaiveDate>,
    pub salary: Option<String>,
    pub is_active: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<EmployeeRow> for EmployeeResponse {
    fn from(row: EmployeeRow) -> Self {
        Self {
            id: row.id,
            employee_no: row.employee_no,
            user_id: row.user_id,
            first_name: row.first_name,
            last_name: row.last_name,
            email: row.email,
            phone: row.phone,
            employment_type: row.employment_type,
            department: row.department,
            designation: row.designation,
            join_date: row.join_date.map(|d| d.to_string()),
            salary: row.salary,
            is_active: row.is_active,
            created_at: row.created_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEmployeeRequest {
    #[validate(length(min = 1))]
    pub employee_no: String,
    pub user_id: Option<String>,
    #[validate(length(min = 1))]
    pub first_name: String,
    #[validate(length(min = 1))]
    pub last_name: String,
    pub email: Option<String>,
    #[validate(length(min = 1))]
    pub phone: String,
    #[validate(length(min = 1))]
    pub employment_type: String,
    pub department: Option<String>,
    pub designation: Option<String>,
    pub join_date: Option<String>,
    pub salary: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateEmployeeRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub employment_type: Option<String>,
    pub department: Option<String>,
    pub designation: Option<String>,
    pub join_date: Option<String>,
    pub salary: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollPeriodResponse {
    pub id: String,
    pub period_start: String,
    pub period_end: String,
    pub status: String,
    pub processed_by: Option<String>,
    pub processed_at: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct PayrollPeriodRow {
    pub id: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub status: String,
    pub processed_by: Option<String>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<PayrollPeriodRow> for PayrollPeriodResponse {
    fn from(row: PayrollPeriodRow) -> Self {
        Self {
            id: row.id,
            period_start: row.period_start.to_string(),
            period_end: row.period_end.to_string(),
            status: row.status,
            processed_by: row.processed_by,
            processed_at: row.processed_at.map(|dt| dt.to_rfc3339()),
            created_at: row.created_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePayrollPeriodRequest {
    #[validate(length(min = 1))]
    pub period_start: String,
    #[validate(length(min = 1))]
    pub period_end: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollEntryResponse {
    pub id: String,
    pub period_id: String,
    pub employee_id: String,
    pub basic_salary: String,
    pub allowances: Option<serde_json::Value>,
    pub deductions: Option<serde_json::Value>,
    pub gross_salary: String,
    pub net_salary: String,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct PayrollEntryRow {
    pub id: String,
    pub period_id: String,
    pub employee_id: String,
    pub basic_salary: String,
    pub allowances: Option<serde_json::Value>,
    pub deductions: Option<serde_json::Value>,
    pub gross_salary: String,
    pub net_salary: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<PayrollEntryRow> for PayrollEntryResponse {
    fn from(row: PayrollEntryRow) -> Self {
        Self {
            id: row.id,
            period_id: row.period_id,
            employee_id: row.employee_id,
            basic_salary: row.basic_salary,
            allowances: row.allowances,
            deductions: row.deductions,
            gross_salary: row.gross_salary,
            net_salary: row.net_salary,
            created_at: row.created_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveTypeResponse {
    pub id: String,
    pub name: String,
    pub leave_type: String,
    pub days_per_year: i32,
    pub is_active: bool,
}

#[derive(Debug, FromRow, Serialize)]
pub struct LeaveTypeRow {
    pub id: String,
    pub name: String,
    pub leave_type: String,
    pub days_per_year: i32,
    pub is_active: bool,
}

impl From<LeaveTypeRow> for LeaveTypeResponse {
    fn from(row: LeaveTypeRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            leave_type: row.leave_type,
            days_per_year: row.days_per_year,
            is_active: row.is_active,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLeaveTypeRequest {
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub leave_type: String,
    pub days_per_year: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaveRequestResponse {
    pub id: String,
    pub employee_id: String,
    pub leave_type_id: String,
    pub start_date: String,
    pub end_date: String,
    pub days_count: String,
    pub reason: Option<String>,
    pub status: String,
    pub approved_by: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct LeaveRequestRow {
    pub id: String,
    pub employee_id: String,
    pub leave_type_id: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub days_count: String,
    pub reason: Option<String>,
    pub status: String,
    pub approved_by: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<LeaveRequestRow> for LeaveRequestResponse {
    fn from(row: LeaveRequestRow) -> Self {
        Self {
            id: row.id,
            employee_id: row.employee_id,
            leave_type_id: row.leave_type_id,
            start_date: row.start_date.to_string(),
            end_date: row.end_date.to_string(),
            days_count: row.days_count,
            reason: row.reason,
            status: row.status,
            approved_by: row.approved_by,
            created_at: row.created_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLeaveRequestRequest {
    #[validate(length(min = 1))]
    pub employee_id: String,
    #[validate(length(min = 1))]
    pub leave_type_id: String,
    #[validate(length(min = 1))]
    pub start_date: String,
    #[validate(length(min = 1))]
    pub end_date: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateLeaveRequestRequest {
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceRecordResponse {
    pub id: String,
    pub employee_id: String,
    pub date: String,
    pub clock_in: Option<String>,
    pub clock_out: Option<String>,
    pub hours_worked: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct AttendanceRecordRow {
    pub id: String,
    pub employee_id: String,
    pub date: chrono::NaiveDate,
    pub clock_in: Option<chrono::DateTime<chrono::Utc>>,
    pub clock_out: Option<chrono::DateTime<chrono::Utc>>,
    pub hours_worked: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<AttendanceRecordRow> for AttendanceRecordResponse {
    fn from(row: AttendanceRecordRow) -> Self {
        Self {
            id: row.id,
            employee_id: row.employee_id,
            date: row.date.to_string(),
            clock_in: row.clock_in.map(|dt| dt.to_rfc3339()),
            clock_out: row.clock_out.map(|dt| dt.to_rfc3339()),
            hours_worked: row.hours_worked,
            status: row.status,
            created_at: row.created_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ListQuery {
    pub page: i64,
    pub limit: i64,
    pub search: Option<String>,
    pub status: Option<String>,
    pub employee_id: Option<String>,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
            search: None,
            status: None,
            employee_id: None,
        }
    }
}
