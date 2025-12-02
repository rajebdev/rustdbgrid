use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::Row;
use uuid::Uuid;

/// Enum for pre-computed PostgreSQL column types
#[derive(Clone, Copy)]
pub enum PgColType {
    Timestamp,
    TimestampTz,
    Date,
    Time,
    TimeTz,
    Interval,
    Int16,
    Int32,
    Int64,
    Oid,
    Float32,
    Float64,
    Numeric,
    Money,
    Boolean,
    String,
    Uuid,
    Json,
    Bytea,
    Network,
    BitString,
    Geometry,
    Range,
    Unknown,
}

/// Map base type string to enum (called once per column)
pub fn map_pg_type(base_type: &str) -> PgColType {
    match base_type {
        "timestamp" => PgColType::Timestamp,
        "timestamptz" => PgColType::TimestampTz,
        "date" => PgColType::Date,
        "time" => PgColType::Time,
        "timetz" => PgColType::TimeTz,
        "interval" => PgColType::Interval,
        "int2" | "smallint" | "smallserial" => PgColType::Int16,
        "int4" | "int" | "integer" | "serial" => PgColType::Int32,
        "int8" | "bigint" | "bigserial" => PgColType::Int64,
        "oid" => PgColType::Oid,
        "float4" | "real" => PgColType::Float32,
        "float8" | "double precision" => PgColType::Float64,
        "numeric" | "decimal" => PgColType::Numeric,
        "money" => PgColType::Money,
        "bool" | "boolean" => PgColType::Boolean,
        "text" | "varchar" | "char" | "bpchar" | "name" | "citext" | "unknown" | "xml" => {
            PgColType::String
        }
        "uuid" => PgColType::Uuid,
        "json" | "jsonb" => PgColType::Json,
        "bytea" => PgColType::Bytea,
        "inet" | "cidr" | "macaddr" | "macaddr8" => PgColType::Network,
        "bit" | "varbit" => PgColType::BitString,
        "point" | "line" | "lseg" | "box" | "path" | "polygon" | "circle" => PgColType::Geometry,
        "int4range" | "int8range" | "numrange" | "tsrange" | "tstzrange" | "daterange" => {
            PgColType::Range
        }
        _ => PgColType::Unknown,
    }
}

