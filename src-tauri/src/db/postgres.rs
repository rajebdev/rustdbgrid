use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{Column as SqlxColumn, PgPool, Row, TypeInfo};
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

/// Enum for pre-computed PostgreSQL column types
#[derive(Clone, Copy)]
enum PgColType {
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
fn map_pg_type(base_type: &str) -> PgColType {
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
fn extract_pg_value_typed(
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
            // Fallback: try common types
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

pub struct PostgresConnection {
    pool: Option<PgPool>,
}

impl PostgresConnection {
    pub fn new() -> Self {
        Self { pool: None }
    }
}

impl Default for PostgresConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseConnection for PostgresConnection {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()> {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.username.as_ref().unwrap_or(&"postgres".to_string()),
            config.password.as_ref().unwrap_or(&"".to_string()),
            config.host,
            config.port,
            config.database.as_ref().unwrap_or(&"postgres".to_string())
        );

        self.pool = Some(PgPool::connect(&url).await?);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(pool) = &self.pool {
            pool.close().await;
            self.pool = None;
        }
        Ok(())
    }

    async fn test_connection(&mut self) -> Result<bool> {
        if let Some(pool) = &self.pool {
            sqlx::query("SELECT 1").fetch_one(pool).await?;
            Ok(true)
        } else {
            Err(anyhow!("Not connected"))
        }
    }

    async fn execute_query(&mut self, query: &str) -> Result<QueryResult> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let start = Instant::now();

        let rows = sqlx::query(query).fetch_all(pool).await?;
        let execution_time = start.elapsed().as_millis();

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: vec![],
                column_types: None,
                rows: vec![],
                rows_affected: None,
                execution_time,
                final_query: None,
            });
        }

        let columns: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| SqlxColumn::name(c).to_string())
            .collect();

        // Pre-compute column type info once upfront (base_type, is_array, col_type enum)
        let col_type_info: Vec<(String, bool, PgColType)> = rows[0]
            .columns()
            .iter()
            .map(|col| {
                let type_name = col.type_info().name();
                // Check if array type (can be _typename or TYPENAME[])
                let (base_type, is_array) = if let Some(stripped) = type_name.strip_prefix('_') {
                    // Format: _varchar, _int4, etc.
                    (stripped.to_lowercase(), true)
                } else if let Some(stripped) = type_name.strip_suffix("[]") {
                    // Format: VARCHAR[], INT4[], etc. - convert to lowercase for matching
                    (stripped.to_lowercase(), true)
                } else {
                    (type_name.to_lowercase(), false)
                };
                let col_type = map_pg_type(&base_type);
                (base_type, is_array, col_type)
            })
            .collect();

        let mut result_rows = Vec::with_capacity(rows.len());

        for row in rows {
            let mut row_map = HashMap::with_capacity(columns.len());
            for (i, col) in columns.iter().enumerate() {
                // Use pre-computed type info - enum matching instead of string matching per row
                let (base_type, is_array, col_type) = &col_type_info[i];
                let value = extract_pg_value_typed(&row, i, *col_type, *is_array, base_type);
                row_map.insert(col.clone(), value);
            }
            result_rows.push(row_map);
        }

        Ok(QueryResult {
            columns,
            column_types: None,
            rows: result_rows,
            rows_affected: None,
            execution_time,
            final_query: None,
        })
    }

    async fn get_databases(&mut self) -> Result<Vec<Database>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let rows = sqlx::query(
            "SELECT datname FROM pg_database 
             WHERE datistemplate = false 
             ORDER BY datname",
        )
        .fetch_all(pool)
        .await?;

        let databases = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get(0).unwrap_or_default();
                Database { name }
            })
            .collect();

        Ok(databases)
    }

    async fn get_tables(&mut self, _database: &str) -> Result<Vec<Table>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        // Note: We're already connected to a specific database, so we query the current database
        // The database parameter is included for API consistency but PostgreSQL requires reconnecting to switch databases
        let query = "SELECT 
                schemaname, 
                tablename,
                pg_total_relation_size(schemaname||'.'||tablename) as size_bytes
            FROM pg_tables 
            WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
            ORDER BY schemaname, tablename";

        let rows = sqlx::query(query).fetch_all(pool).await?;

        let tables = rows
            .iter()
            .map(|row| {
                let schema: String = row.try_get("schemaname").unwrap_or_default();
                let name: String = row.try_get("tablename").unwrap_or_default();
                let size_bytes: Option<i64> = row.try_get("size_bytes").ok();

                Table {
                    name,
                    schema: Some(schema),
                    size_bytes: size_bytes.map(|v| if v >= 0 { v as u64 } else { 0 }),
                }
            })
            .collect();

        Ok(tables)
    }

    async fn get_table_schema(&mut self, _database: &str, table: &str) -> Result<TableSchema> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        // Split schema and table if provided in schema.table format
        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let query = format!(
            "SELECT 
                column_name, 
                data_type, 
                is_nullable,
                column_default,
                CASE WHEN constraint_type = 'PRIMARY KEY' THEN true ELSE false END as is_primary
            FROM information_schema.columns 
            LEFT JOIN (
                SELECT kcu.column_name, tc.constraint_type
                FROM information_schema.key_column_usage kcu
                JOIN information_schema.table_constraints tc 
                    ON kcu.constraint_name = tc.constraint_name 
                    AND kcu.table_schema = tc.table_schema
                WHERE tc.table_schema = '{}' 
                    AND tc.table_name = '{}'
            ) constraints ON columns.column_name = constraints.column_name
            WHERE table_schema = '{}' 
                AND table_name = '{}'
            ORDER BY ordinal_position",
            schema, table_name, schema, table_name
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let columns = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("column_name").unwrap_or_default();
                let data_type: String = row.try_get("data_type").unwrap_or_default();
                let is_nullable: String = row.try_get("is_nullable").unwrap_or_default();
                let default_value: Option<String> = row.try_get("column_default").ok();
                let is_primary_key: bool = row.try_get("is_primary").unwrap_or(false);

                let is_auto_increment = default_value
                    .as_ref()
                    .map(|v| v.contains("nextval"))
                    .unwrap_or(false);

                Column {
                    name,
                    data_type,
                    nullable: is_nullable == "YES",
                    default_value,
                    is_primary_key,
                    is_auto_increment,
                }
            })
            .collect();

        // Get indexes
        let index_query = format!(
            "SELECT 
                indexname, 
                indexdef
            FROM pg_indexes 
            WHERE schemaname = '{}' 
                AND tablename = '{}'",
            schema, table_name
        );

        let index_rows = sqlx::query(&index_query).fetch_all(pool).await?;
        let indexes = index_rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("indexname").unwrap_or_default();
                let indexdef: String = row.try_get("indexdef").unwrap_or_default();
                let is_unique = indexdef.contains("UNIQUE");

                Index {
                    name,
                    columns: vec![], // Could be parsed from indexdef if needed
                    is_unique,
                }
            })
            .collect();

        Ok(TableSchema {
            table_name: table.to_string(),
            columns,
            indexes,
            foreign_keys: vec![],
        })
    }

    async fn get_table_data(
        &mut self,
        _database: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult> {
        // Handle schema.table format
        let table_identifier = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            format!("\"{}\".\"{}\"", parts[0], parts[1])
        } else {
            format!("\"public\".\"{}\"", table)
        };

        let query = format!(
            "SELECT * FROM {} LIMIT {} OFFSET {}",
            table_identifier, limit, offset
        );
        self.execute_query(&query).await
    }
}
