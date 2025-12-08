use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, AsyncCommands, Client, RedisResult};
use std::collections::HashMap;
use std::time::Instant;

pub struct RedisConnection {
    client: Option<Client>,
    connection: Option<MultiplexedConnection>,
    current_db: u8,
}

impl RedisConnection {
    pub fn new() -> Self {
        Self {
            client: None,
            connection: None,
            current_db: 0,
        }
    }
}

impl Default for RedisConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseConnection for RedisConnection {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()> {
        let connection_string =
            if let (Some(username), Some(password)) = (&config.username, &config.password) {
                format!(
                    "redis://{}:{}@{}:{}",
                    username, password, config.host, config.port
                )
            } else if let Some(password) = &config.password {
                format!("redis://:{}@{}:{}", password, config.host, config.port)
            } else {
                format!("redis://{}:{}", config.host, config.port)
            };

        let client = Client::open(connection_string)?;
        let connection = client.get_multiplexed_async_connection().await?;

        self.client = Some(client);
        self.connection = Some(connection);

        if let Some(db_str) = &config.database {
            if let Ok(db_num) = db_str.parse::<u8>() {
                self.current_db = db_num;
                if let Some(conn) = &mut self.connection {
                    redis::cmd("SELECT")
                        .arg(db_num)
                        .query_async::<String>(conn)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connection = None;
        self.client = None;
        Ok(())
    }

    async fn test_connection(&mut self) -> Result<bool> {
        if let Some(conn) = &mut self.connection {
            let _: String = redis::cmd("PING").query_async(conn).await?;
            Ok(true)
        } else {
            Err(anyhow!("Not connected"))
        }
    }

    async fn execute_update(&mut self, query: &str) -> Result<u64> {
        let conn = self
            .connection
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;

        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty query"));
        }

        let command = parts[0].to_uppercase();
        let args = &parts[1..];

        // Execute the command using redis commands
        match command.as_str() {
            "SET" => {
                if args.len() < 2 {
                    return Err(anyhow!("SET requires key and value"));
                }
                let key = args[0];
                let value = args[1];
                let _: () = redis::cmd("SET")
                    .arg(key)
                    .arg(value)
                    .query_async(conn)
                    .await?;
                Ok(1)
            }
            "DEL" => {
                if args.is_empty() {
                    return Err(anyhow!("DEL requires at least one key"));
                }
                let count: u64 = redis::cmd("DEL").arg(args).query_async(conn).await?;
                Ok(count)
            }
            "HSET" => {
                if args.len() < 3 {
                    return Err(anyhow!("HSET requires hash, field, and value"));
                }
                let hash = args[0];
                let field = args[1];
                let value = args[2];
                let _: () = redis::cmd("HSET")
                    .arg(hash)
                    .arg(field)
                    .arg(value)
                    .query_async(conn)
                    .await?;
                Ok(1)
            }
            "HDEL" => {
                if args.len() < 2 {
                    return Err(anyhow!("HDEL requires hash and fields"));
                }
                let hash = args[0];
                let fields = &args[1..];
                let count: u64 = redis::cmd("HDEL")
                    .arg(hash)
                    .arg(fields)
                    .query_async(conn)
                    .await?;
                Ok(count)
            }
            _ => Err(anyhow!(
                "Unsupported command for execute_update: {}",
                command
            )),
        }
    }

    async fn execute_query(&mut self, query: &str) -> Result<QueryResult> {
        let conn = self
            .connection
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;
        let start = Instant::now();

        let parts: Vec<&str> = query.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty query"));
        }

        let command = parts[0].to_uppercase();
        let args = &parts[1..];

        let mut columns = vec!["key".to_string(), "value".to_string()];
        let mut rows = Vec::new();

        match command.as_str() {
            "GET" => {
                if args.is_empty() {
                    return Err(anyhow!("GET requires a key"));
                }
                let result: RedisResult<String> = conn.get(args[0]).await;
                if let Ok(value) = result {
                    let mut row = HashMap::new();
                    row.insert("key".to_string(), serde_json::json!(args[0]));
                    row.insert("value".to_string(), serde_json::json!(value));
                    rows.push(row);
                }
            }
            "SET" => {
                if args.len() < 2 {
                    return Err(anyhow!("SET requires key and value"));
                }
                let _: () = conn.set(args[0], args[1]).await?;
                let mut row = HashMap::new();
                row.insert("key".to_string(), serde_json::json!(args[0]));
                row.insert("value".to_string(), serde_json::json!("OK"));
                rows.push(row);
            }
            "KEYS" => {
                let pattern = if args.is_empty() { "*" } else { args[0] };
                let keys: Vec<String> = conn.keys(pattern).await?;
                for key in keys {
                    let mut row = HashMap::new();
                    row.insert("key".to_string(), serde_json::json!(key));
                    row.insert("value".to_string(), serde_json::json!(""));
                    rows.push(row);
                }
            }
            "SCAN" => {
                let cursor = if args.is_empty() {
                    0
                } else {
                    args[0].parse().unwrap_or(0)
                };
                let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("COUNT")
                    .arg(100)
                    .query_async(conn)
                    .await?;

                columns = vec!["cursor".to_string(), "key".to_string()];
                for key in keys {
                    let mut row = HashMap::new();
                    row.insert("cursor".to_string(), serde_json::json!(next_cursor));
                    row.insert("key".to_string(), serde_json::json!(key));
                    rows.push(row);
                }
            }
            _ => {
                let result: RedisResult<redis::Value> = redis::cmd(command.as_str())
                    .arg(args)
                    .query_async(conn)
                    .await;

                match result {
                    Ok(redis::Value::Okay) => {
                        let mut row = HashMap::new();
                        row.insert("key".to_string(), serde_json::json!("result"));
                        row.insert("value".to_string(), serde_json::json!("OK"));
                        rows.push(row);
                    }
                    Ok(redis::Value::Int(i)) => {
                        let mut row = HashMap::new();
                        row.insert("key".to_string(), serde_json::json!("result"));
                        row.insert("value".to_string(), serde_json::json!(i));
                        rows.push(row);
                    }
                    Err(e) => return Err(anyhow!("Redis error: {}", e)),
                    _ => {}
                }
            }
        }

        let execution_time = start.elapsed().as_millis();

        Ok(QueryResult {
            columns,
            column_display_names: None,
            column_types: None,
            rows,
            rows_affected: None,
            execution_time,
            final_query: Some(query.to_string()),
        })
    }

