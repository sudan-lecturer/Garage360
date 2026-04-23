use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;
use crate::common::pagination::PaginationMeta;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct VehicleResponse {
    pub id: String,
    pub customer_id: String,
    pub registration_no: String,
    pub make: String,
    pub model: String,
    pub year: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct VehicleRequest {
    #[validate(length(min = 1))]
    pub customer_id: String,
    #[validate(length(min = 1))]
    pub registration_no: String,
    #[validate(length(min = 1))]
    pub make: String,
    #[validate(length(min = 1))]
    pub model: String,
    pub year: Option<i32>,
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
