use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use std::collections::HashMap;
use std::time::Instant;
use tiberius::{AuthMethod, Config, Row};

pub struct MSSQLConnection {
    pool: Option<Pool<ConnectionManager>>,
}

impl MSSQLConnection {
    pub fn new() -> Self {
        Self { pool: None }
    }
}

impl Default for MSSQLConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseConnection for MSSQLConnection {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()> {
        let mut tiberius_config = Config::new();

        tiberius_config.host(&config.host);
        tiberius_config.port(config.port);

        if let Some(database) = &config.database {
            tiberius_config.database(database);
        }

        let auth = AuthMethod::sql_server(
            config.username.as_ref().unwrap_or(&"sa".to_string()),
            config.password.as_ref().unwrap_or(&"".to_string()),
        );
        tiberius_config.authentication(auth);

        // Configure encryption based on SSL setting
        if config.ssl {
            // Enable encryption and trust server certificate
            tiberius_config.encryption(tiberius::EncryptionLevel::Required);
            tiberius_config.trust_cert();
        } else {
            // Disable encryption completely to avoid TLS handshake errors
            tiberius_config.encryption(tiberius::EncryptionLevel::NotSupported);
        }

        // Create connection pool
        let manager = ConnectionManager::new(tiberius_config);
        let pool = Pool::builder()
            .max_size(15)
            .build(manager)
            .await
            .map_err(|e| anyhow!("Failed to create connection pool: {}", e))?;

        self.pool = Some(pool);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(pool) = self.pool.take() {
            // Pool will be dropped and connections will be closed
            drop(pool);
        }
        Ok(())
    }

    async fn test_connection(&mut self) -> Result<bool> {
        if let Some(pool) = &self.pool {
            let mut conn = pool
                .get()
                .await
                .map_err(|e| anyhow!("Failed to get connection from pool: {}", e))?;
            conn.simple_query("SELECT 1").await?;
            Ok(true)
        } else {
            Err(anyhow!("Not connected"))
        }
    }

    async fn execute_query(&mut self, query: &str) -> Result<QueryResult> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        let mut conn = pool
            .get()
            .await
            .map_err(|e| anyhow!("Failed to get connection from pool: {}", e))?;
        let start = Instant::now();

        let stream = conn.query(query, &[]).await?;
        let rows = stream.into_first_result().await?;
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
            .map(|c| c.name().to_string())
            .collect();

        // Extract column types
        let mut column_types = HashMap::new();
        for col in rows[0].columns() {
            let col_name = col.name().to_string();
            let type_name = format!("{:?}", col.column_type()).to_uppercase();
            column_types.insert(col_name, type_name);
        }

        let mut result_rows = Vec::new();

        for row in rows {
            let mut row_map = HashMap::new();
            for (i, col_name) in columns.iter().enumerate() {
                let value = row_value_to_json(&row, i);
                row_map.insert(col_name.clone(), value);
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

        let mut conn = pool
            .get()
            .await
            .map_err(|e| anyhow!("Failed to get connection from pool: {}", e))?;

        let stream = conn
            .query(
                "SELECT name FROM sys.databases WHERE database_id > 4 ORDER BY name",
                &[],
            )
            .await?;
        let rows = stream.into_first_result().await?;

        let databases = rows
            .iter()
            .filter_map(|row| {
                row.get::<&str, _>(0).map(|name| Database {
                    name: name.to_string(),
                })
            })
            .collect();

        Ok(databases)
    }

    async fn get_tables(&mut self, database: &str) -> Result<Vec<Table>> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        let mut conn = pool
            .get()
            .await
            .map_err(|e| anyhow!("Failed to get connection from pool: {}", e))?;

        let query = format!(
            "SELECT TABLE_NAME as name, TABLE_SCHEMA as schema_name
            FROM [{database}].INFORMATION_SCHEMA.TABLES 
            WHERE TABLE_TYPE = 'BASE TABLE'
            ORDER BY TABLE_NAME"
        );

        let stream = conn.query(query, &[]).await?;
        let rows = stream.into_first_result().await?;

        let tables = rows
            .iter()
            .filter_map(|row| {
                let name = row.get::<&str, _>("name")?.to_string();
                let schema = row.get::<&str, _>("schema_name").map(|s| s.to_string());
                Some(Table {
                    name,
                    schema,
                    size_bytes: Some(0),
                })
            })
            .collect();

        Ok(tables)
    }