    async fn get_databases(&mut self) -> Result<Vec<Database>> {
        let conn = self
            .connection
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;

        let config: Vec<String> = redis::cmd("CONFIG")
            .arg("GET")
            .arg("databases")
            .query_async(conn)
            .await
            .unwrap_or_else(|_| vec!["databases".to_string(), "16".to_string()]);

        let max_dbs: u8 = if config.len() >= 2 {
            config[1].parse().unwrap_or(16)
        } else {
            16
        };

        let databases = (0..max_dbs)
            .map(|i| Database {
                name: format!("db{}", i),
            })
            .collect();

        Ok(databases)
    }

    async fn get_tables(&mut self, database: &str) -> Result<Vec<Table>> {
        let conn = self
            .connection
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;

        let db_num = database
            .strip_prefix("db")
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0);

        redis::cmd("SELECT")
            .arg(db_num)
            .query_async::<String>(conn)
            .await?;
        self.current_db = db_num;

        let info: String = redis::cmd("INFO").arg("keyspace").query_async(conn).await?;

        let mut tables = Vec::new();
        for line in info.lines() {
            if line.starts_with(&format!("db{}", db_num)) {
                if let Some(keys_part) = line.split(',').next() {
                    if let Some(count_str) = keys_part.split('=').nth(1) {
                        if let Ok(_count) = count_str.parse::<u64>() {
                            tables.push(Table {
                                name: format!("keys (db{})", db_num),
                                schema: None,
                                size_bytes: None,
                            });
                        }
                    }
                }
            }
        }

        if tables.is_empty() {
            tables.push(Table {
                name: format!("keys (db{})", db_num),
                schema: None,
                size_bytes: None,
            });
        }

        Ok(tables)
    }

    async fn get_table_schema(&mut self, _database: &str, _table: &str) -> Result<TableSchema> {
        Ok(TableSchema {
            table_name: "redis_keys".to_string(),
            columns: vec![
                Column {
                    name: "key".to_string(),
                    data_type: "string".to_string(),
                    nullable: false,
                    default_value: None,
                    is_primary_key: true,
                    is_auto_increment: false,
                },
                Column {
                    name: "type".to_string(),
                    data_type: "string".to_string(),
                    nullable: false,
                    default_value: None,
                    is_primary_key: false,
                    is_auto_increment: false,
                },
                Column {
                    name: "value".to_string(),
                    data_type: "any".to_string(),
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

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
