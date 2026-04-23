use serde::Serialize;

/// Simple pagination metadata returned alongside list results.
#[derive(Debug, Serialize, Clone, Copy)]
pub struct PaginationMeta {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}
