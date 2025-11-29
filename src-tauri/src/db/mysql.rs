use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::{Column as SqlxColumn, MySqlPool, Row, TypeInfo};
use std::collections::HashMap;
use std::time::Instant;

pub struct MySQLConnection {
    pool: Option<MySqlPool>,
}

impl MySQLConnection {
    pub fn new() -> Self {
        Self { pool: None }
    }

    // Helper method to get views for MySQL
    pub async fn get_views_impl(&mut self, database: &str) -> Result<Vec<View>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let query = format!(
            "SELECT TABLE_NAME as name 
            FROM information_schema.VIEWS 
            WHERE TABLE_SCHEMA = '{}' 
            ORDER BY TABLE_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let views = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                View { name, schema: None }
            })
            .collect();

        Ok(views)
    }

    // Helper method to get indexes for MySQL
    pub async fn get_indexes_impl(&mut self, database: &str) -> Result<Vec<DbIndex>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let query = format!(
            "SELECT DISTINCT
                INDEX_NAME as name,
                TABLE_NAME as table_name,
                NON_UNIQUE,
                INDEX_TYPE as index_type
            FROM information_schema.STATISTICS 
            WHERE TABLE_SCHEMA = '{}' 
            ORDER BY TABLE_NAME, INDEX_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let indexes = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let table_name: String = row.try_get("table_name").unwrap_or_default();
                let non_unique: i32 = row.try_get("NON_UNIQUE").unwrap_or(1);
                let index_type: Option<String> = row.try_get("index_type").ok();
                DbIndex {
                    name,
                    table_name,
                    columns: vec![],
                    is_unique: non_unique == 0,
                    index_type,
                }
            })
            .collect();

        Ok(indexes)
    }

    // Helper method to get procedures for MySQL
    pub async fn get_procedures_impl(&mut self, database: &str) -> Result<Vec<Procedure>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let query = format!(
            "SELECT 
                ROUTINE_NAME as name,
                ROUTINE_TYPE as procedure_type
            FROM information_schema.ROUTINES 
            WHERE ROUTINE_SCHEMA = '{}' 
            ORDER BY ROUTINE_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let procedures = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let procedure_type: Option<String> = row.try_get("procedure_type").ok();
                Procedure {
                    name,
                    schema: None,
                    procedure_type,
                }
            })
            .collect();

        Ok(procedures)
    }

    // Helper method to get triggers for MySQL
    pub async fn get_triggers_impl(&mut self, database: &str) -> Result<Vec<Trigger>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let query = format!(
            "SELECT 
                TRIGGER_NAME as name,
                EVENT_OBJECT_TABLE as table_name,
                EVENT_MANIPULATION as event,
                ACTION_TIMING as timing
            FROM information_schema.TRIGGERS 
            WHERE TRIGGER_SCHEMA = '{}' 
            ORDER BY TRIGGER_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let triggers = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let table_name: String = row.try_get("table_name").unwrap_or_default();
                let event: String = row.try_get("event").unwrap_or_default();
                let timing: String = row.try_get("timing").unwrap_or_default();
                Trigger {
                    name,
                    table_name,
                    event,
                    timing,
                }
            })
            .collect();

        Ok(triggers)
    }

    // Helper method to get events for MySQL
    pub async fn get_events_impl(&mut self, database: &str) -> Result<Vec<Event>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;
        let query = format!(
            "SELECT 
                EVENT_NAME as name,
                STATUS as status,
                INTERVAL_VALUE as interval_value,
                INTERVAL_FIELD as interval_field
            FROM information_schema.EVENTS 
            WHERE EVENT_SCHEMA = '{}' 
            ORDER BY EVENT_NAME",
            database
        );
        let rows = sqlx::query(&query).fetch_all(pool).await?;

        let events = rows
            .iter()
            .map(|row| {
                let name: String = row.try_get("name").unwrap_or_default();
                let status: Option<String> = row.try_get("status").ok();
                let interval_value: Option<String> =
                    row.try_get::<String, _>("interval_value").ok();
                let interval_field: Option<String> = row.try_get("interval_field").ok();
                Event {
                    name,
                    status,
                    interval_value,
                    interval_field,
                }
            })
            .collect();

        Ok(events)
    }
}

impl Default for MySQLConnection {
    fn default() -> Self {
        Self::new()
    }
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

        // Extract column types and categorize them once upfront
        let mut column_types = HashMap::new();

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

        // Pre-compute column types once
        let col_type_map: Vec<ColType> = rows[0]
            .columns()
            .iter()
            .map(|col| {
                let col_name = SqlxColumn::name(col).to_string();
                let type_name = col.type_info().name().to_uppercase();
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

        for row in rows {
            let mut row_map = HashMap::with_capacity(columns.len());
            for (i, col) in columns.iter().enumerate() {
                // Use pre-computed type - no string matching per row
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
                            row.try_get::<Vec<u8>, _>(i)
                                .map(|v| serde_json::json!(format!("[BINARY {} bytes]", v.len())))
                        })
                        .unwrap_or(serde_json::Value::Null),
                };
                row_map.insert(col.clone(), value);
            }
            result_rows.push(row_map);
        }

        Ok(QueryResult {
            columns,
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

                Column {
                    name,
                    data_type,
                    nullable: nullable == "YES",
                    default_value: None,
                    is_primary_key: key == "PRI",
                    is_auto_increment: extra.contains("auto_increment"),
                }
            })
            .collect();

        Ok(TableSchema {
            table_name: table.to_string(),
            columns,
            indexes: vec![],
            foreign_keys: vec![],
        })
    }

    async fn get_table_data(
        &mut self,
        database: &str,
        table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult> {
        // Check if table already includes database prefix (e.g., apps_config.jns_config)
        let query = if table.contains('.') {
            // Table already has database prefix, use it directly
            format!("SELECT * FROM {} LIMIT {} OFFSET {}", table, limit, offset)
        } else {
            // Table is simple name, add database prefix
            format!(
                "SELECT * FROM `{}`.`{}` LIMIT {} OFFSET {}",
                database, table, limit, offset
            )
        };
        self.execute_query(&query).await
    }

    async fn get_views(&mut self, database: &str) -> Result<Vec<View>> {
        self.get_views_impl(database).await
    }

    async fn get_indexes(&mut self, database: &str) -> Result<Vec<DbIndex>> {
        self.get_indexes_impl(database).await
    }

    async fn get_procedures(&mut self, database: &str) -> Result<Vec<Procedure>> {
        self.get_procedures_impl(database).await
    }

    async fn get_triggers(&mut self, database: &str) -> Result<Vec<Trigger>> {
        self.get_triggers_impl(database).await
    }

    async fn get_events(&mut self, database: &str) -> Result<Vec<Event>> {
        self.get_events_impl(database).await
    }
}
