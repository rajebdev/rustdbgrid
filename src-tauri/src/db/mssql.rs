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

// Enum for pre-computed column types
#[derive(Clone, Copy)]
enum MssqlColType {
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

// Optimized helper function using pre-computed type
fn row_value_to_json_typed(row: &Row, index: usize, col_type: MssqlColType) -> serde_json::Value {
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

        // Pre-compute column types once
        let col_type_map: Vec<MssqlColType> = rows[0]
            .columns()
            .iter()
            .map(|col| {
                let col_name = col.name().to_string();
                let type_name = format!("{:?}", col.column_type()).to_uppercase();
                let col_type = match type_name.as_str() {
                    t if t.contains("VARCHAR")
                        || t.contains("CHAR")
                        || t.contains("TEXT")
                        || t.contains("NVARCHAR")
                        || t.contains("NCHAR")
                        || t.contains("NTEXT") =>
                    {
                        MssqlColType::String
                    }
                    t if t.contains("BIGINT") || t.contains("INT8") => MssqlColType::Int64,
                    t if t.contains("SMALLINT") || t.contains("INT2") => MssqlColType::Int16,
                    t if t.contains("TINYINT") => MssqlColType::UInt8,
                    t if t.contains("INT") || t.contains("INT4") => MssqlColType::Int32,
                    t if t.contains("REAL") || t.contains("FLOAT4") => MssqlColType::Float32,
                    t if t.contains("FLOAT") || t.contains("DOUBLE") || t.contains("FLOAT8") => {
                        MssqlColType::Float64
                    }
                    t if t.contains("BIT") || t.contains("BOOL") => MssqlColType::Boolean,
                    t if t.contains("UNIQUEIDENTIFIER") || t.contains("UUID") => MssqlColType::Uuid,
                    t if t.contains("DATETIME")
                        || t.contains("TIMESTAMP")
                        || t.contains("SMALLDATETIME") =>
                    {
                        MssqlColType::DateTime
                    }
                    t if t.contains("DATE") => MssqlColType::Date,
                    t if t.contains("TIME") => MssqlColType::Time,
                    t if t.contains("BINARY") || t.contains("VARBINARY") || t.contains("IMAGE") => {
                        MssqlColType::Binary
                    }
                    t if t.contains("DECIMAL")
                        || t.contains("NUMERIC")
                        || t.contains("MONEY")
                        || t.contains("SMALLMONEY") =>
                    {
                        MssqlColType::Decimal
                    }
                    _ => MssqlColType::Unknown,
                };
                column_types.insert(col_name, type_name);
                col_type
            })
            .collect();

        let mut result_rows = Vec::with_capacity(rows.len());

        for row in rows {
            let mut row_map = HashMap::with_capacity(columns.len());
            for (i, col_name) in columns.iter().enumerate() {
                // Use pre-computed type for faster extraction
                let value = row_value_to_json_typed(&row, i, col_type_map[i]);
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
            "SELECT 
                t.TABLE_NAME as name, 
                t.TABLE_SCHEMA as schema_name,
                COALESCE(SUM(p.rows * 8 * 1024), 0) as size_bytes
            FROM [{database}].INFORMATION_SCHEMA.TABLES t
            LEFT JOIN [{database}].sys.tables st ON t.TABLE_NAME = st.name
            LEFT JOIN [{database}].sys.partitions p ON st.object_id = p.object_id
            WHERE t.TABLE_TYPE = 'BASE TABLE' AND (p.index_id = 0 OR p.index_id = 1 OR p.index_id IS NULL)
            GROUP BY t.TABLE_NAME, t.TABLE_SCHEMA
            ORDER BY t.TABLE_NAME"
        );

        let stream = conn.query(query, &[]).await?;
        let rows = stream.into_first_result().await?;

        let tables = rows
            .iter()
            .filter_map(|row| {
                let name = row.get::<&str, _>("name")?.to_string();
                let schema = row.get::<&str, _>("schema_name").map(|s| s.to_string());
                let size_bytes: Option<i64> = row.get("size_bytes");
                Some(Table {
                    name,
                    schema,
                    size_bytes: size_bytes.map(|v| if v >= 0 { v as u64 } else { 0 }),
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
        // For first page, use TOP for better performance
        if offset == 0 {
            let query = format!("SELECT TOP {} * FROM [{database}].[dbo].[{table}]", limit);
            return self.execute_query(&query).await;
        }

        // For subsequent pages, we need ORDER BY with OFFSET-FETCH
        // Use a subquery with ROW_NUMBER() to avoid ORDER BY issues
        let query = format!(
            "SELECT * FROM (SELECT ROW_NUMBER() OVER (ORDER BY (SELECT 0)) AS __RowNum, * FROM [{database}].[dbo].[{table}]) AS __Paginated WHERE __RowNum > {} AND __RowNum <= {}",
            offset, offset + limit
        );
        self.execute_query(&query).await
    }

    async fn get_views(&mut self, database: &str, schema: Option<&str>) -> Result<Vec<View>> {
        let schema_filter = if let Some(s) = schema {
            format!("AND s.name = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT s.name as schema_name, v.name as view_name
             FROM [{database}].sys.views v
             INNER JOIN [{database}].sys.schemas s ON v.schema_id = s.schema_id
             WHERE 1=1 {}
             ORDER BY s.name, v.name",
            schema_filter
        );

        let result = self.execute_query(&query).await?;
        let mut views = Vec::new();

        for row in result.rows {
            if let (Some(schema_val), Some(name_val)) =
                (row.get("schema_name"), row.get("view_name"))
            {
                if let (Some(schema_str), Some(name_str)) = (schema_val.as_str(), name_val.as_str())
                {
                    views.push(View {
                        schema: Some(schema_str.to_string()),
                        name: name_str.to_string(),
                    });
                }
            }
        }

        Ok(views)
    }

    async fn get_indexes(&mut self, database: &str, schema: Option<&str>) -> Result<Vec<DbIndex>> {
        let schema_filter = if let Some(s) = schema {
            format!("AND s.name = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT 
                t.name as table_name,
                i.name as index_name,
                i.is_unique
             FROM [{database}].sys.indexes i
             INNER JOIN [{database}].sys.tables t ON i.object_id = t.object_id
             INNER JOIN [{database}].sys.schemas s ON t.schema_id = s.schema_id
             WHERE i.name IS NOT NULL {}
             ORDER BY s.name, t.name, i.name",
            schema_filter
        );

        let result = self.execute_query(&query).await?;
        let mut indexes = Vec::new();

        for row in result.rows {
            if let (Some(table_val), Some(name_val), Some(unique_val)) = (
                row.get("table_name"),
                row.get("index_name"),
                row.get("is_unique"),
            ) {
                if let (Some(table_str), Some(name_str)) = (table_val.as_str(), name_val.as_str()) {
                    let is_unique = unique_val.as_bool().unwrap_or(false);
                    indexes.push(DbIndex {
                        name: name_str.to_string(),
                        table_name: table_str.to_string(),
                        columns: vec![],
                        is_unique,
                        index_type: None,
                    });
                }
            }
        }

        Ok(indexes)
    }

    async fn get_procedures(
        &mut self,
        database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Procedure>> {
        let schema_filter = if let Some(s) = schema {
            format!("AND s.name = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT 
                p.name as procedure_name,
                s.name as schema_name,
                CASE p.type
                    WHEN 'P' THEN 'PROCEDURE'
                    WHEN 'FN' THEN 'FUNCTION'
                    WHEN 'IF' THEN 'FUNCTION'
                    WHEN 'TF' THEN 'FUNCTION'
                    ELSE 'PROCEDURE'
                END as type
             FROM [{database}].sys.procedures p
             INNER JOIN [{database}].sys.schemas s ON p.schema_id = s.schema_id
             WHERE 1=1 {}
             UNION ALL
             SELECT 
                o.name as procedure_name,
                s.name as schema_name,
                'FUNCTION' as type
             FROM [{database}].sys.objects o
             INNER JOIN [{database}].sys.schemas s ON o.schema_id = s.schema_id
             WHERE o.type IN ('FN', 'IF', 'TF') {}
             ORDER BY procedure_name",
            schema_filter, schema_filter
        );

        let result = self.execute_query(&query).await?;
        let mut procedures = Vec::new();

        for row in result.rows {
            if let (Some(name_val), Some(schema_val), Some(type_val)) = (
                row.get("procedure_name"),
                row.get("schema_name"),
                row.get("type"),
            ) {
                if let (Some(name_str), Some(schema_str), Some(type_str)) =
                    (name_val.as_str(), schema_val.as_str(), type_val.as_str())
                {
                    procedures.push(Procedure {
                        name: name_str.to_string(),
                        schema: Some(schema_str.to_string()),
                        procedure_type: Some(type_str.to_string()),
                    });
                }
            }
        }

        Ok(procedures)
    }

    async fn get_triggers(&mut self, database: &str, schema: Option<&str>) -> Result<Vec<Trigger>> {
        let schema_filter = if let Some(s) = schema {
            format!("AND s.name = '{}'", s)
        } else {
            String::new()
        };

        let query = format!(
            "SELECT 
                tr.name as trigger_name,
                t.name as table_name
             FROM [{database}].sys.triggers tr
             INNER JOIN [{database}].sys.tables t ON tr.parent_id = t.object_id
             INNER JOIN [{database}].sys.schemas s ON t.schema_id = s.schema_id
             WHERE tr.is_ms_shipped = 0 {}
             ORDER BY s.name, t.name, tr.name",
            schema_filter
        );

        let result = self.execute_query(&query).await?;
        let mut triggers = Vec::new();

        for row in result.rows {
            if let (Some(name_val), Some(table_val)) =
                (row.get("trigger_name"), row.get("table_name"))
            {
                if let (Some(name_str), Some(table_str)) = (name_val.as_str(), table_val.as_str()) {
                    triggers.push(Trigger {
                        name: name_str.to_string(),
                        table_name: table_str.to_string(),
                        timing: String::new(),
                        event: String::new(),
                    });
                }
            }
        }

        Ok(triggers)
    }
}
