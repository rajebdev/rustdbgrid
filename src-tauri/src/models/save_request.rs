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
    pub inserted_rows: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_rows: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_rows: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
}

impl SaveResponse {
    pub fn success_with_counts(
        message: String,
        affected_rows: i64,
        executed_queries: Vec<String>,
        inserted_rows: i64,
        updated_rows: i64,
        deleted_rows: i64,
    ) -> Self {
        SaveResponse {
            status: "success".to_string(),
            message,
            affected_rows,
            executed_queries,
            inserted_rows: Some(inserted_rows),
            updated_rows: Some(updated_rows),
            deleted_rows: Some(deleted_rows),
            errors: None,
        }
    }

    pub fn partial_with_counts(
        message: String,
        affected_rows: i64,
        executed_queries: Vec<String>,
        inserted_rows: i64,
        updated_rows: i64,
        deleted_rows: i64,
        errors: Vec<String>,
    ) -> Self {
        SaveResponse {
            status: "partial".to_string(),
            message,
            affected_rows,
            executed_queries,
            inserted_rows: Some(inserted_rows),
            updated_rows: Some(updated_rows),
            deleted_rows: Some(deleted_rows),
            errors: Some(errors),
        }
    }

    pub fn error(message: String) -> Self {
        SaveResponse {
            status: "error".to_string(),
            message,
            affected_rows: 0,
            executed_queries: vec![],
            inserted_rows: None,
            updated_rows: None,
            deleted_rows: None,
            errors: None,
        }
    }
}
