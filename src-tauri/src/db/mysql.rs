use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{Column as SqlxColumn, MySqlPool, Row};
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
                let value: Option<String> = row.try_get(i).ok();
                row_map.insert(col.clone(), serde_json::json!(value));
            }
            result_rows.push(row_map);
        }

        Ok(QueryResult {
            columns,
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
        let query = format!(
            "SELECT * FROM `{}`.`{}` LIMIT {} OFFSET {}",
            database, table, limit, offset
        );
        self.execute_query(&query).await
    }
}
