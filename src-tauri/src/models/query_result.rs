use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub rows_affected: Option<u64>,
    pub execution_time: u128,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct FilterRequest {
    pub column: String,
    pub search_query: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterValuesResult {
    pub values: Vec<String>,
    pub total_count: usize,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryError {
    pub message: String,
    pub code: Option<String>,
}
