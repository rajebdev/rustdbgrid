use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::{Column as SqlxColumn, PgPool, Row, TypeInfo};
use std::collections::HashMap;
use std::time::Instant;

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
        let mut result_rows = Vec::new();

        for row in rows {
            let mut row_map = HashMap::new();
            for (i, col) in columns.iter().enumerate() {
                let col_info = &row.columns()[i];
                let type_name = col_info.type_info().name();

                let value = if type_name == "TIMESTAMP" || type_name == "TIMESTAMPTZ" {
                    if let Ok(v) = row.try_get::<NaiveDateTime, _>(i) {
                        serde_json::json!(v.format("%Y-%m-%d %H:%M:%S").to_string())
                    } else {
                        serde_json::Value::Null
                    }
                } else if type_name == "DATE" {
                    if let Ok(v) = row.try_get::<NaiveDate, _>(i) {
                        serde_json::json!(v.format("%Y-%m-%d").to_string())
                    } else {
                        serde_json::Value::Null
                    }
                } else if type_name == "TIME" {
                    if let Ok(v) = row.try_get::<NaiveTime, _>(i) {
                        serde_json::json!(v.format("%H:%M:%S").to_string())
                    } else {
                        serde_json::Value::Null
                    }
                } else if let Ok(v) = row.try_get::<i64, _>(i) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<i32, _>(i) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<i16, _>(i) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<f64, _>(i) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<f32, _>(i) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<bool, _>(i) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<String, _>(i) {
                    serde_json::json!(v)
                } else if let Ok(v) = row.try_get::<Vec<u8>, _>(i) {
                    serde_json::json!(format!("[BYTEA {} bytes]", v.len()))
                } else {
                    serde_json::Value::Null
                };
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
