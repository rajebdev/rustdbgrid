use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json;
use sqlx::mysql::MySqlRow;
use sqlx::Row;

/// Enum for MySQL column types
#[derive(Clone, Copy)]
pub enum MySqlColType {
    DateTime,
    Date,
    Time,
    Integer,
    Float,
    Boolean,
    String,
    Blob,
    Unknown,
}

/// MySQL type converter utility
pub struct MySqlTypeConverter;

impl MySqlTypeConverter {
    /// Map MySQL type name to enum
    pub fn map_type(type_name: &str) -> MySqlColType {
        match type_name {
            "DATETIME" | "TIMESTAMP" => MySqlColType::DateTime,
            "DATE" => MySqlColType::Date,
            "TIME" => MySqlColType::Time,
            "TINYINT" | "SMALLINT" | "MEDIUMINT" | "INT" | "BIGINT" => MySqlColType::Integer,
            "FLOAT" | "DOUBLE" | "DECIMAL" => MySqlColType::Float,
            "BOOLEAN" | "BOOL" => MySqlColType::Boolean,
            "VARCHAR" | "CHAR" | "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT" | "ENUM"
            | "SET" => MySqlColType::String,
            "BLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY" | "VARBINARY" | "TINYBLOB" => {
                MySqlColType::Blob
            }
            _ => MySqlColType::Unknown,
        }
    }

    /// Extract value using pre-computed type enum
    pub fn extract_value_typed(
        row: &MySqlRow,
        idx: usize,
        col_type: MySqlColType,
    ) -> serde_json::Value {
        match col_type {
            MySqlColType::DateTime => row
                .try_get::<NaiveDateTime, _>(idx)
                .map(|v| serde_json::json!(v.format("%Y-%m-%d %H:%M:%S").to_string()))
                .unwrap_or(serde_json::Value::Null),
            MySqlColType::Date => row
                .try_get::<NaiveDate, _>(idx)
                .map(|v| serde_json::json!(v.format("%Y-%m-%d").to_string()))
                .unwrap_or(serde_json::Value::Null),
            MySqlColType::Time => row
                .try_get::<NaiveTime, _>(idx)
                .map(|v| serde_json::json!(v.format("%H:%M:%S").to_string()))
                .unwrap_or(serde_json::Value::Null),
            MySqlColType::Integer => row
                .try_get::<i64, _>(idx)
                .map(|v| serde_json::json!(v))
                .unwrap_or(serde_json::Value::Null),
            MySqlColType::Float => row
                .try_get::<f64, _>(idx)
                .map(|v| serde_json::json!(v))
                .unwrap_or(serde_json::Value::Null),
            MySqlColType::Boolean => row
                .try_get::<bool, _>(idx)
                .map(|v| serde_json::json!(v))
                .unwrap_or(serde_json::Value::Null),
            MySqlColType::String => row
                .try_get::<String, _>(idx)
                .map(|v| serde_json::json!(v))
                .unwrap_or(serde_json::Value::Null),
            MySqlColType::Blob => row
                .try_get::<Vec<u8>, _>(idx)
                .map(|v| serde_json::json!(format!("[BLOB {} bytes]", v.len())))
                .unwrap_or(serde_json::Value::Null),
            MySqlColType::Unknown => row
                .try_get::<String, _>(idx)
                .map(|v| serde_json::json!(v))
                .or_else(|_| {
                    row.try_get::<Vec<u8>, _>(idx)
                        .map(|v| serde_json::json!(format!("[BINARY {} bytes]", v.len())))
                })
                .unwrap_or(serde_json::Value::Null),
        }
    }
}
