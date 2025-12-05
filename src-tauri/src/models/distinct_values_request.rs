use crate::models::connection::DatabaseType;
use crate::models::table_request::Filter;
use serde::{Deserialize, Serialize};

/// Request structure for loading distinct column values with optional filtering and search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistinctValuesRequest {
    pub connection_id: String,
    pub query: DistinctValuesQuery,
}

/// Query details for distinct values operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistinctValuesQuery {
    pub db_type: DatabaseType,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub table: String,
    pub column: String,
    pub search_term: Option<String>,
    pub limit: usize,
    #[serde(default)]
    pub filters: Vec<Filter>,
}

/// Response structure for distinct values
#[derive(Debug, Serialize, Deserialize)]
pub struct DistinctValuesResponse {
    pub values: Vec<String>,
    pub total_count: usize,
    pub execution_time: u128,
    pub query_used: String,
}
