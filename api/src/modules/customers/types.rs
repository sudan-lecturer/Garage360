use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;
use crate::common::pagination::PaginationMeta;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerResponse {
    pub id: String,
    pub customer_type: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: String,
    pub address: Option<String>,
    pub created_at: Option<String>,
    pub name: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct CustomerRow {
    pub id: String,
    pub customer_type: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company_name: Option<String>,
    pub email: Option<String>,
    pub phone: String,
    pub address: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<CustomerRow> for CustomerResponse {
    fn from(row: CustomerRow) -> Self {
        let name = if row.customer_type == "ORGANIZATION" {
            row.company_name.as_deref().unwrap_or_default().to_string()
        } else {
            match (&row.first_name, &row.last_name) {
                (Some(first), Some(last)) => format!("{} {}", first, last),
                (Some(first), None) => first.clone(),
                (None, Some(last)) => last.clone(),
                (None, None) => String::new(),
            }
        };

        Self {
            id: row.id,
            customer_type: row.customer_type,
            first_name: row.first_name,
            last_name: row.last_name,
            company_name: row.company_name,
            email: row.email,
            phone: row.phone,
            address: row.address,
            created_at: row.created_at.map(|date| date.to_rfc3339()),
            name,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCustomerRequest {
    #[validate(length(min = 1))]
    pub customer_type: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company_name: Option<String>,
    pub email: Option<String>,
    #[validate(length(min = 1))]
    pub phone: String,
    pub address: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ListQuery {
    pub page: i64,
    pub limit: i64,
    pub search: Option<String>,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
            search: None,
        }
    }
}
