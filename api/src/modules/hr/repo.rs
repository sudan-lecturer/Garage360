use sqlx::{Pool, Postgres};
use crate::errors::AppError;
use super::types::*;

pub struct HrRepo {
    pool: Pool<Postgres>,
}

impl HrRepo {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn list_employees(&self, query: &ListQuery) -> Result<(Vec<EmployeeRow>, i64), AppError> {
        let offset = (query.page - 1) * query.limit;
        let mut sql = "SELECT id, employee_no, user_id, first_name, last_name, email, phone, employment_type, department, designation, join_date, salary, is_active, created_at FROM employees WHERE is_active = true".to_string();
        let mut count_sql = "SELECT COUNT(*) FROM employees WHERE is_active = true".to_string();

        if let Some(ref search) = query.search {
            let search_filter = format!(" AND (first_name ILIKE '%{}%' OR last_name ILIKE '%{}%' OR employee_no ILIKE '%{}%' OR phone ILIKE '%{}%')", search, search, search, search);
            sql.push_str(&search_filter);
            count_sql.push_str(&search_filter);
        }

        sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", query.limit, offset));

        let rows = sqlx::query_as::<_, EmployeeRow>(&sql).fetch_all(&self.pool).await?;
        let total = sqlx::query_scalar::<_, i64>(&count_sql).fetch_one(&self.pool).await?;

        Ok((rows, total))
    }

