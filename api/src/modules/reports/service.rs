use chrono::{DateTime, NaiveDate, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

use super::{
    repo,
    types::{
        ExportReportRequest, GenerateReportRequest, SaveReportRequest, SavedReportResponse,
    },
};

const REPORT_REVENUE_SUMMARY: &str = "revenue_summary";
const REPORT_JOB_CARD_ANALYSIS: &str = "job_card_analysis";
const REPORT_CUSTOMER_ACTIVITY: &str = "customer_activity";

struct ReportFilters {
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
    limit: i64,
}

impl ReportFilters {
    fn as_json(&self) -> Value {
        json!({
            "dateFrom": self.date_from.map(|value| value.to_rfc3339()),
            "dateTo": self.date_to.map(|value| value.to_rfc3339()),
            "limit": self.limit,
        })
    }
}

pub async fn generate_report(
    pool: &PgPool,
    req: &GenerateReportRequest,
) -> AppResult<serde_json::Value> {
    let report_type = normalize_report_type(&req.report_type);
    let filters = parse_filters(req.config.as_ref())?;
    let data = build_report_data(pool, &report_type, &filters).await?;

    Ok(json!({
        "reportType": report_type,
        "generatedAt": Utc::now().to_rfc3339(),
        "filters": filters.as_json(),
        "data": data,
    }))
}

pub async fn export_report(
    pool: &PgPool,
    req: &ExportReportRequest,
) -> AppResult<serde_json::Value> {
    let report_type = normalize_report_type(&req.report_type);
    let filters = parse_filters(req.config.as_ref())?;
    let format = normalize_export_format(req.format.as_deref())?;
    let data = build_report_data(pool, &report_type, &filters).await?;

    Ok(json!({
        "reportType": report_type,
        "format": format,
        "exportedAt": Utc::now().to_rfc3339(),
        "report": {
            "filters": filters.as_json(),
            "data": data,
        }
    }))
}

pub async fn list_saved_reports(
    pool: &PgPool,
    created_by: &str,
) -> AppResult<Vec<SavedReportResponse>> {
    ensure_uuid(created_by, "userId")?;

    let reports = repo::list_saved_reports(pool, created_by).await?;
    Ok(reports.into_iter().map(SavedReportResponse::from).collect())
}

pub async fn create_saved_report(
    pool: &PgPool,
    req: &SaveReportRequest,
    created_by: &str,
) -> AppResult<SavedReportResponse> {
    ensure_uuid(created_by, "userId")?;

    let report_type = normalize_report_type(&req.report_type);
    validate_report_type(&report_type)?;
    validate_report_config(&req.config)?;

    repo::create_saved_report(pool, req.name.trim(), &report_type, &req.config, created_by)
        .await
        .map(SavedReportResponse::from)
}

pub async fn delete_saved_report(
    pool: &PgPool,
    id: &str,
    created_by: &str,
) -> AppResult<serde_json::Value> {
    ensure_uuid(id, "savedReportId")?;
    ensure_uuid(created_by, "userId")?;

    let rows_affected = repo::delete_saved_report(pool, id, created_by).await?;
    if rows_affected == 0 {
        return Err(AppError::NotFound("Saved report not found".into()));
    }

    Ok(json!({ "deleted": true }))
}

async fn build_report_data(
    pool: &PgPool,
    report_type: &str,
    filters: &ReportFilters,
) -> AppResult<serde_json::Value> {
    validate_report_type(report_type)?;

    match report_type {
        REPORT_REVENUE_SUMMARY => build_revenue_summary(pool, filters).await,
        REPORT_JOB_CARD_ANALYSIS => build_job_card_analysis(pool, filters).await,
        REPORT_CUSTOMER_ACTIVITY => build_customer_activity(pool, filters).await,
        _ => Err(AppError::Validation(format!(
            "Unsupported report type. Supported report types: {}",
            supported_report_types_message()
        ))),
    }
}

async fn build_revenue_summary(
    pool: &PgPool,
    filters: &ReportFilters,
) -> AppResult<serde_json::Value> {
    let summary = repo::fetch_revenue_totals(pool, filters.date_from, filters.date_to).await?;
    let breakdown =
        repo::fetch_revenue_status_breakdown(pool, filters.date_from, filters.date_to).await?;

    Ok(json!({
        "summary": summary,
        "statusBreakdown": breakdown,
    }))
}

async fn build_job_card_analysis(
    pool: &PgPool,
    filters: &ReportFilters,
) -> AppResult<serde_json::Value> {
    let summary = repo::fetch_job_summary(pool, filters.date_from, filters.date_to).await?;
    let status_breakdown =
        repo::fetch_job_status_counts(pool, filters.date_from, filters.date_to).await?;
    let mechanic_breakdown =
        repo::fetch_mechanic_job_counts(pool, filters.date_from, filters.date_to, filters.limit)
            .await?;

    Ok(json!({
        "summary": summary,
        "statusBreakdown": status_breakdown,
        "mechanicBreakdown": mechanic_breakdown,
    }))
}

async fn build_customer_activity(
    pool: &PgPool,
    filters: &ReportFilters,
) -> AppResult<serde_json::Value> {
    let customers =
        repo::fetch_customer_activity(pool, filters.date_from, filters.date_to, filters.limit)
            .await?;

    Ok(json!({
        "customers": customers,
    }))
}

fn parse_filters(config: Option<&Value>) -> AppResult<ReportFilters> {
    let config = match config {
        Some(value) => {
            validate_report_config(value)?;
            value
        }
        None => &Value::Null,
    };

    let date_from = get_config_value(config, "date_from", "dateFrom")
        .map(|value| parse_filter_datetime(value, "config.date_from", false))
        .transpose()?;
    let date_to = get_config_value(config, "date_to", "dateTo")
        .map(|value| parse_filter_datetime(value, "config.date_to", true))
        .transpose()?;

    if let (Some(date_from), Some(date_to)) = (date_from, date_to) {
        if date_from > date_to {
            return Err(AppError::Validation(
                "config.date_from must be before or equal to config.date_to".into(),
            ));
        }
    }

    let limit = config
        .get("limit")
        .and_then(Value::as_i64)
        .unwrap_or(10)
        .clamp(1, 100);

    Ok(ReportFilters {
        date_from,
        date_to,
        limit,
    })
}

fn validate_report_type(report_type: &str) -> AppResult<()> {
    match report_type {
        REPORT_REVENUE_SUMMARY | REPORT_JOB_CARD_ANALYSIS | REPORT_CUSTOMER_ACTIVITY => Ok(()),
        _ => Err(AppError::Validation(format!(
            "Unsupported report type. Supported report types: {}",
            supported_report_types_message()
        ))),
    }
}

fn validate_report_config(config: &Value) -> AppResult<()> {
    if !config.is_object() {
        return Err(AppError::Validation(
            "config must be a JSON object".into(),
        ));
    }

    Ok(())
}

fn normalize_report_type(report_type: &str) -> String {
    report_type.trim().to_ascii_lowercase().replace('-', "_")
}

fn normalize_export_format(format: Option<&str>) -> AppResult<String> {
    let value = format.unwrap_or("json").trim().to_ascii_lowercase();

    match value.as_str() {
        "json" | "csv" | "xlsx" | "excel" => Ok(if value == "excel" {
            "xlsx".to_string()
        } else {
            value
        }),
        _ => Err(AppError::Validation(
            "Unsupported export format. Supported formats: json, csv, xlsx".into(),
        )),
    }
}

fn get_config_value<'a>(config: &'a Value, snake_case: &str, camel_case: &str) -> Option<&'a str> {
    config
        .get(snake_case)
        .or_else(|| config.get(camel_case))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn parse_filter_datetime(
    value: &str,
    field: &str,
    end_of_day: bool,
) -> AppResult<DateTime<Utc>> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Ok(parsed.with_timezone(&Utc));
    }

    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let naive = if end_of_day {
            date.and_hms_opt(23, 59, 59)
        } else {
            date.and_hms_opt(0, 0, 0)
        };
        let Some(naive) = naive else {
            return Err(AppError::Validation(format!(
                "{field} must be a valid date"
            )));
        };

        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }

    Err(AppError::Validation(format!(
        "{field} must be an RFC3339 timestamp or YYYY-MM-DD date"
    )))
}

fn supported_report_types_message() -> &'static str {
    "revenue_summary, job_card_analysis, customer_activity"
}

fn ensure_uuid(value: &str, field: &str) -> AppResult<()> {
    Uuid::parse_str(value)
        .map(|_| ())
        .map_err(|_| AppError::Validation(format!("{field} must be a valid UUID")))
}
