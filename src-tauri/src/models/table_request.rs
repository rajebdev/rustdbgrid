use crate::models::connection::DatabaseType;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Request structure for loading table data with filters, sorting, and pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDataRequest {
    pub connection_id: String,
    pub query: QueryRequest,
}

/// Query details including database type and all query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub db_type: DatabaseType,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub table: String,
    pub limit: usize,
    pub offset: usize,
    #[serde(default)]
    pub filters: Vec<Filter>,
    #[serde(default)]
    pub order_by: Vec<OrderBy>,
}

/// Filter configuration for a column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    pub column: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

/// Filter operators supported across different database types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    In,
    NotIn,
    Like,
    NotLike,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Between,
    IsNull,
    IsNotNull,
}

/// Filter value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    Single(serde_json::Value),
    Multiple(Vec<serde_json::Value>),
    Range {
        from: serde_json::Value,
        to: serde_json::Value,
    },
}

/// Sorting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBy {
    pub column: String,
    pub direction: SortDirection,
}

/// Sort direction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Response structure for table data with metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct TableDataResponse {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub final_query: String,
    pub has_more_data: bool,
    pub execution_time: u128, // milliseconds
}

/// Column information including name and data type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
}

impl TableDataResponse {
    pub fn new(
        columns: Vec<ColumnInfo>,
        rows: Vec<Vec<serde_json::Value>>,
        final_query: String,
        has_more_data: bool,
        execution_time: Duration,
    ) -> Self {
        Self {
            columns,
            rows,
            final_query,
            has_more_data,
            execution_time: execution_time.as_millis(),
        }
    }
}
