use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

pub fn default_true() -> bool {
    true
}

pub fn default_timezone() -> String {
    "Asia/Kathmandu".to_string()
}

pub fn default_currency_code() -> String {
    "NPR".to_string()
}

pub fn default_currency_symbol() -> String {
    "Rs.".to_string()
}

pub fn default_job_updates_email() -> bool {
    true
}

pub fn default_low_stock_alerts() -> bool {
    true
}

pub fn default_approval_alerts() -> bool {
    true
}

pub fn default_push_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkshopProfileResponse {
    pub name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub logo_url: Option<String>,
    pub tax_id: Option<String>,
    pub timezone: String,
    pub currency_code: String,
    pub currency_symbol: String,
}

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkshopProfileRequest {
    pub name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    #[validate(email(message = "Invalid email"))]
    pub email: Option<String>,
    pub logo_url: Option<String>,
    pub tax_id: Option<String>,
    pub timezone: Option<String>,
    pub currency_code: Option<String>,
    pub currency_symbol: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationResponse {
    pub id: String,
    pub name: String,
    pub address: Option<String>,
    pub is_primary: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow)]
pub struct LocationRow {
    pub id: String,
    pub name: String,
    pub address: Option<String>,
    pub is_primary: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<LocationRow> for LocationResponse {
    fn from(row: LocationRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            address: row.address,
            is_primary: row.is_primary,
            is_active: row.is_active,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocationRequest {
    #[validate(length(min = 1, message = "Location name is required"))]
    pub name: String,
    pub address: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRow> for UserResponse {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            name: row.name,
            role: row.role,
            is_active: row.is_active,
            last_login_at: row.last_login_at.map(|value| value.to_rfc3339()),
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email"))]
    #[validate(length(min = 1, message = "Email is required"))]
    pub email: String,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    #[validate(length(min = 1, message = "Role is required"))]
    pub role: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    #[validate(email(message = "Invalid email"))]
    #[validate(length(min = 1, message = "Email is required"))]
    pub email: String,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    #[validate(length(min = 1, message = "Role is required"))]
    pub role: String,
    pub password: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct IntakeTemplateItem {
    #[validate(length(min = 1, message = "Item key is required"))]
    pub key: String,
    #[validate(length(min = 1, message = "Item label is required"))]
    pub label: String,
    #[validate(length(min = 1, message = "Item inputType is required"))]
    pub input_type: String,
    #[serde(default)]
    pub required: bool,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpsertIntakeTemplateRequest {
    #[validate(length(min = 1, message = "Template name is required"))]
    pub name: String,
    pub items: Vec<IntakeTemplateItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntakeTemplateResponse {
    pub id: Option<String>,
    pub name: String,
    pub items: Vec<IntakeTemplateItem>,
    pub is_active: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, FromRow)]
pub struct IntakeTemplateRow {
    pub id: String,
    pub name: String,
    pub items: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlagResponse {
    pub key: String,
    pub description: Option<String>,
    pub default_enabled: bool,
    pub is_enabled: bool,
    pub has_override: bool,
}

#[derive(Debug, FromRow)]
pub struct FeatureFlagRow {
    pub key: String,
    pub description: Option<String>,
    pub default_enabled: bool,
    pub is_enabled: bool,
    pub has_override: bool,
}

impl From<FeatureFlagRow> for FeatureFlagResponse {
    fn from(row: FeatureFlagRow) -> Self {
        Self {
            key: row.key,
            description: row.description,
            default_enabled: row.default_enabled,
            is_enabled: row.is_enabled,
            has_override: row.has_override,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlagUpdateRequest {
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPreferences {
    #[serde(default = "default_job_updates_email")]
    pub job_updates_email: bool,
    #[serde(default)]
    pub job_updates_sms: bool,
    #[serde(default = "default_push_enabled")]
    pub push_enabled: bool,
    #[serde(default = "default_low_stock_alerts")]
    pub low_stock_alerts: bool,
    #[serde(default = "default_approval_alerts")]
    pub approval_alerts: bool,
    #[validate(email(message = "Invalid daily summary email"))]
    pub daily_summary_email: Option<String>,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            job_updates_email: default_job_updates_email(),
            job_updates_sms: false,
            push_enabled: default_push_enabled(),
            low_stock_alerts: default_low_stock_alerts(),
            approval_alerts: default_approval_alerts(),
            daily_summary_email: None,
        }
    }
}