    pub async fn get_employee(&self, id: &str) -> Result<EmployeeRow, AppError> {
        let row = sqlx::query_as::<_, EmployeeRow>(
            "SELECT id, employee_no, user_id, first_name, last_name, email, phone, employment_type, department, designation, join_date, salary, is_active, created_at FROM employees WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_employee(&self, req: &CreateEmployeeRequest, user_id: Option<&str>) -> Result<EmployeeRow, AppError> {
        let join_date = req.join_date.as_ref().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

        let row = sqlx::query_as::<_, EmployeeRow>(
            "INSERT INTO employees (employee_no, user_id, first_name, last_name, email, phone, employment_type, department, designation, join_date, salary) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING id, employee_no, user_id, first_name, last_name, email, phone, employment_type, department, designation, join_date, salary, is_active, created_at"
        )
        .bind(&req.employee_no)
        .bind(user_id)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.email)
        .bind(&req.phone)
        .bind(&req.employment_type)
        .bind(&req.department)
        .bind(&req.designation)
        .bind(join_date)
        .bind(&req.salary)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update_employee(&self, id: &str, req: &UpdateEmployeeRequest) -> Result<EmployeeRow, AppError> {
        let join_date = req.join_date.as_ref().and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

        let row = sqlx::query_as::<_, EmployeeRow>(
            "UPDATE employees SET first_name = COALESCE($2, first_name), last_name = COALESCE($3, last_name), email = COALESCE($4, email), phone = COALESCE($5, phone), employment_type = COALESCE($6, employment_type), department = COALESCE($7, department), designation = COALESCE($8, designation), join_date = COALESCE($9, join_date), salary = COALESCE($10, salary), is_active = COALESCE($11, is_active), updated_at = NOW() WHERE id = $1 RETURNING id, employee_no, user_id, first_name, last_name, email, phone, employment_type, department, designation, join_date, salary, is_active, created_at"
        )
        .bind(id)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.email)
        .bind(&req.phone)
        .bind(&req.employment_type)
        .bind(&req.department)
        .bind(&req.designation)
        .bind(join_date)
        .bind(&req.salary)
        .bind(req.is_active)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn delete_employee(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE employees SET is_active = false WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_payroll_periods(&self, query: &ListQuery) -> Result<(Vec<PayrollPeriodRow>, i64), AppError> {
        let offset = (query.page - 1) * query.limit;
        let sql = format!(
            "SELECT id, period_start, period_end, status, processed_by, processed_at, created_at FROM payroll_periods ORDER BY created_at DESC LIMIT {} OFFSET {}",
            query.limit, offset
        );
        let count_sql = "SELECT COUNT(*) FROM payroll_periods";

        let rows = sqlx::query_as::<_, PayrollPeriodRow>(&sql).fetch_all(&self.pool).await?;
        let total = sqlx::query_scalar::<_, i64>(count_sql).fetch_one(&self.pool).await?;

        Ok((rows, total))
    }

    pub async fn get_payroll_period(&self, id: &str) -> Result<PayrollPeriodRow, AppError> {
        let row = sqlx::query_as::<_, PayrollPeriodRow>(
            "SELECT id, period_start, period_end, status, processed_by, processed_at, created_at FROM payroll_periods WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_payroll_period(&self, req: &CreatePayrollPeriodRequest) -> Result<PayrollPeriodRow, AppError> {
        let period_start = chrono::NaiveDate::parse_from_str(&req.period_start, "%Y-%m-%d")
            .map_err(|_| AppError::Validation("Invalid period_start format".to_string()))?;
        let period_end = chrono::NaiveDate::parse_from_str(&req.period_end, "%Y-%m-%d")
            .map_err(|_| AppError::Validation("Invalid period_end format".to_string()))?;

        let row = sqlx::query_as::<_, PayrollPeriodRow>(
            "INSERT INTO payroll_periods (period_start, period_end) VALUES ($1, $2) RETURNING id, period_start, period_end, status, processed_by, processed_at, created_at"
        )
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn close_payroll_period(&self, id: &str, user_id: &str) -> Result<PayrollPeriodRow, AppError> {
        let row = sqlx::query_as::<_, PayrollPeriodRow>(
            "UPDATE payroll_periods SET status = 'CLOSED', processed_by = $2, processed_at = NOW() WHERE id = $1 RETURNING id, period_start, period_end, status, processed_by, processed_at, created_at"
        )
        .bind(id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_payroll_entries(&self, period_id: &str) -> Result<Vec<PayrollEntryRow>, AppError> {
        let rows = sqlx::query_as::<_, PayrollEntryRow>(
            "SELECT id, period_id, employee_id, basic_salary, allowances, deductions, gross_salary, net_salary, created_at FROM payroll_entries WHERE period_id = $1"
        )
        .bind(period_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_employee_payroll(&self, employee_id: &str, period_id: &str) -> Result<Option<PayrollEntryRow>, AppError> {
        let row = sqlx::query_as::<_, PayrollEntryRow>(
            "SELECT id, period_id, employee_id, basic_salary, allowances, deductions, gross_salary, net_salary, created_at FROM payroll_entries WHERE employee_id = $1 AND period_id = $2"
        )
        .bind(employee_id)
        .bind(period_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_payroll_entry(&self, req: &CreatePayrollEntryRequest) -> Result<PayrollEntryRow, AppError> {
        let row = sqlx::query_as::<_, PayrollEntryRow>(
            "INSERT INTO payroll_entries (period_id, employee_id, basic_salary, allowances, deductions, gross_salary, net_salary) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id, period_id, employee_id, basic_salary, allowances, deductions, gross_salary, net_salary, created_at"
        )
        .bind(&req.period_id)
        .bind(&req.employee_id)
        .bind(&req.basic_salary)
        .bind(&req.allowances)
        .bind(&req.deductions)
        .bind(&req.gross_salary)
        .bind(&req.net_salary)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_leave_types(&self) -> Result<Vec<LeaveTypeRow>, AppError> {
        let rows = sqlx::query_as::<_, LeaveTypeRow>(
            "SELECT id, name, leave_type, days_per_year, is_active FROM leave_types WHERE is_active = true"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn create_leave_type(&self, req: &CreateLeaveTypeRequest) -> Result<LeaveTypeRow, AppError> {
        let days_per_year = req.days_per_year.unwrap_or(0);
        let row = sqlx::query_as::<_, LeaveTypeRow>(
            "INSERT INTO leave_types (name, leave_type, days_per_year) VALUES ($1, $2, $3) RETURNING id, name, leave_type, days_per_year, is_active"
        )
        .bind(&req.name)
        .bind(&req.leave_type)
        .bind(days_per_year)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_leave_requests(&self, query: &ListQuery) -> Result<(Vec<LeaveRequestRow>, i64), AppError> {
        let offset = (query.page - 1) * query.limit;
        let mut sql = "SELECT id, employee_id, leave_type_id, start_date, end_date, days_count, reason, status, approved_by, created_at FROM leave_requests".to_string();
        let mut count_sql = "SELECT COUNT(*) FROM leave_requests".to_string();

        if let Some(ref employee_id) = query.employee_id {
            let filter = format!(" WHERE employee_id = '{}'", employee_id);
            sql.push_str(&filter);
            count_sql.push_str(&filter);
        } else if let Some(ref status) = query.status {
            let cond = if sql.contains("WHERE") { " AND" } else { " WHERE" };
            sql.push_str(&format!("{} status = '{}'", cond, status));
            count_sql.push_str(&format!("{} status = '{}'", cond, status));
        }

        sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", query.limit, offset));

        let rows = sqlx::query_as::<_, LeaveRequestRow>(&sql).fetch_all(&self.pool).await?;
        let total = sqlx::query_scalar::<_, i64>(&count_sql).fetch_one(&self.pool).await?;

        Ok((rows, total))
    }

    pub async fn update_leave_request(&self, id: &str, status: &str, approved_by: &str) -> Result<LeaveRequestRow, AppError> {
        let row = sqlx::query_as::<_, LeaveRequestRow>(
            "UPDATE leave_requests SET status = $2, approved_by = $3 WHERE id = $1 RETURNING id, employee_id, leave_type_id, start_date, end_date, days_count, reason, status, approved_by, created_at"
        )
        .bind(id)
        .bind(status)
        .bind(approved_by)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn create_leave_request(&self, req: &CreateLeaveRequestRequest) -> Result<LeaveRequestRow, AppError> {
        let start_date = chrono::NaiveDate::parse_from_str(&req.start_date, "%Y-%m-%d")
            .map_err(|_| AppError::Validation("Invalid start_date format".to_string()))?;
        let end_date = chrono::NaiveDate::parse_from_str(&req.end_date, "%Y-%m-%d")
            .map_err(|_| AppError::Validation("Invalid end_date format".to_string()))?;
        let days_count: f64 = (end_date - start_date).num_days() as f64 + 1.0;
        let days_count_str = days_count.to_string();

        let row = sqlx::query_as::<_, LeaveRequestRow>(
            "INSERT INTO leave_requests (employee_id, leave_type_id, start_date, end_date, days_count, reason) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, employee_id, leave_type_id, start_date, end_date, days_count, reason, status, approved_by, created_at"
        )
        .bind(&req.employee_id)
        .bind(&req.leave_type_id)
        .bind(start_date)
        .bind(end_date)
        .bind(days_count_str)
        .bind(&req.reason)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_attendance(&self, query: &ListQuery) -> Result<(Vec<AttendanceRecordRow>, i64), AppError> {
        let offset = (query.page - 1) * query.limit;
        let mut sql = "SELECT id, employee_id, date, clock_in, clock_out, hours_worked, status, created_at FROM attendance_records".to_string();
        let mut count_sql = "SELECT COUNT(*) FROM attendance_records".to_string();

        if let Some(ref employee_id) = query.employee_id {
            let cond = if sql.contains("WHERE") { " AND" } else { " WHERE" };
            sql.push_str(&format!("{} employee_id = '{}'", cond, employee_id));
            count_sql.push_str(&format!("{} employee_id = '{}'", cond, employee_id));
        }

        sql.push_str(&format!(" ORDER BY date DESC, clock_in DESC LIMIT {} OFFSET {}", query.limit, offset));

        let rows = sqlx::query_as::<_, AttendanceRecordRow>(&sql).fetch_all(&self.pool).await?;
        let total = sqlx::query_scalar::<_, i64>(&count_sql).fetch_one(&self.pool).await?;

        Ok((rows, total))
    }

    pub async fn clock_in(&self, employee_id: &str) -> Result<AttendanceRecordRow, AppError> {
        let today = chrono::Utc::now().date_naive();

        let existing = sqlx::query_as::<_, AttendanceRecordRow>(
            "SELECT id, employee_id, date, clock_in, clock_out, hours_worked, status, created_at FROM attendance_records WHERE employee_id = $1 AND date = $2"
        )
        .bind(employee_id)
        .bind(today)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Err(AppError::Conflict("Already clocked in today".to_string()));
        }

        let row = sqlx::query_as::<_, AttendanceRecordRow>(
            "INSERT INTO attendance_records (employee_id, date, clock_in, status) VALUES ($1, $2, NOW(), 'PRESENT') RETURNING id, employee_id, date, clock_in, clock_out, hours_worked, status, created_at"
        )
        .bind(employee_id)
        .bind(today)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn clock_out(&self, employee_id: &str) -> Result<AttendanceRecordRow, AppError> {
        let today = chrono::Utc::now().date_naive();

        let row = sqlx::query_as::<_, AttendanceRecordRow>(
            "UPDATE attendance_records SET clock_out = NOW(), hours_worked = EXTRACT(EPOCH FROM (NOW() - clock_in)) / 3600 WHERE employee_id = $1 AND date = $2 AND clock_out IS NULL RETURNING id, employee_id, date, clock_in, clock_out, hours_worked, status, created_at"
        )
        .bind(employee_id)
        .bind(today)
        .fetch_one(&self.pool)
        .await?;

        if row.clock_out.is_none() {
            return Err(AppError::NotFound("No clock-in record found for today".to_string()));
        }

        Ok(row)
    }

    pub async fn get_active_employees(&self) -> Result<Vec<EmployeeRow>, AppError> {
        let rows = sqlx::query_as::<_, EmployeeRow>(
            "SELECT id, employee_no, user_id, first_name, last_name, email, phone, employment_type, department, designation, join_date, salary, is_active, created_at FROM employees WHERE is_active = true ORDER BY first_name, last_name"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }
}

pub struct CreatePayrollEntryRequest {
    pub period_id: String,
    pub employee_id: String,
    pub basic_salary: String,
    pub allowances: Option<serde_json::Value>,
    pub deductions: Option<serde_json::Value>,
    pub gross_salary: String,
    pub net_salary: String,
}