use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRequest {
    pub new_rows: Vec<HashMap<String, serde_json::Value>>,
    pub edited_rows: Vec<EditedRow>,
    pub deleted_rows: Vec<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditedRow {
    pub original_data: HashMap<String, serde_json::Value>,
    pub updated_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveResponse {
    pub status: String, // "success" | "partial" | "error"
    pub message: String,
    pub affected_rows: i64,
    pub executed_queries: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

impl SaveResponse {
    pub fn success(message: String, affected_rows: i64, executed_queries: Vec<String>) -> Self {
        SaveResponse {
            status: "success".to_string(),
            message,
            affected_rows,
            executed_queries,
            errors: None,
        }
    }

    pub fn partial(
        message: String,
        affected_rows: i64,
        executed_queries: Vec<String>,
        errors: Vec<String>,
    ) -> Self {
        SaveResponse {
            status: "partial".to_string(),
            message,
            affected_rows,
            executed_queries,
            errors: Some(errors),
        }
    }

    pub fn error(message: String) -> Self {
        SaveResponse {
            status: "error".to_string(),
            message,
            affected_rows: 0,
            executed_queries: vec![],
            errors: None,
        }
    }
}
