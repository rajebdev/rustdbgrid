use crate::db::mssql::metadata_ops::MSSQLMetadataOps;
use crate::db::mssql::type_converter::{row_value_to_json_typed, MssqlColType};
use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use std::collections::HashMap;
use std::time::Instant;
use tiberius::{AuthMethod, Config};

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
            tiberius_config.encryption(tiberius::EncryptionLevel::Required);
            tiberius_config.trust_cert();
        } else {
            tiberius_config.encryption(tiberius::EncryptionLevel::NotSupported);
        }

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
                column_display_names: None,
                column_types: None,
                rows: vec![],
                rows_affected: None,
                execution_time,
                final_query: None,
            });
        }

        let mut column_name_counts: HashMap<String, usize> = HashMap::new();
        let mut display_names = Vec::new();
        let columns: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| {
                let base_name = c.name().to_string();
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

        let mut column_types = HashMap::new();
        let mut column_name_counts_reset: HashMap<String, usize> = HashMap::new();
        let col_type_map: Vec<MssqlColType> = rows[0]
            .columns()
            .iter()
            .map(|col| {
                let base_name = col.name().to_string();
                let type_name = format!("{:?}", col.column_type()).to_uppercase();

                let count = column_name_counts_reset
                    .entry(base_name.clone())
                    .or_insert(0);
                *count += 1;
                let col_name = if *count == 1 {
                    base_name
                } else {
                    format!("{}_{}", base_name, count)
                };
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
                let value = row_value_to_json_typed(&row, i, col_type_map[i]);
                row_map.insert(col_name.clone(), value);
            }
            result_rows.push(row_map);
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

        let (schema, table_name) = if table.contains('.') {
            let parts: Vec<&str> = table.split('.').collect();
            (parts[0], parts[1])
        } else {
            ("dbo", table)
        };

        let query = format!(
            "SELECT 
                COLUMN_NAME as column_name,
                DATA_TYPE as data_type,
                IS_NULLABLE as is_nullable,
                COLUMN_DEFAULT as column_default,
                CHARACTER_MAXIMUM_LENGTH as max_length,
                COLUMNPROPERTY(object_id(TABLE_SCHEMA+'.'+TABLE_NAME), COLUMN_NAME, 'IsIdentity') as is_identity
            FROM [{database}].INFORMATION_SCHEMA.COLUMNS 
            WHERE TABLE_NAME = '{table_name}' AND TABLE_SCHEMA = '{schema}'
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
                    is_primary_key: false,
                    is_auto_increment: is_identity.unwrap_or(0) == 1,
                })
            })
            .collect::<Vec<Column>>();

        let pk_query = format!(
            "SELECT COLUMN_NAME
            FROM [{database}].INFORMATION_SCHEMA.KEY_COLUMN_USAGE
            WHERE OBJECTPROPERTY(OBJECT_ID(CONSTRAINT_SCHEMA + '.' + QUOTENAME(CONSTRAINT_NAME)), 'IsPrimaryKey') = 1
            AND TABLE_NAME = '{table_name}' AND TABLE_SCHEMA = '{schema}'"
        );

        let pk_stream = conn.query(pk_query, &[]).await?;
        let pk_rows = pk_stream.into_first_result().await?;
        let pk_columns: Vec<String> = pk_rows
            .iter()
            .filter_map(|row| row.get::<&str, _>("COLUMN_NAME").map(|s| s.to_string()))
            .collect();

        let mut columns = columns;
        for col in columns.iter_mut() {
            if pk_columns.contains(&col.name) {
                col.is_primary_key = true;
            }
        }

        let idx_query = format!(
            "SELECT 
                i.name as index_name,
                COL_NAME(ic.object_id, ic.column_id) as column_name,
                i.is_unique
            FROM [{database}].sys.indexes i
            INNER JOIN [{database}].sys.index_columns ic ON i.object_id = ic.object_id AND i.index_id = ic.index_id
            WHERE i.object_id = OBJECT_ID('[{schema}].[{table_name}]', 'U')
            ORDER BY i.name, ic.key_ordinal"
        );

        let idx_stream = conn.query(idx_query, &[]).await?;
        let idx_rows = idx_stream.into_first_result().await?;

        let mut indexes_map: std::collections::HashMap<String, (Vec<String>, bool)> =
            std::collections::HashMap::new();
        for row in idx_rows.iter() {
            if let (Some(idx_name), Some(col_name)) = (
                row.get::<&str, _>("index_name"),
                row.get::<&str, _>("column_name"),
            ) {
                let is_unique: Option<bool> = row.get("is_unique");
                indexes_map
                    .entry(idx_name.to_string())
                    .or_insert((Vec::new(), is_unique.unwrap_or(false)))
                    .0
                    .push(col_name.to_string());
            }
        }

        let indexes: Vec<Index> = indexes_map
            .into_iter()
            .map(|(name, (columns, is_unique))| Index {
                name,
                columns,
                is_unique,
                index_type: None,
                ascending: Some(true),
                nullable: None,
                extra: None,
            })
            .collect();

        let fk_query = format!(
            "SELECT 
                fk.name as constraint_name,
                COL_NAME(fkc.parent_object_id, fkc.parent_column_id) as column_name,
                OBJECT_NAME(fkc.referenced_object_id) as referenced_table,
                COL_NAME(fkc.referenced_object_id, fkc.referenced_column_id) as referenced_column,
                fk.delete_referential_action_desc as delete_rule,
                fk.update_referential_action_desc as update_rule
            FROM [{database}].sys.foreign_keys fk
            INNER JOIN [{database}].sys.foreign_key_columns fkc ON fk.object_id = fkc.constraint_object_id
            WHERE fk.parent_object_id = OBJECT_ID('{table}')"
        );

        let fk_stream = conn.query(fk_query, &[]).await?;
        let fk_rows = fk_stream.into_first_result().await?;

        let foreign_keys: Vec<ForeignKey> = fk_rows
            .iter()
            .filter_map(|row| {
                let name = row.get::<&str, _>("constraint_name")?.to_string();
                let column = row.get::<&str, _>("column_name")?.to_string();
                let referenced_table = row.get::<&str, _>("referenced_table")?.to_string();
                let referenced_column = row.get::<&str, _>("referenced_column")?.to_string();
                let on_delete = row.get::<&str, _>("delete_rule").map(|s| s.to_string());
                let on_update = row.get::<&str, _>("update_rule").map(|s| s.to_string());

                Some(ForeignKey {
                    name,
                    column,
                    referenced_table,
                    referenced_column,
                    owner: None,
                    ref_object_type: Some("TABLE".to_string()),
                    on_delete,
                    on_update,
                })
            })
            .collect();

        Ok(TableSchema {
            table_name: table.to_string(),
            columns,
            indexes,
            foreign_keys,
        })
    }

    async fn get_views(&mut self, database: &str, schema: Option<&str>) -> Result<Vec<View>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected"))?
            .clone();
        MSSQLMetadataOps::get_views(&pool, database, schema).await
    }

    async fn get_indexes(&mut self, database: &str, schema: Option<&str>) -> Result<Vec<DbIndex>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected"))?
            .clone();
        MSSQLMetadataOps::get_indexes(&pool, database, schema).await
    }

    async fn get_procedures(
        &mut self,
        database: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Procedure>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected"))?
            .clone();
        MSSQLMetadataOps::get_procedures(&pool, database, schema).await
    }

    async fn get_triggers(&mut self, database: &str, schema: Option<&str>) -> Result<Vec<Trigger>> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected"))?
            .clone();
        MSSQLMetadataOps::get_triggers(&pool, database, schema).await
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