    async fn get_table_schema(&mut self, database: &str, table: &str) -> Result<TableSchema> {
        let pool = self.pool.as_ref().ok_or_else(|| anyhow!("Not connected"))?;

        let mut conn = pool
            .get()
            .await
            .map_err(|e| anyhow!("Failed to get connection from pool: {}", e))?;

        let query = format!(
            "SELECT 
                COLUMN_NAME as column_name,
                DATA_TYPE as data_type,
                IS_NULLABLE as is_nullable,
                COLUMN_DEFAULT as column_default,
                CHARACTER_MAXIMUM_LENGTH as max_length,
                COLUMNPROPERTY(object_id(TABLE_SCHEMA+'.'+TABLE_NAME), COLUMN_NAME, 'IsIdentity') as is_identity
            FROM [{database}].INFORMATION_SCHEMA.COLUMNS 
            WHERE TABLE_NAME = '{table}'
            ORDER BY ORDINAL_POSITION"
        );

        let stream = conn.query(query, &[]).await?;
        let rows = stream.into_first_result().await?;

        let columns = rows
            .iter()
            .filter_map(|row| {
                let name = row.get::<&str, _>("column_name")?.to_string();
                let data_type = row.get::<&str, _>("data_type")?.to_string();
                let nullable = row.get::<&str, _>("is_nullable")?.to_string();
                let default: Option<String> =
                    row.get::<&str, _>("column_default").map(|s| s.to_string());
                let max_length: Option<i32> = row.get("max_length");
                let is_identity: Option<i32> = row.get("is_identity");

                let data_type_display = if let Some(len) = max_length {
                    if len > 0 {
                        format!("{}({})", data_type, len)
                    } else {
                        data_type.clone()
                    }
                } else {
                    data_type.clone()
                };

                Some(Column {
                    name,
                    data_type: data_type_display,
                    nullable: nullable == "YES",
                    default_value: default,
                    is_primary_key: false, // We'd need to query sys.key_constraints for this
                    is_auto_increment: is_identity.unwrap_or(0) == 1,
                })
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
            "SELECT * FROM [{database}].[dbo].[{table}] ORDER BY (SELECT NULL) OFFSET {offset} ROWS FETCH NEXT {limit} ROWS ONLY"
        );
        self.execute_query(&query).await
    }
}

// Helper function to convert tiberius row values to JSON
fn row_value_to_json(row: &Row, index: usize) -> serde_json::Value {
    // Try different types and return appropriate JSON value

    // String types
    if let Some(v) = row.try_get::<&str, _>(index).ok().flatten() {
        return serde_json::json!(v);
    }

    // Integer types
    if let Some(v) = row.try_get::<i32, _>(index).ok().flatten() {
        return serde_json::json!(v);
    }
    if let Some(v) = row.try_get::<i64, _>(index).ok().flatten() {
        return serde_json::json!(v);
    }
    if let Some(v) = row.try_get::<i16, _>(index).ok().flatten() {
        return serde_json::json!(v);
    }
    if let Some(v) = row.try_get::<u8, _>(index).ok().flatten() {
        return serde_json::json!(v);
    }

    // Float types
    if let Some(v) = row.try_get::<f32, _>(index).ok().flatten() {
        return serde_json::json!(v);
    }
    if let Some(v) = row.try_get::<f64, _>(index).ok().flatten() {
        return serde_json::json!(v);
    }

    // Boolean
    if let Some(v) = row.try_get::<bool, _>(index).ok().flatten() {
        return serde_json::json!(v);
    }

    // UUID/GUID
    if let Some(v) = row.try_get::<tiberius::Uuid, _>(index).ok().flatten() {
        return serde_json::json!(v.to_string());
    }

    // Date and time types
    if let Some(v) = row
        .try_get::<chrono::NaiveDateTime, _>(index)
        .ok()
        .flatten()
    {
        return serde_json::json!(v.format("%Y-%m-%d %H:%M:%S").to_string());
    }
    if let Some(v) = row.try_get::<chrono::NaiveDate, _>(index).ok().flatten() {
        return serde_json::json!(v.format("%Y-%m-%d").to_string());
    }
    if let Some(v) = row.try_get::<chrono::NaiveTime, _>(index).ok().flatten() {
        return serde_json::json!(v.format("%H:%M:%S").to_string());
    }

    // Binary data
    if let Some(v) = row.try_get::<&[u8], _>(index).ok().flatten() {
        return serde_json::json!(format!("[BINARY {} bytes]", v.len()));
    }

    // Numeric/Decimal (using bigdecimal from tiberius features)
    if let Some(v) = row
        .try_get::<bigdecimal::BigDecimal, _>(index)
        .ok()
        .flatten()
    {
        return serde_json::json!(v.to_string());
    }

    // Default to null if we can't convert
    serde_json::Value::Null
}
