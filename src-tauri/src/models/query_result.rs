use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_display_names: Option<Vec<String>>, // Original column names for display (without suffix)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_types: Option<HashMap<String, String>>, // Map<column_name, data_type>
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub rows_affected: Option<u64>,
    pub execution_time: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_query: Option<String>,
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
