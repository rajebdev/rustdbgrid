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

        // PostgreSQL is case-sensitive for unquoted identifiers, so lowercase them
        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT 
                c.column_name, 
                c.data_type,
                c.character_maximum_length,
                c.numeric_precision,
                c.numeric_scale,
                c.is_nullable,
                c.column_default,
                c.ordinal_position,
                COALESCE(tc.constraint_type = 'PRIMARY KEY', false) as is_primary
            FROM information_schema.columns c
            LEFT JOIN information_schema.constraint_column_usage ccu 
                ON c.column_name = ccu.column_name 
                AND c.table_schema = ccu.table_schema
                AND c.table_name = ccu.table_name
            LEFT JOIN information_schema.table_constraints tc 
                ON ccu.constraint_name = tc.constraint_name
                AND tc.constraint_type = 'PRIMARY KEY'
            WHERE c.table_schema = '{}' 
                AND c.table_name = '{}'
            ORDER BY c.ordinal_position",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let columns = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("column_name").unwrap_or_default();
                let mut data_type: String = row.try_get("data_type").unwrap_or_default();
                let is_nullable: String = row.try_get("is_nullable").unwrap_or_default();
                let default_value: Option<String> = row.try_get("column_default").ok();
                let is_primary_key: bool = row.try_get("is_primary").unwrap_or(false);

                // Format data types with length/precision information
                let char_max_length: Option<i32> = row.try_get("character_maximum_length").ok();
                let numeric_precision: Option<i32> = row.try_get("numeric_precision").ok();
                let numeric_scale: Option<i32> = row.try_get("numeric_scale").ok();

                // Convert PostgreSQL type names to user-friendly short format
                data_type = match data_type.as_str() {
                    "character varying" => {
                        if let Some(length) = char_max_length {
                            format!("varchar({})", length)
                        } else {
                            "varchar".to_string()
                        }
                    }
                    "character" => {
                        if let Some(length) = char_max_length {
                            format!("char({})", length)
                        } else {
                            "char".to_string()
                        }
                    }
                    "numeric" | "decimal" => match (numeric_precision, numeric_scale) {
                        (Some(p), Some(s)) => format!("numeric({},{})", p, s),
                        (Some(p), None) => format!("numeric({})", p),
                        _ => data_type,
                    },
                    "double precision" => "float8".to_string(),
                    "smallint" => "int2".to_string(),
                    "integer" => "int4".to_string(),
                    "bigint" => "int8".to_string(),
                    "boolean" => "bool".to_string(),
                    "timestamp without time zone" => "timestamp".to_string(),
                    "timestamp with time zone" => "timestamptz".to_string(),
                    "time without time zone" => "time".to_string(),
                    "time with time zone" => "timetz".to_string(),
                    _ => data_type,
                };

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
            schema_lower, table_lower
        );

        let index_rows = sqlx::query(&index_query).fetch_all(pool).await?;
        let indexes = index_rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("indexname").unwrap_or_default();
                let indexdef: String = row.try_get("indexdef").unwrap_or_default();
                let is_unique = indexdef.contains("UNIQUE");

                // Parse index type from definition
                let index_type = if indexdef.to_lowercase().contains("btree") {
                    Some("BTREE".to_string())
                } else if indexdef.to_lowercase().contains("hash") {
                    Some("HASH".to_string())
                } else if indexdef.to_lowercase().contains("gist") {
                    Some("GIST".to_string())
                } else if indexdef.to_lowercase().contains("gin") {
                    Some("GIN".to_string())
                } else {
                    Some("BTREE".to_string())
                };

                Index {
                    name,
                    columns: vec![], // Could be parsed from indexdef if needed
                    is_unique,
                    index_type,
                    ascending: Some(true),
                    nullable: None,
                    extra: None,
                }
            })
            .collect();

        // Get foreign keys
        let fk_query = format!(
            "SELECT
                tc.constraint_name,
                kcu.column_name,
                ccu.table_name AS foreign_table_name,
                ccu.column_name AS foreign_column_name,
                rc.update_rule,
                rc.delete_rule,
                tc.table_schema as owner
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
            LEFT JOIN information_schema.referential_constraints AS rc
                ON tc.constraint_name = rc.constraint_name
                AND tc.table_schema = rc.constraint_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_schema = '{}'
                AND tc.table_name = '{}'",
            schema_lower, table_lower
        );

        let fk_rows = sqlx::query(&fk_query).fetch_all(pool).await?;
        let foreign_keys = fk_rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("constraint_name").unwrap_or_default();
                let column: String = row.try_get("column_name").unwrap_or_default();
                let referenced_table: String =
                    row.try_get("foreign_table_name").unwrap_or_default();
                let referenced_column: String =
                    row.try_get("foreign_column_name").unwrap_or_default();
                let on_update: Option<String> = row.try_get("update_rule").ok();
                let on_delete: Option<String> = row.try_get("delete_rule").ok();
                let owner: Option<String> = row.try_get("owner").ok();

                ForeignKey {
                    name,
                    column,
                    referenced_table,
                    referenced_column,
                    owner,
                    ref_object_type: Some("TABLE".to_string()),
                    on_delete,
                    on_update,
                }
            })
            .collect();

        Ok(TableSchema {
            table_name: table.to_string(),
            columns,
            indexes,
            foreign_keys,
        })
    }

    async fn get_table_relationships(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<TableRelationship>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        // Split schema and table if provided in schema.table format
        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        // PostgreSQL is case-sensitive for unquoted identifiers, so lowercase them
        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT
                tc.constraint_name,
                tc.table_name,
                kcu.column_name,
                ccu.table_name AS referenced_table_name,
                ccu.column_name AS referenced_column_name,
                rc.update_rule,
                rc.delete_rule,
                'FOREIGN_KEY' as relationship_type
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
            LEFT JOIN information_schema.referential_constraints AS rc
                ON tc.constraint_name = rc.constraint_name
                AND tc.table_schema = rc.constraint_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_schema = '{}'
                AND tc.table_name = '{}'
            UNION ALL
            SELECT
                tc.constraint_name,
                tc.table_name,
                kcu.column_name,
                ccu.table_name AS referenced_table_name,
                ccu.column_name AS referenced_column_name,
                rc.update_rule,
                rc.delete_rule,
                'REFERENCED_BY' as relationship_type
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
            LEFT JOIN information_schema.referential_constraints AS rc
                ON tc.constraint_name = rc.constraint_name
                AND tc.table_schema = rc.constraint_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_schema = '{}'
                AND ccu.table_name = '{}'",
            schema_lower, table_lower, schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let relationships = rows
            .iter()
            .map(|row| {
                let constraint_name: String = row.try_get("constraint_name").unwrap_or_default();
                let table_name: String = row.try_get("table_name").unwrap_or_default();
                let column_name: String = row.try_get("column_name").unwrap_or_default();
                let referenced_table_name: String =
                    row.try_get("referenced_table_name").unwrap_or_default();
                let referenced_column_name: String =
                    row.try_get("referenced_column_name").unwrap_or_default();
                let relationship_type: String =
                    row.try_get("relationship_type").unwrap_or_default();
                let on_update: Option<String> = row.try_get("update_rule").ok();
                let on_delete: Option<String> = row.try_get("delete_rule").ok();

                TableRelationship {
                    constraint_name,
                    table_name,
                    column_name,
                    referenced_table_name,
                    referenced_column_name,
                    relationship_type,
                    owner: Some(schema.clone()),
                    ref_object_type: Some("TABLE".to_string()),
                    on_delete,
                    on_update,
                }
            })
            .collect();

        Ok(relationships)
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

    async fn get_views(&mut self, _database: &str, schema: Option<&str>) -> Result<Vec<View>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let schema_filter = if let Some(s) = schema {
            format!("AND schemaname = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT schemaname as schema, viewname as name 
             FROM pg_views 
             WHERE schemaname NOT IN ('pg_catalog', 'information_schema') {}
             ORDER BY schemaname, viewname",
            schema_filter
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let views = rows
            .iter()
            .map(|row| {
                let schema: String = row.try_get("schema").unwrap_or_default();
                let name: String = row.try_get("name").unwrap_or_default();
                View {
                    schema: Some(schema),
                    name,
                }
            })
            .collect();

        Ok(views)
    }

    async fn get_indexes(&mut self, _database: &str, schema: Option<&str>) -> Result<Vec<DbIndex>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let schema_filter = if let Some(s) = schema {
            format!("AND schemaname = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT schemaname, tablename, indexname, indexdef
             FROM pg_indexes
             WHERE schemaname NOT IN ('pg_catalog', 'information_schema') {}
             ORDER BY schemaname, tablename, indexname",
            schema_filter
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let indexes = rows
            .iter()
            .map(|row| {
                let table_name: String = row.try_get("tablename").unwrap_or_default();
                let name: String = row.try_get("indexname").unwrap_or_default();
                let indexdef: String = row.try_get("indexdef").unwrap_or_default();
                let is_unique = indexdef.to_lowercase().contains("unique");

                // Parse index type from definition
                let index_type = if indexdef.to_lowercase().contains("btree") {
                    Some("BTREE".to_string())
                } else if indexdef.to_lowercase().contains("hash") {
                    Some("HASH".to_string())
                } else if indexdef.to_lowercase().contains("gist") {
                    Some("GIST".to_string())
                } else if indexdef.to_lowercase().contains("gin") {
                    Some("GIN".to_string())
                } else {
                    Some("BTREE".to_string())
                };

                DbIndex {
                    name,
                    table_name,
                    columns: vec![], // Could be parsed from indexdef if needed
                    is_unique,
                    index_type,
                    ascending: Some(true), // Default for PostgreSQL
                    nullable: None,
                    extra: None,
                }
            })
            .collect();

        Ok(indexes)
    }

    async fn get_procedures(
        &mut self,
        _database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Procedure>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let schema_filter = if let Some(s) = schema {
            format!("AND n.nspname = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT n.nspname as schema, p.proname as name,
                    CASE 
                        WHEN p.prokind = 'f' THEN 'FUNCTION'
                        WHEN p.prokind = 'p' THEN 'PROCEDURE'
                        ELSE 'FUNCTION'
                    END as type,
                    p.oid::text as oid
             FROM pg_proc p
             JOIN pg_namespace n ON p.pronamespace = n.oid
             WHERE n.nspname NOT IN ('pg_catalog', 'information_schema') {}
             ORDER BY n.nspname, p.proname",
            schema_filter
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let procedures = rows
            .iter()
            .map(|row| {
                let schema: String = row.try_get("schema").unwrap_or_default();
                let name: String = row.try_get("name").unwrap_or_default();
                let proc_type: String = row.try_get("type").unwrap_or_default();
                let oid: String = row.try_get("oid").unwrap_or_default();
                Procedure {
                    name,
                    schema: Some(schema),
                    procedure_type: Some(proc_type),
                    oid: Some(oid),
                }
            })
            .collect();

        Ok(procedures)
    }

    async fn get_triggers(
        &mut self,
        _database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Trigger>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let schema_filter = if let Some(s) = schema {
            format!("AND n.nspname = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT t.tgname as trigger_name,
                    c.relname as table_name,
                    CASE 
                        WHEN t.tgtype & 2 = 2 THEN 'BEFORE'
                        WHEN t.tgtype & 64 = 64 THEN 'INSTEAD OF'
                        ELSE 'AFTER'
                    END as timing,
                    CASE 
                        WHEN t.tgtype & 4 = 4 THEN 'INSERT'
                        WHEN t.tgtype & 8 = 8 THEN 'DELETE'
                        WHEN t.tgtype & 16 = 16 THEN 'UPDATE'
                        ELSE 'UNKNOWN'
                    END as event
             FROM pg_trigger t
             JOIN pg_class c ON t.tgrelid = c.oid
             JOIN pg_namespace n ON c.relnamespace = n.oid
             WHERE NOT t.tgisinternal
                   AND n.nspname NOT IN ('pg_catalog', 'information_schema') {}
             ORDER BY n.nspname, c.relname, t.tgname",
            schema_filter
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let triggers = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("trigger_name").unwrap_or_default();
                let table_name: String = row.try_get("table_name").unwrap_or_default();
                let timing: String = row.try_get("timing").unwrap_or_default();
                let event: String = row.try_get("event").unwrap_or_default();

                Trigger {
                    name,
                    table_name,
                    timing,
                    event,
                    trigger_type: Some("ROW".to_string()),
                    description: None,
                }
            })
            .collect();

        Ok(triggers)
    }

    async fn get_table_statistics(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<TableStatistics> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        // Parse schema and table name
        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        // PostgreSQL is case-sensitive for unquoted identifiers, so lowercase them
        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT 
                c.reltuples::bigint as row_count,
                pg_total_relation_size(c.oid)::bigint as total_size,
                pg_relation_size(c.oid)::bigint as relation_size,
                pg_indexes_size(c.oid)::bigint as index_length,
                CASE 
                    WHEN c.relkind = 'r' THEN 'TABLE'
                    WHEN c.relkind = 'v' THEN 'VIEW'
                    WHEN c.relkind = 'm' THEN 'MATERIALIZED VIEW'
                    ELSE c.relkind::text
                END as row_format,
                obj_description(c.oid, 'pg_class') as comment
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = '{}' AND c.relname = '{}'",
            schema_lower, table_lower
        );

        // Use fetch_optional to handle case when table is not found
        match sqlx::query(&query).fetch_optional(pool).await {
            Ok(Some(row)) => {
                let row_count = row.try_get::<Option<i64>, _>("row_count").ok().flatten();
                let total_size = row.try_get::<Option<i64>, _>("total_size").ok().flatten();
                let relation_size = row
                    .try_get::<Option<i64>, _>("relation_size")
                    .ok()
                    .flatten();
                let index_length = row.try_get::<Option<i64>, _>("index_length").ok().flatten();

                // Format table_size for PostgreSQL display (Disk Space)
                let table_size = total_size.map(|size| {
                    if size < 1024 {
                        format!("{}B", size)
                    } else if size < 1024 * 1024 {
                        format!("{:.0}K", size as f64 / 1024.0)
                    } else if size < 1024 * 1024 * 1024 {
                        format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
                    } else {
                        format!("{:.1}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
                    }
                });

                // Calculate average row length if we have both row count and relation size
                let avg_row_length = match (row_count, relation_size) {
                    (Some(count), Some(size)) if count > 0 => Some(size / count),
                    _ => None,
                };

                let statistics = TableStatistics {
                    row_count,
                    avg_row_length,
                    data_length: relation_size,
                    max_data_length: None,
                    data_free: None,
                    index_length,
                    row_format: row
                        .try_get::<Option<String>, _>("row_format")
                        .ok()
                        .flatten(),
                    create_time: None,
                    update_time: None,
                    check_time: None,
                    collation: None,
                    checksum: None,
                    engine: Some("PostgreSQL".to_string()),
                    comment: row.try_get::<Option<String>, _>("comment").ok().flatten(),
                    table_size,
                    pages: None,
                };

                Ok(statistics)
            }
            Ok(None) => {
                // Table not found, return default statistics with N/A values
                println!(
                    "⚠️ PostgreSQL table not found: {}.{}",
                    schema_lower, table_lower
                );
                Ok(TableStatistics {
                    row_count: None,
                    avg_row_length: None,
                    data_length: None,
                    max_data_length: None,
                    data_free: None,
                    index_length: None,
                    row_format: None,
                    create_time: None,
                    update_time: None,
                    check_time: None,
                    collation: None,
                    checksum: None,
                    engine: Some("PostgreSQL".to_string()),
                    comment: None,
                    table_size: None,
                    pages: None,
                })
            }
            Err(e) => {
                // Error executing query
                println!("❌ Error fetching PostgreSQL table statistics: {}", e);
                Err(anyhow!("Error fetching table statistics: {}", e))
            }
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// PostgreSQL-specific implementations
impl PostgresConnection {
    pub async fn get_pg_constraints(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<crate::models::schema::PgConstraint>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT 
                con.conname as constraint_name,
                CASE con.contype
                    WHEN 'c' THEN 'CHECK'
                    WHEN 'f' THEN 'FOREIGN KEY'
                    WHEN 'p' THEN 'PRIMARY KEY'
                    WHEN 'u' THEN 'UNIQUE'
                    WHEN 't' THEN 'TRIGGER'
                    WHEN 'x' THEN 'EXCLUSION'
                    ELSE con.contype::text
                END as constraint_type,
                array_to_string(ARRAY(
                    SELECT a.attname
                    FROM unnest(con.conkey) AS u(attnum)
                    JOIN pg_attribute AS a ON a.attnum = u.attnum AND a.attrelid = con.conrelid
                ), ', ') as attributes,
                pg_get_constraintdef(con.oid) as expression,
                pg_catalog.obj_description(con.oid, 'pg_constraint') as comment,
                n.nspname as owner
            FROM pg_constraint con
            JOIN pg_class c ON c.oid = con.conrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = '{}'
                AND c.relname = '{}'
                AND con.contype IN ('c', 'p', 'u', 'x')
            ORDER BY con.contype, con.conname",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let constraints: Vec<crate::models::schema::PgConstraint> = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("constraint_name").unwrap_or_default();
                let constraint_type: String = row.try_get("constraint_type").unwrap_or_default();
                let attribute: String = row.try_get("attributes").unwrap_or_default();
                let expression: Option<String> = row.try_get("expression").ok();
                let comment: Option<String> = row.try_get("comment").ok();
                let owner: Option<String> = row.try_get("owner").ok();

                crate::models::schema::PgConstraint {
                    name,
                    attribute,
                    owner,
                    constraint_type,
                    expression,
                    comment,
                }
            })
            .collect();

        Ok(constraints)
    }

    pub async fn get_pg_foreign_keys(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<crate::models::schema::PgForeignKey>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT 
                con.conname as fk_name,
                array_to_string(ARRAY(
                    SELECT a.attname
                    FROM unnest(con.conkey) AS u(attnum)
                    JOIN pg_attribute AS a ON a.attnum = u.attnum AND a.attrelid = con.conrelid
                ), ', ') as attributes,
                array_to_string(ARRAY(
                    SELECT a.attname
                    FROM unnest(con.confkey) AS u(attnum)
                    JOIN pg_attribute AS a ON a.attnum = u.attnum AND a.attrelid = con.confrelid
                ), ', ') as reference_columns,
                nf.nspname || '.' || cf.relname as associated_entity,
                CASE con.confmatchtype
                    WHEN 'f' THEN 'FULL'
                    WHEN 'p' THEN 'PARTIAL'
                    WHEN 's' THEN 'SIMPLE'
                    ELSE 'NONE'
                END as match_type,
                CASE con.confdeltype
                    WHEN 'a' THEN 'NO ACTION'
                    WHEN 'r' THEN 'RESTRICT'
                    WHEN 'c' THEN 'CASCADE'
                    WHEN 'n' THEN 'SET NULL'
                    WHEN 'd' THEN 'SET DEFAULT'
                    ELSE 'NO ACTION'
                END as delete_rule,
                CASE con.confupdtype
                    WHEN 'a' THEN 'NO ACTION'
                    WHEN 'r' THEN 'RESTRICT'
                    WHEN 'c' THEN 'CASCADE'
                    WHEN 'n' THEN 'SET NULL'
                    WHEN 'd' THEN 'SET DEFAULT'
                    ELSE 'NO ACTION'
                END as update_rule,
                pg_catalog.obj_description(con.oid, 'pg_constraint') as comment,
                n.nspname as owner,
                'FOREIGN KEY' as fk_type
            FROM pg_constraint con
            JOIN pg_class c ON c.oid = con.conrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_class cf ON cf.oid = con.confrelid
            JOIN pg_namespace nf ON nf.oid = cf.relnamespace
            WHERE n.nspname = '{}'
                AND c.relname = '{}'
                AND con.contype = 'f'
            ORDER BY con.conname",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let foreign_keys: Vec<crate::models::schema::PgForeignKey> = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("fk_name").unwrap_or_default();
                let attribute: String = row.try_get("attributes").unwrap_or_default();
                let reference_column: String = row.try_get("reference_columns").unwrap_or_default();
                let associated_entity: String =
                    row.try_get("associated_entity").unwrap_or_default();
                let match_type: Option<String> = row.try_get("match_type").ok();
                let delete_rule: Option<String> = row.try_get("delete_rule").ok();
                let update_rule: Option<String> = row.try_get("update_rule").ok();
                let comment: Option<String> = row.try_get("comment").ok();
                let owner: Option<String> = row.try_get("owner").ok();
                let fk_type: String = row.try_get("fk_type").unwrap_or_default();

                crate::models::schema::PgForeignKey {
                    name,
                    attribute,
                    owner,
                    fk_type,
                    reference_column,
                    associated_entity,
                    match_type,
                    delete_rule,
                    update_rule,
                    comment,
                }
            })
            .collect();

        Ok(foreign_keys)
    }

    pub async fn get_pg_indexes(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<crate::models::schema::PgIndex>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT 
                i.relname as idx_name,
                c.relname as table_name,
                a.attname as column_name,
                ix.indisunique as is_unique,
                am.amname as index_type,
                CASE 
                    WHEN ix.indoption[array_position(ix.indkey::int[], a.attnum::int) - 1] & 1 = 1 THEN false
                    ELSE true
                END as ascending,
                NOT a.attnotnull as nullable,
                opc.opcname as operator_class,
                pg_get_expr(ix.indpred, ix.indrelid) as predicate
            FROM pg_index ix
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_class c ON c.oid = ix.indrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_am am ON am.oid = i.relam
            CROSS JOIN LATERAL unnest(ix.indkey) WITH ORDINALITY AS u(attnum, ord)
            JOIN pg_attribute a ON a.attrelid = c.oid AND a.attnum = u.attnum
            LEFT JOIN pg_opclass opc ON opc.oid = ix.indclass[u.ord - 1]
            WHERE n.nspname = '{}'
                AND c.relname = '{}'
                AND NOT ix.indisprimary
            ORDER BY i.relname, u.ord",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let indexes: Vec<crate::models::schema::PgIndex> = rows
            .iter()
            .map(|row| {
                let idx_name: String = row.try_get("idx_name").unwrap_or_default();
                let table: String = row.try_get("table_name").unwrap_or_default();
                let column: String = row.try_get("column_name").unwrap_or_default();
                let unique: bool = row.try_get("is_unique").unwrap_or(false);
                let ascending: Option<bool> = row.try_get("ascending").ok();
                let nullable: Option<bool> = row.try_get("nullable").ok();
                let operator_class: Option<String> = row.try_get("operator_class").ok();
                let predicate: Option<String> = row.try_get("predicate").ok();

                crate::models::schema::PgIndex {
                    column,
                    idx_name,
                    table,
                    ascending,
                    nullable,
                    unique,
                    operator_class,
                    predicate,
                }
            })
            .collect();

        Ok(indexes)
    }

    pub async fn get_pg_references(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<crate::models::schema::PgReference>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT 
                con.conname as ref_name,
                'FOREIGN KEY' as ref_type,
                n.nspname || '.' || c.relname as associated_entity,
                NULL::int as sequence_num,
                pg_catalog.obj_description(con.oid, 'pg_constraint') as comment,
                n.nspname as owner
            FROM pg_constraint con
            JOIN pg_class c ON c.oid = con.conrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_class cf ON cf.oid = con.confrelid
            JOIN pg_namespace nf ON nf.oid = cf.relnamespace
            WHERE nf.nspname = '{}'
                AND cf.relname = '{}'
                AND con.contype = 'f'
            ORDER BY con.conname",
            schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let references: Vec<crate::models::schema::PgReference> = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("ref_name").unwrap_or_default();
                let ref_type: String = row.try_get("ref_type").unwrap_or_default();
                let associated_entity: String =
                    row.try_get("associated_entity").unwrap_or_default();
                let sequence_num: Option<i32> = row.try_get("sequence_num").ok();
                let comment: Option<String> = row.try_get("comment").ok();
                let owner: Option<String> = row.try_get("owner").ok();

                crate::models::schema::PgReference {
                    name,
                    owner,
                    ref_type,
                    comment,
                    associated_entity,
                    sequence_num,
                }
            })
            .collect();

        Ok(references)
    }

    pub async fn get_pg_partitions(
        &mut self,
        _database: &str,
        table: &str,
    ) -> Result<Vec<crate::models::schema::PgPartition>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to database"))?;

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("public".to_string(), table.to_string())
        };

        let schema_lower = schema.to_lowercase();
        let table_lower = table_name.to_lowercase();

        let query = format!(
            "SELECT 
                c.relname as table_name,
                c.oid::text as object_id,
                n.nspname as owner,
                ts.spcname as tablespace,
                c.reltuples::bigint as rowcount_estimate,
                c.relrowsecurity as has_row_level_security,
                (SELECT count(*) FROM pg_inherits WHERE inhparent = c.oid) as partition_count,
                CASE 
                    WHEN c.relkind = 'p' THEN 
                        pg_get_partkeydef(c.oid)
                    ELSE NULL
                END as partition_by,
                pg_catalog.obj_description(c.oid, 'pg_class') as comment,
                c.reloptions as extra_options
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            LEFT JOIN pg_tablespace ts ON ts.oid = c.reltablespace
            WHERE n.nspname = '{}'
                AND c.relname = '{}'
                AND c.relkind IN ('r', 'p')
            
            UNION ALL
            
            SELECT 
                child.relname as table_name,
                child.oid::text as object_id,
                n.nspname as owner,
                ts.spcname as tablespace,
                child.reltuples::bigint as rowcount_estimate,
                child.relrowsecurity as has_row_level_security,
                0 as partition_count,
                pg_get_expr(child.relpartbound, child.oid) as partition_by,
                pg_catalog.obj_description(child.oid, 'pg_class') as comment,
                child.reloptions as extra_options
            FROM pg_inherits i
            JOIN pg_class parent ON parent.oid = i.inhparent
            JOIN pg_class child ON child.oid = i.inhrelid
            JOIN pg_namespace n ON n.oid = child.relnamespace
            LEFT JOIN pg_tablespace ts ON ts.oid = child.reltablespace
            JOIN pg_namespace pn ON pn.oid = parent.relnamespace
            WHERE pn.nspname = '{}'
                AND parent.relname = '{}'
            ORDER BY table_name",
            schema_lower, table_lower, schema_lower, table_lower
        );

        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let partitions: Vec<crate::models::schema::PgPartition> = rows
            .iter()
            .map(|row| {
                let table_name: String = row.try_get("table_name").unwrap_or_default();
                let object_id: Option<String> = row.try_get("object_id").ok();
                let owner: Option<String> = row.try_get("owner").ok();
                let tablespace: Option<String> = row.try_get("tablespace").ok();
                let rowcount_estimate: Option<i64> = row.try_get("rowcount_estimate").ok();
                let has_row_level_security: bool =
                    row.try_get("has_row_level_security").unwrap_or(false);
                let partitions: Option<i32> = row.try_get("partition_count").ok();
                let partition_by: Option<String> = row.try_get("partition_by").ok();
                let comment: Option<String> = row.try_get("comment").ok();
                let extra_options: Option<String> = row
                    .try_get::<Option<Vec<String>>, _>("extra_options")
                    .ok()
                    .flatten()
                    .map(|opts| opts.join(", "));

                crate::models::schema::PgPartition {
                    table_name,
                    object_id,
                    owner,
                    tablespace,
                    rowcount_estimate,
                    has_row_level_security,
                    partitions,
                    partition_by: partition_by.clone(),
                    partitions_expression: partition_by,
                    extra_options,
                    comment,
                }
            })
            .collect();

        Ok(partitions)
    }
}
