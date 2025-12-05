use crate::db::mysql::metadata_ops::MySqlMetadataOps;
use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::{Column as SqlxColumn, Executor, MySqlPool, Row, Statement, TypeInfo};
use std::collections::HashMap;
use std::time::Instant;

pub struct MySQLConnection {
    pool: Option<MySqlPool>,
}

impl MySQLConnection {
    pub fn new() -> Self {
        Self { pool: None }
    }
}

impl Default for MySQLConnection {
    fn default() -> Self {
        Self::new()
    }
}

// Define column type categories for efficient lookup
#[derive(Clone, Copy)]
enum ColType {
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

#[async_trait]
impl DatabaseConnection for MySQLConnection {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()> {
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username.as_ref().unwrap_or(&"root".to_string()),
            config.password.as_ref().unwrap_or(&"".to_string()),
            config.host,
            config.port,
            config.database.as_ref().unwrap_or(&"".to_string())
        );

        self.pool = Some(MySqlPool::connect(&url).await?);
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

        // Execute query first
        let rows = sqlx::query(query).fetch_all(pool).await?;
        let execution_time = start.elapsed().as_millis();

        // Extract columns from first row if available, otherwise prepare to get metadata
        let stmt_columns_vec: Vec<_> = if !rows.is_empty() {
            rows[0].columns().to_vec()
        } else {
            // For empty results, prepare statement to get column metadata
            let prepared = pool.prepare(query).await?;
            prepared.columns().to_vec()
        };

        // Handle duplicate column names by adding numeric suffix
        let mut column_name_counts: HashMap<String, usize> = HashMap::new();
        let mut display_names = Vec::new();
        let columns: Vec<String> = stmt_columns_vec
            .iter()
            .map(|c| {
                let base_name = SqlxColumn::name(c).to_string();
                display_names.push(base_name.clone());
                let count = column_name_counts.entry(base_name.clone()).or_insert(0);
                *count += 1;
                if *count == 1 {
                    base_name
                } else {
                    format!("{}_{}", base_name, count)
                }
            })
            .collect();

        // Extract column types and categorize them once upfront
        let mut column_types = HashMap::new();

        // Pre-compute column types once
        let mut column_name_counts_reset: HashMap<String, usize> = HashMap::new();
        let col_type_map: Vec<ColType> = stmt_columns_vec
            .iter()
            .map(|col: &sqlx::mysql::MySqlColumn| {
                let base_name = SqlxColumn::name(col).to_string();
                let type_name = col.type_info().name().to_uppercase();

                let count = column_name_counts_reset
                    .entry(base_name.clone())
                    .or_insert(0);
                *count += 1;
                let col_name = if *count == 1 {
                    base_name
                } else {
                    format!("{}_{}", base_name, count)
                };
                column_types.insert(col_name, type_name.clone());

                match type_name.as_str() {
                    "DATETIME" | "TIMESTAMP" => ColType::DateTime,
                    "DATE" => ColType::Date,
                    "TIME" => ColType::Time,
                    "TINYINT" | "SMALLINT" | "MEDIUMINT" | "INT" | "BIGINT" => ColType::Integer,
                    "FLOAT" | "DOUBLE" | "DECIMAL" => ColType::Float,
                    "BOOLEAN" | "BOOL" => ColType::Boolean,
                    "VARCHAR" | "CHAR" | "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT"
                    | "ENUM" | "SET" => ColType::String,
                    "BLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY" | "VARBINARY" | "TINYBLOB" => {
                        ColType::Blob
                    }
                    _ => ColType::Unknown,
                }
            })
            .collect();

        let mut result_rows = Vec::with_capacity(rows.len());

