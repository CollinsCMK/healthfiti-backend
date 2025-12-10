use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub search: Option<String>,
    pub all: Option<bool>,
    pub selector: Option<bool>,
    pub columns: Option<String>,
    pub filename: Option<String>,
    pub expiration: Option<bool>,
}
