use tiberius::Row;

/// Enum for pre-computed MSSQL column types
#[derive(Clone, Copy)]
pub enum MssqlColType {
    String,
    Int32,
    Int64,
    Int16,
    UInt8,
    Float32,
    Float64,
    Boolean,
    Uuid,
    DateTime,
    Date,
    Time,
    Binary,
    Decimal,
    Unknown,
}

/// Optimized helper function using pre-computed type
pub fn row_value_to_json_typed(
    row: &Row,
    index: usize,
    col_type: MssqlColType,
) -> serde_json::Value {
    match col_type {
        MssqlColType::String => {
            if let Some(v) = row.try_get::<&str, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
        }
        MssqlColType::Int32 => {
            if let Some(v) = row.try_get::<i32, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
        }
        MssqlColType::Int64 => {
            if let Some(v) = row.try_get::<i64, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
        }
        MssqlColType::Int16 => {
            if let Some(v) = row.try_get::<i16, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
        }
        MssqlColType::UInt8 => {
            if let Some(v) = row.try_get::<u8, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
        }
        MssqlColType::Float32 => {
            if let Some(v) = row.try_get::<f32, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
        }
        MssqlColType::Float64 => {
            if let Some(v) = row.try_get::<f64, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
        }
        MssqlColType::Boolean => {
            if let Some(v) = row.try_get::<bool, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
        }
        MssqlColType::Uuid => {
            if let Some(v) = row.try_get::<tiberius::Uuid, _>(index).ok().flatten() {
                return serde_json::json!(v.to_string());
            }
        }
        MssqlColType::DateTime => {
            if let Some(v) = row
                .try_get::<chrono::NaiveDateTime, _>(index)
                .ok()
                .flatten()
            {
                return serde_json::json!(v.format("%Y-%m-%d %H:%M:%S").to_string());
            }
        }
        MssqlColType::Date => {
            if let Some(v) = row.try_get::<chrono::NaiveDate, _>(index).ok().flatten() {
                return serde_json::json!(v.format("%Y-%m-%d").to_string());
            }
        }
        MssqlColType::Time => {
            if let Some(v) = row.try_get::<chrono::NaiveTime, _>(index).ok().flatten() {
                return serde_json::json!(v.format("%H:%M:%S").to_string());
            }
        }
        MssqlColType::Binary => {
            if let Some(v) = row.try_get::<&[u8], _>(index).ok().flatten() {
                return serde_json::json!(format!("[BINARY {} bytes]", v.len()));
            }
        }
        MssqlColType::Decimal => {
            if let Some(v) = row
                .try_get::<bigdecimal::BigDecimal, _>(index)
                .ok()
                .flatten()
            {
                return serde_json::json!(v.to_string());
            }
        }
        MssqlColType::Unknown => {
            // Fallback: try common types in order
            if let Some(v) = row.try_get::<&str, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
            if let Some(v) = row.try_get::<i32, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
            if let Some(v) = row.try_get::<i64, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
            if let Some(v) = row.try_get::<f64, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
            if let Some(v) = row.try_get::<bool, _>(index).ok().flatten() {
                return serde_json::json!(v);
            }
            if let Some(v) = row.try_get::<&[u8], _>(index).ok().flatten() {
                return serde_json::json!(format!("[BINARY {} bytes]", v.len()));
            }
        }
    }
    serde_json::Value::Null
}
