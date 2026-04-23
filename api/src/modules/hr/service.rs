use sqlx::PgPool;
use crate::errors::AppError;
use super::repo::HrRepo;
use super::types::*;
use crate::common::pagination::PaginationMeta;

pub struct HrService;

impl HrService {
    pub async fn list_employees(
        pool: &PgPool,
        query: ListQuery,
    ) -> Result<(Vec<EmployeeResponse>, PaginationMeta), AppError> {
        let repo = HrRepo::new(pool.clone());
        let (rows, total) = repo.list_employees(&query).await?;
        let employees: Vec<EmployeeResponse> = rows.into_iter().map(|r| r.into()).collect();
        let meta = PaginationMeta {
            page: query.page,
            limit: query.limit,
            total,
        };
        Ok((employees, meta))
    }

    pub async fn get_employee(
        pool: &PgPool,
        id: &str,
    ) -> Result<EmployeeResponse, AppError> {
        let repo = HrRepo::new(pool.clone());
        let row = repo.get_employee(id).await?;
        Ok(row.into())
    }

    pub async fn create_employee(
        pool: &PgPool,
        req: CreateEmployeeRequest,
        user_id: Option<&str>,
    ) -> Result<EmployeeResponse, AppError> {
        let repo = HrRepo::new(pool.clone());
        let row = repo.create_employee(&req, user_id).await?;
        Ok(row.into())
    }

    pub async fn update_employee(
        pool: &PgPool,
        id: &str,
        req: UpdateEmployeeRequest,
    ) -> Result<EmployeeResponse, AppError> {
        let repo = HrRepo::new(pool.clone());
        let row = repo.update_employee(id, &req).await?;
        Ok(row.into())
    }

    pub async fn delete_employee(
        pool: &PgPool,
        id: &str,
    ) -> Result<(), AppError> {
        let repo = HrRepo::new(pool.clone());
        repo.delete_employee(id).await?;
        Ok(())
    }

    pub async fn list_payroll_periods(
        pool: &PgPool,
        query: ListQuery,
    ) -> Result<(Vec<PayrollPeriodResponse>, PaginationMeta), AppError> {
        let repo = HrRepo::new(pool.clone());
        let (rows, total) = repo.list_payroll_periods(&query).await?;
        let periods: Vec<PayrollPeriodResponse> = rows.into_iter().map(|r| r.into()).collect();
        let meta = PaginationMeta {
            page: query.page,
            limit: query.limit,
            total,
        };
        Ok((periods, meta))
    }

    pub async fn get_payroll_period(
        pool: &PgPool,
        id: &str,
    ) -> Result<PayrollPeriodResponse, AppError> {
        let repo = HrRepo::new(pool.clone());
        let row = repo.get_payroll_period(id).await?;
        Ok(row.into())
    }

    pub async fn create_payroll_period(
        pool: &PgPool,
        req: CreatePayrollPeriodRequest,
    ) -> Result<PayrollPeriodResponse, AppError> {
        let repo = HrRepo::new(pool.clone());
        let row = repo.create_payroll_period(&req).await?;
        Ok(row.into())
    }

    pub async fn run_payroll(
        pool: &PgPool,
        period_id: &str,
    ) -> Result<Vec<PayrollEntryResponse>, AppError> {
        let repo = HrRepo::new(pool.clone());
        let period = repo.get_payroll_period(period_id).await?;
        if period.status != "OPEN" {
            return Err(AppError::Validation("Period is not open".to_string()));
        }
        let employees = repo.get_active_employees().await?;
        let mut entries = Vec::new();
        for emp in employees {
            let basic_salary = emp.salary.unwrap_or_else(|| "0".to_string());
            let gross_salary = basic_salary.clone();
            let net_salary = basic_salary.clone();
            let req = super::repo::CreatePayrollEntryRequest {
                period_id: period_id.to_string(),
                employee_id: emp.id,
                basic_salary,
                allowances: Some(serde_json::json!([{"name": "Basic", "amount": gross_salary}])),
                deductions: Some(serde_json::json!([{"name": "Tax", "amount": gross_salary}])),
                gross_salary,
                net_salary,
            };
            let entry = repo.create_payroll_entry(&req).await?;
            entries.push(entry.into());
        }
        Ok(entries)
    }

    pub async fn list_payroll_entries(
        pool: &PgPool,
        period_id: &str,
    ) -> Result<Vec<PayrollEntryResponse>, AppError> {
        let repo = HrRepo::new(pool.clone());
        let rows = repo.list_payroll_entries(period_id).await?;
        let entries: Vec<PayrollEntryResponse> = rows.into_iter().map(|r| r.into()).collect();
        Ok(entries)
    }

    pub async fn list_leave_types(
        pool: &PgPool,
    ) -> Result<Vec<LeaveTypeResponse>, AppError> {
        let repo = HrRepo::new(pool.clone());
        let rows = repo.list_leave_types().await?;
        let types: Vec<LeaveTypeResponse> = rows.into_iter().map(|r| r.into()).collect();
        Ok(types)
    }

    pub async fn create_leave_type(
        pool: &PgPool,
        req: CreateLeaveTypeRequest,
    ) -> Result<LeaveTypeResponse, AppError> {
        let repo = HrRepo::new(pool.clone());
        let row = repo.create_leave_type(&req).await?;
        Ok(row.into())
    }

    pub async fn list_leave_requests(
        pool: &PgPool,
        query: ListQuery,
    ) -> Result<(Vec<LeaveRequestResponse>, PaginationMeta), AppError> {
        let repo = HrRepo::new(pool.clone());
        let (rows, total) = repo.list_leave_requests(&query).await?;
        let requests: Vec<LeaveRequestResponse> = rows.into_iter().map(|r| r.into()).collect();
        let meta = PaginationMeta {
            page: query.page,
            limit: query.limit,
            total,
        };
        Ok((requests, meta))
    }

    pub async fn create_leave_request(
        pool: &PgPool,
        req: CreateLeaveRequestRequest,
    ) -> Result<LeaveRequestResponse, AppError> {
        let repo = HrRepo::new(pool.clone());
        let row = repo.create_leave_request(&req).await?;
        Ok(row.into())
    }

    pub async fn update_leave_request(
        pool: &PgPool,
        id: &str,
        status: &str,
        approved_by: &str,
    ) -> Result<LeaveRequestResponse, AppError> {
        let repo = HrRepo::new(pool.clone());
        let row = repo.update_leave_request(id, status, approved_by).await?;
        Ok(row.into())
    }

    pub async fn list_attendance(
        pool: &PgPool,
        query: ListQuery,
    ) -> Result<(Vec<AttendanceRecordResponse>, PaginationMeta), AppError> {
        let repo = HrRepo::new(pool.clone());
        let (rows, total) = repo.list_attendance(&query).await?;
        let records: Vec<AttendanceRecordResponse> = rows.into_iter().map(|r| r.into()).collect();
        let meta = PaginationMeta {
            page: query.page,
            limit: query.limit,
            total,
        };
        Ok((records, meta))
    }

    pub async fn clock_in(
        pool: &PgPool,
        employee_id: &str,
    ) -> Result<AttendanceRecordResponse, AppError> {
        let repo = HrRepo::new(pool.clone());
        let row = repo.clock_in(employee_id).await?;
        Ok(row.into())
    }

    pub async fn clock_out(
        pool: &PgPool,
        employee_id: &str,
    ) -> Result<AttendanceRecordResponse, AppError> {
        let repo = HrRepo::new(pool.clone());
        let row = repo.clock_out(employee_id).await?;
        Ok(row.into())
    }
}