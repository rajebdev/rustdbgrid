use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ignite_rs::{Client, ClientConfig, Ignite};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

pub struct IgniteConnection {
    client: Option<Arc<Mutex<Client>>>,
    config: Option<ConnectionConfig>,
}

impl IgniteConnection {
    pub fn new() -> Self {
        Self {
            client: None,
            config: None,
        }
    }
}

impl Default for IgniteConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseConnection for IgniteConnection {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()> {
        let host = config.host.as_str();
        let port = config.port;

        let addr = format!("{}:{}", host, port);

        let mut client_config = ClientConfig::new(&addr);

        // Set credentials if provided
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            client_config.username = Some(username.clone());
            client_config.password = Some(password.clone());
        }

        let client =
            tokio::task::spawn_blocking(move || ignite_rs::new_client(client_config)).await??;

        self.client = Some(Arc::new(Mutex::new(client)));
        self.config = Some(config.clone());

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.client = None;
        self.config = None;
        Ok(())
    }

    async fn test_connection(&mut self) -> Result<bool> {
        if self.client.is_none() {
            return Ok(false);
        }

        // Try to get cache names as a connection test
        let client = self.client.as_ref().unwrap().clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut client = client.blocking_lock();
            client.get_cache_names()
        })
        .await?;

        Ok(result.is_ok())
    }

    async fn execute_query(&mut self, query: &str) -> Result<QueryResult> {
        let start = Instant::now();
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Ignite"))?
            .clone();

        let query_str = query.trim().to_string();
        
        // Check if it's a SHOW CACHES command or simple listing
        if query_str.to_uppercase().contains("SHOW") && query_str.to_uppercase().contains("CACHE") {
            // Handle SHOW CACHES command
            let result = tokio::task::spawn_blocking(move || {
                let mut client = client.blocking_lock();
                client.get_cache_names()
            })
            .await??;

            let columns = vec!["cache_name".to_string()];
            let rows: Vec<HashMap<String, serde_json::Value>> = result
                .iter()
                .map(|name| {
                    let mut row = HashMap::new();
                    row.insert(
                        "cache_name".to_string(),
                        serde_json::Value::String(name.clone()),
                    );
                    row
                })
                .collect();

            let execution_time = start.elapsed().as_millis();

            return Ok(QueryResult {
                columns,
                column_types: None,
                rows,
                rows_affected: None,
                execution_time,
                final_query: Some(query_str),
            });
        }

        // For actual SQL queries, try to execute them
        // Note: ignite-rs may have limited SQL support, so we'll try our best
        let query_for_exec = query_str.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut client = client.blocking_lock();
            
            // Try to parse the query to extract cache name
            // This is a simplified approach - in production you'd want proper SQL parsing
            let query_upper = query_for_exec.to_uppercase();
            
            if query_upper.contains("SELECT") {
                // Extract table/cache name from SELECT query
                // Try to get cache and execute query on it
                // Since ignite-rs has limited API, we'll return an informative message
                Err(anyhow!(
                    "SQL query execution requires proper cache schema. Please ensure the cache has a SQL schema defined. Query: {}",
                    query_for_exec
                ))
            } else if query_upper.contains("INSERT") || query_upper.contains("UPDATE") || query_upper.contains("DELETE") {
                Err(anyhow!(
                    "DML operations (INSERT/UPDATE/DELETE) are not yet supported through this interface. Query: {}",
                    query_for_exec
                ))
            } else {
                // For other queries, try to list caches as fallback
                client.get_cache_names().map_err(|e| anyhow!("Query execution failed: {}", e))
            }
        })
        .await?;

        match result {
            Ok(cache_names) => {
                // Return cache list as fallback
                let columns = vec!["cache_name".to_string(), "note".to_string()];
                let rows: Vec<HashMap<String, serde_json::Value>> = cache_names
                    .iter()
                    .map(|name| {
                        let mut row = HashMap::new();
                        row.insert(
                            "cache_name".to_string(),
                            serde_json::Value::String(name.clone()),
                        );
                        row.insert(
                            "note".to_string(),
                            serde_json::Value::String("Available cache".to_string()),
                        );
                        row
                    })
                    .collect();

                let execution_time = start.elapsed().as_millis();

                Ok(QueryResult {
                    columns,
                    column_types: None,
                    rows,
                    rows_affected: None,
                    execution_time,
                    final_query: Some(query_str),
                })
            }
            Err(e) => {
                // Return error as a result row
                let columns = vec!["error".to_string()];
                let mut row = HashMap::new();
                row.insert(
                    "error".to_string(),
                    serde_json::Value::String(e.to_string()),
                );

                let execution_time = start.elapsed().as_millis();

                Ok(QueryResult {
                    columns,
                    column_types: None,
                    rows: vec![row],
                    rows_affected: None,
                    execution_time,
                    final_query: Some(query_str),
                })
            }
        }
    }

    async fn get_databases(&mut self) -> Result<Vec<Database>> {
        // In Ignite, caches are similar to databases
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Ignite"))?
            .clone();

        let cache_names = tokio::task::spawn_blocking(move || {
            let mut client = client.blocking_lock();
            client.get_cache_names()
        })
        .await??;

        let databases = cache_names
            .into_iter()
            .map(|name| Database { name })
            .collect();

        Ok(databases)
    }

    async fn get_tables(&mut self, database: &str) -> Result<Vec<Table>> {
        // In Ignite, we need to query system tables to get table information
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Ignite"))?
            .clone();

        let cache_name = database.to_string();

        // This is a simplified implementation
        // Real implementation would query system tables for schema information
        let tables = tokio::task::spawn_blocking(move || {
            let mut client = client.blocking_lock();

            // Try to get cache configuration
            match client.get_cache_names() {
                Ok(names) => {
                    if names.contains(&cache_name) {
                        Ok(vec![Table {
                            name: cache_name.clone(),
                            schema: None,
                            size_bytes: None,
                        }])
                    } else {
                        Ok(vec![])
                    }
                }
                Err(e) => Err(e),
            }
        })
        .await??;

        Ok(tables)
    }

    async fn get_table_schema(&mut self, _database: &str, table: &str) -> Result<TableSchema> {
        // Return a basic schema structure
        // Real implementation would query Ignite metadata
        Ok(TableSchema {
            table_name: table.to_string(),
            columns: vec![
                Column {
                    name: "key".to_string(),
                    data_type: "BINARY".to_string(),
                    nullable: false,
                    default_value: None,
                    is_primary_key: true,
                    is_auto_increment: false,
                },
                Column {
                    name: "value".to_string(),
                    data_type: "BINARY".to_string(),
                    nullable: true,
                    default_value: None,
                    is_primary_key: false,
                    is_auto_increment: false,
                },
            ],
            indexes: vec![],
            foreign_keys: vec![],
        })
    }

    async fn get_table_data(
        &mut self,
        database: &str,
        table: &str,
        limit: u32,
        _offset: u32,
    ) -> Result<QueryResult> {
        let start = Instant::now();
        
        // Use SQL query to get data from the cache
        // In Ignite, caches can be queried using SQL
        let cache_name = if database.is_empty() { table } else { database };
        let query = format!("SELECT * FROM {} LIMIT {}", cache_name, limit);
        
        // Try to execute the query
        match self.execute_query(&query).await {
            Ok(mut result) => {
                result.execution_time = start.elapsed().as_millis();
                result.final_query = Some(query);
                Ok(result)
            }
            Err(e) => {
                // If SQL query fails, return a helpful message
                let columns = vec!["message".to_string()];
                let mut rows: Vec<HashMap<String, serde_json::Value>> = vec![];
                
                let mut info_row = HashMap::new();
                info_row.insert(
                    "message".to_string(),
                    serde_json::Value::String(format!(
                        "Cannot display cache data directly. Cache '{}' exists but may not have a SQL table schema defined. Try executing SQL queries manually. Error: {}",
                        cache_name, e
                    )),
                );
                rows.push(info_row);

                let execution_time = start.elapsed().as_millis();

                Ok(QueryResult {
                    columns,
                    column_types: None,
                    rows,
                    rows_affected: None,
                    execution_time,
                    final_query: Some(query),
                })
            }
        }
    }
}