        // Process rows only if there are any (col_type_map will be empty for empty result set)
        if !rows.is_empty() && !col_type_map.is_empty() {
            for row in rows {
                let mut row_map = HashMap::with_capacity(columns.len());
                for (i, col) in columns.iter().enumerate() {
                    let value = match col_type_map[i] {
                        ColType::DateTime => row
                            .try_get::<NaiveDateTime, _>(i)
                            .map(|v| serde_json::json!(v.format("%Y-%m-%d %H:%M:%S").to_string()))
                            .unwrap_or(serde_json::Value::Null),
                        ColType::Date => row
                            .try_get::<NaiveDate, _>(i)
                            .map(|v| serde_json::json!(v.format("%Y-%m-%d").to_string()))
                            .unwrap_or(serde_json::Value::Null),
                        ColType::Time => row
                            .try_get::<NaiveTime, _>(i)
                            .map(|v| serde_json::json!(v.format("%H:%M:%S").to_string()))
                            .unwrap_or(serde_json::Value::Null),
                        ColType::Integer => row
                            .try_get::<i64, _>(i)
                            .map(|v| serde_json::json!(v))
                            .unwrap_or(serde_json::Value::Null),
                        ColType::Float => row
                            .try_get::<f64, _>(i)
                            .map(|v| serde_json::json!(v))
                            .unwrap_or(serde_json::Value::Null),
                        ColType::Boolean => row
                            .try_get::<bool, _>(i)
                            .map(|v| serde_json::json!(v))
                            .unwrap_or(serde_json::Value::Null),
                        ColType::String => row
                            .try_get::<String, _>(i)
                            .map(|v| serde_json::json!(v))
                            .unwrap_or(serde_json::Value::Null),
                        ColType::Blob => row
                            .try_get::<Vec<u8>, _>(i)
                            .map(|v| serde_json::json!(format!("[BLOB {} bytes]", v.len())))
                            .unwrap_or(serde_json::Value::Null),
                        ColType::Unknown => row
                            .try_get::<String, _>(i)
                            .map(|v| serde_json::json!(v))
                            .or_else(|_| {
                                row.try_get::<Vec<u8>, _>(i).map(|v| {
                                    serde_json::json!(format!("[BINARY {} bytes]", v.len()))
                                })
                            })
                            .unwrap_or(serde_json::Value::Null),
                    };
                    row_map.insert(col.clone(), value);
                }
                result_rows.push(row_map);
            }
        }

        Ok(QueryResult {
            columns,
            column_display_names: Some(display_names),
            column_types: Some(column_types),
            rows: result_rows,
            rows_affected: None,
            execution_time,
            final_query: None,
        })
    }

    async fn get_databases(&mut self) -> Result<Vec<Database>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let rows = sqlx::query("SHOW DATABASES").fetch_all(pool).await?;