/// Extract value using pre-computed type enum (no string matching per row)
pub fn extract_pg_value_typed(
    row: &PgRow,
    idx: usize,
    col_type: PgColType,
    is_array: bool,
    base_type: &str,
) -> serde_json::Value {
    match col_type {
        PgColType::Timestamp => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<NaiveDateTime>, _>(idx) {
                    let formatted: Vec<String> = v
                        .iter()
                        .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                        .collect();
                    return serde_json::json!(formatted);
                }
            } else if let Ok(v) = row.try_get::<NaiveDateTime, _>(idx) {
                return serde_json::json!(v.format("%Y-%m-%d %H:%M:%S").to_string());
            }
        }
        PgColType::TimestampTz => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<DateTime<Utc>>, _>(idx) {
                    let formatted: Vec<String> = v
                        .iter()
                        .map(|d| d.format("%Y-%m-%d %H:%M:%S %Z").to_string())
                        .collect();
                    return serde_json::json!(formatted);
                }
            } else if let Ok(v) = row.try_get::<DateTime<Utc>, _>(idx) {
                return serde_json::json!(v.format("%Y-%m-%d %H:%M:%S %Z").to_string());
            } else if let Ok(v) = row.try_get::<NaiveDateTime, _>(idx) {
                return serde_json::json!(v.format("%Y-%m-%d %H:%M:%S").to_string());
            }
        }
        PgColType::Date => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<NaiveDate>, _>(idx) {
                    let formatted: Vec<String> =
                        v.iter().map(|d| d.format("%Y-%m-%d").to_string()).collect();
                    return serde_json::json!(formatted);
                }
            } else if let Ok(v) = row.try_get::<NaiveDate, _>(idx) {
                return serde_json::json!(v.format("%Y-%m-%d").to_string());
            }
        }
        PgColType::Time => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<NaiveTime>, _>(idx) {
                    let formatted: Vec<String> =
                        v.iter().map(|t| t.format("%H:%M:%S").to_string()).collect();
                    return serde_json::json!(formatted);
                }
            } else if let Ok(v) = row.try_get::<NaiveTime, _>(idx) {
                return serde_json::json!(v.format("%H:%M:%S").to_string());
            }
        }
        PgColType::TimeTz => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<String>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else if let Ok(v) = row.try_get::<String, _>(idx) {
                return serde_json::json!(v);
            }
        }
        PgColType::Interval => {
            if !is_array {
                if let Ok(v) = row.try_get::<sqlx::postgres::types::PgInterval, _>(idx) {
                    return serde_json::json!(format!(
                        "{} mons {} days {} µs",
                        v.months, v.days, v.microseconds
                    ));
                }
            }
        }
        PgColType::Int16 => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<i16>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else if let Ok(v) = row.try_get::<i16, _>(idx) {
                return serde_json::json!(v);
            }
        }
        PgColType::Int32 => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<i32>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else if let Ok(v) = row.try_get::<i32, _>(idx) {
                return serde_json::json!(v);
            }
        }
        PgColType::Int64 => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<i64>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else if let Ok(v) = row.try_get::<i64, _>(idx) {
                return serde_json::json!(v);
            }
        }
        PgColType::Oid => {
            if !is_array {
                if let Ok(v) = row.try_get::<sqlx::postgres::types::Oid, _>(idx) {
                    return serde_json::json!(v.0);
                }
            }
            if let Ok(v) = row.try_get::<i32, _>(idx) {
                return serde_json::json!(v);
            }
        }
        PgColType::Float32 => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<f32>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else if let Ok(v) = row.try_get::<f32, _>(idx) {
                return serde_json::json!(v);
            }
        }
        PgColType::Float64 => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<f64>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else if let Ok(v) = row.try_get::<f64, _>(idx) {
                return serde_json::json!(v);
            }
        }
        PgColType::Numeric => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<String>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else {
                if let Ok(v) = row.try_get::<String, _>(idx) {
                    return serde_json::json!(v);
                }
                if let Ok(v) = row.try_get::<f64, _>(idx) {
                    return serde_json::json!(v);
                }
            }
        }
        PgColType::Money => {
            if !is_array {
                if let Ok(v) = row.try_get::<sqlx::postgres::types::PgMoney, _>(idx) {
                    return serde_json::json!(format!("${:.2}", v.0 as f64 / 100.0));
                }
            }
        }
        PgColType::Boolean => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<bool>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else if let Ok(v) = row.try_get::<bool, _>(idx) {
                return serde_json::json!(v);
            }
        }
        PgColType::String => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<String>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else if let Ok(v) = row.try_get::<String, _>(idx) {
                return serde_json::json!(v);
            }
        }
        PgColType::Uuid => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<Uuid>, _>(idx) {
                    let formatted: Vec<String> = v.iter().map(|u| u.to_string()).collect();
                    return serde_json::json!(formatted);
                }
            } else if let Ok(v) = row.try_get::<Uuid, _>(idx) {
                return serde_json::json!(v.to_string());
            }
        }
        PgColType::Json => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<serde_json::Value>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else if let Ok(v) = row.try_get::<serde_json::Value, _>(idx) {
                return v;
            }
        }
        PgColType::Bytea => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<Vec<u8>>, _>(idx) {
                    let formatted: Vec<String> = v
                        .iter()
                        .map(|b| format!("[BYTEA {} bytes]", b.len()))
                        .collect();
                    return serde_json::json!(formatted);
                }
            } else if let Ok(v) = row.try_get::<Vec<u8>, _>(idx) {
                return serde_json::json!(format!("[BYTEA {} bytes]", v.len()));
            }
        }
        PgColType::Network | PgColType::BitString => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<String>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else if let Ok(v) = row.try_get::<String, _>(idx) {
                return serde_json::json!(v);
            }
        }
        PgColType::Geometry => {
            return serde_json::json!(format!("[{} geometry]", base_type.to_uppercase()));
        }
        PgColType::Range => {
            return serde_json::json!(format!("[{} range]", base_type));
        }
        PgColType::Unknown => {
            if is_array {
                if let Ok(v) = row.try_get::<Vec<String>, _>(idx) {
                    return serde_json::json!(v);
                }
            } else {
                if let Ok(v) = row.try_get::<i64, _>(idx) {
                    return serde_json::json!(v);
                }
                if let Ok(v) = row.try_get::<i32, _>(idx) {
                    return serde_json::json!(v);
                }
                if let Ok(v) = row.try_get::<f64, _>(idx) {
                    return serde_json::json!(v);
                }
                if let Ok(v) = row.try_get::<bool, _>(idx) {
                    return serde_json::json!(v);
                }
                if let Ok(v) = row.try_get::<String, _>(idx) {
                    return serde_json::json!(v);
                }
                if let Ok(v) = row.try_get::<Vec<u8>, _>(idx) {
                    return serde_json::json!(format!("[Binary {} bytes]", v.len()));
                }
            }
        }
    }
    serde_json::Value::Null
}