        let databases = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get(0).unwrap_or_default();
                Database { name }
            })
            .collect();

        Ok(databases)
    }

    async fn get_tables(&mut self, database: &str) -> Result<Vec<Table>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let query = format!(
            "SELECT 
                TABLE_NAME as name, 
                COALESCE(DATA_LENGTH + INDEX_LENGTH, 0) as size_bytes
            FROM information_schema.TABLES 
            WHERE TABLE_SCHEMA = '{}' AND TABLE_TYPE = 'BASE TABLE'
            ORDER BY TABLE_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let tables = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let size_bytes: Option<i64> = row.try_get("size_bytes").ok();

                Table {
                    name,
                    schema: None,
                    size_bytes: size_bytes.map(|v| if v >= 0 { v as u64 } else { 0 }),
                }
            })
            .collect();

        Ok(tables)
    }

    async fn get_table_schema(&mut self, database: &str, table: &str) -> Result<TableSchema> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let query = format!("DESCRIBE `{}`.`{}`", database, table);
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let columns = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("Field").unwrap_or_default();
                let data_type: String = row.try_get("Type").unwrap_or_default();
                let nullable: String = row.try_get("Null").unwrap_or_default();
                let key: String = row.try_get("Key").unwrap_or_default();
                let extra: String = row.try_get("Extra").unwrap_or_default();
                let default: Option<String> = row.try_get("Default").ok();

                Column {
                    name,
                    data_type,
                    nullable: nullable == "YES",
                    default_value: default,
                    is_primary_key: key == "PRI",
                    is_auto_increment: extra.contains("auto_increment"),
                }
            })
            .collect();

        // Fetch indexes
        let index_query = format!(
            "SELECT INDEX_NAME, COLUMN_NAME, NON_UNIQUE, INDEX_TYPE, COLLATION, NULLABLE, INDEX_COMMENT
             FROM INFORMATION_SCHEMA.STATISTICS 
             WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}' 
             ORDER BY INDEX_NAME, SEQ_IN_INDEX",
            database, table
        );
        let index_rows = sqlx::query(&index_query).fetch_all(pool).await?;

        let mut indexes_map: std::collections::HashMap<
            String,
            (
                Vec<String>,
                bool,
                Option<String>,
                Option<bool>,
                Option<bool>,
                Option<String>,
            ),
        > = std::collections::HashMap::new();
        for row in index_rows {
            let index_name: String = row.try_get("INDEX_NAME").unwrap_or_default();
            let column_name: String = row.try_get("COLUMN_NAME").unwrap_or_default();
            let non_unique: i32 = row.try_get("NON_UNIQUE").unwrap_or(1);
            let index_type: Option<String> = row.try_get("INDEX_TYPE").ok();
            let collation: Option<String> = row.try_get("COLLATION").ok();
            let nullable: Option<String> = row.try_get("NULLABLE").ok();
            let comment: Option<String> = row.try_get("INDEX_COMMENT").ok();

            let ascending = collation.as_ref().map(|c| c == "A");
            let is_nullable = nullable.as_ref().map(|n| n == "YES");

            indexes_map
                .entry(index_name.clone())
                .or_insert((
                    Vec::new(),
                    non_unique == 0,
                    index_type,
                    ascending,
                    is_nullable,
                    comment,
                ))
                .0
                .push(column_name);
        }

        let indexes: Vec<Index> = indexes_map
            .into_iter()
            .map(
                |(name, (columns, is_unique, index_type, ascending, nullable, extra))| Index {
                    name,
                    columns,
                    is_unique,
                    index_type,
                    ascending,
                    nullable,
                    extra,
                },
            )
            .collect();

        // Fetch foreign keys
        let fk_query = format!(
            "SELECT 
                kcu.CONSTRAINT_NAME,
                kcu.COLUMN_NAME,
                kcu.REFERENCED_TABLE_NAME,
                kcu.REFERENCED_COLUMN_NAME,
                rc.UPDATE_RULE,
                rc.DELETE_RULE,
                kcu.TABLE_SCHEMA as owner
             FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE kcu
             LEFT JOIN INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS rc
                ON kcu.CONSTRAINT_NAME = rc.CONSTRAINT_NAME
                AND kcu.CONSTRAINT_SCHEMA = rc.CONSTRAINT_SCHEMA
             WHERE kcu.TABLE_SCHEMA = '{}' AND kcu.TABLE_NAME = '{}' AND kcu.REFERENCED_TABLE_NAME IS NOT NULL",
            database, table
        );
        let fk_rows = sqlx::query(&fk_query).fetch_all(pool).await?;

        let foreign_keys: Vec<ForeignKey> = fk_rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("CONSTRAINT_NAME").unwrap_or_default();
                let column: String = row.try_get("COLUMN_NAME").unwrap_or_default();
                let referenced_table: String =
                    row.try_get("REFERENCED_TABLE_NAME").unwrap_or_default();
                let referenced_column: String =
                    row.try_get("REFERENCED_COLUMN_NAME").unwrap_or_default();
                let on_update: Option<String> = row.try_get("UPDATE_RULE").ok();
                let on_delete: Option<String> = row.try_get("DELETE_RULE").ok();
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
        database: &str,
        table: &str,
    ) -> Result<Vec<TableRelationship>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        let query = format!(
            "SELECT 
                kcu.CONSTRAINT_NAME as constraint_name,
                kcu.TABLE_NAME as table_name,
                kcu.COLUMN_NAME as column_name,
                kcu.REFERENCED_TABLE_NAME as referenced_table_name,
                kcu.REFERENCED_COLUMN_NAME as referenced_column_name,
                kcu.TABLE_SCHEMA as owner,
                rc.UPDATE_RULE as update_rule,
                rc.DELETE_RULE as delete_rule
            FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE kcu
            LEFT JOIN INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS rc
                ON kcu.CONSTRAINT_NAME = rc.CONSTRAINT_NAME
                AND kcu.CONSTRAINT_SCHEMA = rc.CONSTRAINT_SCHEMA
            WHERE kcu.TABLE_SCHEMA = '{}' 
            AND (kcu.TABLE_NAME = '{}' OR kcu.REFERENCED_TABLE_NAME = '{}')
            AND kcu.REFERENCED_TABLE_NAME IS NOT NULL",
            database, table, table
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
                let owner: Option<String> = row.try_get("owner").ok();
                let on_update: Option<String> = row.try_get("update_rule").ok();
                let on_delete: Option<String> = row.try_get("delete_rule").ok();

                let relationship_type = if table_name == table {
                    "FOREIGN_KEY".to_string()
                } else {
                    "REFERENCED_BY".to_string()
                };

                TableRelationship {
                    constraint_name,
                    table_name,
                    column_name,
                    referenced_table_name,
                    referenced_column_name,
                    relationship_type,
                    owner,
                    ref_object_type: Some("TABLE".to_string()),
                    on_delete,
                    on_update,
                }
            })
            .collect();

        Ok(relationships)
    }

    async fn get_views(&mut self, database: &str, _schema: Option<&str>) -> Result<Vec<View>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        MySqlMetadataOps::get_views(pool, database).await
    }

    async fn get_indexes(&mut self, database: &str, _schema: Option<&str>) -> Result<Vec<DbIndex>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        MySqlMetadataOps::get_indexes(pool, database).await
    }

    async fn get_procedures(
        &mut self,
        database: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<Procedure>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        MySqlMetadataOps::get_procedures(pool, database).await
    }

    async fn get_procedure_source(
        &mut self,
        database: &str,
        procedure_name: &str,
        procedure_type: Option<String>,
        _schema: Option<String>,
    ) -> Result<String> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        // For MySQL, use INFORMATION_SCHEMA to get routine definition
        let routine_type = match procedure_type.as_deref() {
            Some("PROCEDURE") => "PROCEDURE",
            _ => "FUNCTION",
        };

        let query = format!(
            "SELECT ROUTINE_DEFINITION FROM INFORMATION_SCHEMA.ROUTINES 
             WHERE ROUTINE_SCHEMA = '{}' AND ROUTINE_NAME = '{}' AND ROUTINE_TYPE = '{}'",
            database, procedure_name, routine_type
        );

        match sqlx::query_scalar::<_, String>(&query)
            .fetch_optional(pool)
            .await
        {
            Ok(Some(source)) => Ok(source),
            Ok(None) => Ok("-- Source code not available".to_string()),
            Err(e) => {
                tracing::warn!(
                    "Failed to retrieve procedure source for '{}': {}",
                    procedure_name,
                    e
                );
                Ok(format!(
                    "-- Error retrieving source: {}\n-- {}",
                    e, procedure_name
                ))
            }
        }
    }

    async fn get_triggers(
        &mut self,
        database: &str,
        _schema: Option<&str>,
    ) -> Result<Vec<Trigger>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        MySqlMetadataOps::get_triggers(pool, database).await
    }

    async fn get_events(&mut self, database: &str, _schema: Option<&str>) -> Result<Vec<Event>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        MySqlMetadataOps::get_events(pool, database).await
    }

    async fn get_table_statistics(
        &mut self,
        database: &str,
        table: &str,
    ) -> Result<TableStatistics> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        let query = format!(
            "SELECT 
                TABLE_ROWS as row_count,
                AVG_ROW_LENGTH as avg_row_length,
                DATA_LENGTH as data_length,
                MAX_DATA_LENGTH as max_data_length,
                DATA_FREE as data_free,
                INDEX_LENGTH as index_length,
                ROW_FORMAT as row_format,
                CREATE_TIME as create_time,
                UPDATE_TIME as update_time,
                CHECK_TIME as check_time,
                TABLE_COLLATION as collation,
                CHECKSUM as checksum,
                ENGINE as engine,
                TABLE_COMMENT as comment
            FROM information_schema.TABLES 
            WHERE TABLE_SCHEMA = '{}' AND TABLE_NAME = '{}'",
            database, table
        );

        let row = sqlx::query(&query).fetch_one(pool).await?;

        // Helper function to get numeric value
        let get_numeric = |col: &str| -> Option<i64> {
            row.try_get::<Option<u64>, _>(col)
                .ok()
                .flatten()
                .map(|v| v as i64)
                .or_else(|| row.try_get::<Option<i64>, _>(col).ok().flatten())
        };

        let statistics = TableStatistics {
            row_count: get_numeric("row_count"),
            avg_row_length: get_numeric("avg_row_length"),
            data_length: get_numeric("data_length"),
            max_data_length: get_numeric("max_data_length"),
            data_free: get_numeric("data_free"),
            index_length: get_numeric("index_length"),
            row_format: row
                .try_get::<Option<String>, _>("row_format")
                .ok()
                .flatten(),
            create_time: row
                .try_get::<Option<chrono::NaiveDateTime>, _>("create_time")
                .ok()
                .flatten()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.f").to_string()),
            update_time: row
                .try_get::<Option<chrono::NaiveDateTime>, _>("update_time")
                .ok()
                .flatten()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.f").to_string()),
            check_time: row
                .try_get::<Option<chrono::NaiveDateTime>, _>("check_time")
                .ok()
                .flatten()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S%.f").to_string()),
            collation: row.try_get::<Option<String>, _>("collation").ok().flatten(),
            checksum: get_numeric("checksum").map(|c| c.to_string()),
            engine: row.try_get::<Option<String>, _>("engine").ok().flatten(),
            comment: row.try_get::<Option<String>, _>("comment").ok().flatten(),
            table_size: None,
            pages: None,
        };

        Ok(statistics)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
