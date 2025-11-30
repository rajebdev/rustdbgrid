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

        // Select database if specified
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

    async fn execute_query(&mut self, query: &str) -> Result<QueryResult> {
        let conn = self
            .connection
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;
        let start = Instant::now();

        // Parse Redis command from query string
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
            "DEL" => {
                if args.is_empty() {
                    return Err(anyhow!("DEL requires at least one key"));
                }
                let deleted: i32 = conn.del(args).await?;
                let mut row = HashMap::new();
                row.insert("key".to_string(), serde_json::json!("deleted_count"));
                row.insert("value".to_string(), serde_json::json!(deleted));
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
            "HGETALL" => {
                if args.is_empty() {
                    return Err(anyhow!("HGETALL requires a key"));
                }
                let result: HashMap<String, String> = conn.hgetall(args[0]).await?;
                columns = vec!["field".to_string(), "value".to_string()];
                for (field, value) in result {
                    let mut row = HashMap::new();
                    row.insert("field".to_string(), serde_json::json!(field));
                    row.insert("value".to_string(), serde_json::json!(value));
                    rows.push(row);
                }
            }
            "HGET" => {
                if args.len() < 2 {
                    return Err(anyhow!("HGET requires key and field"));
                }
                let result: RedisResult<String> = conn.hget(args[0], args[1]).await;
                if let Ok(value) = result {
                    columns = vec!["field".to_string(), "value".to_string()];
                    let mut row = HashMap::new();
                    row.insert("field".to_string(), serde_json::json!(args[1]));
                    row.insert("value".to_string(), serde_json::json!(value));
                    rows.push(row);
                }
            }
            "LRANGE" => {
                if args.len() < 3 {
                    return Err(anyhow!("LRANGE requires key, start, and stop"));
                }
                let start: isize = args[1].parse()?;
                let stop: isize = args[2].parse()?;
                let result: Vec<String> = conn.lrange(args[0], start, stop).await?;
                columns = vec!["index".to_string(), "value".to_string()];
                for (idx, value) in result.iter().enumerate() {
                    let mut row = HashMap::new();
                    row.insert("index".to_string(), serde_json::json!(idx));
                    row.insert("value".to_string(), serde_json::json!(value));
                    rows.push(row);
                }
            }
            "SMEMBERS" => {
                if args.is_empty() {
                    return Err(anyhow!("SMEMBERS requires a key"));
                }
                let result: Vec<String> = conn.smembers(args[0]).await?;
                columns = vec!["member".to_string()];
                for member in result {
                    let mut row = HashMap::new();
                    row.insert("member".to_string(), serde_json::json!(member));
                    rows.push(row);
                }
            }
            "ZRANGE" => {
                if args.len() < 3 {
                    return Err(anyhow!("ZRANGE requires key, start, and stop"));
                }
                let start: isize = args[1].parse()?;
                let stop: isize = args[2].parse()?;
                let result: Vec<String> = conn.zrange(args[0], start, stop).await?;
                columns = vec!["member".to_string(), "score".to_string()];
                for member in result {
                    let mut row = HashMap::new();
                    row.insert("member".to_string(), serde_json::json!(member));
                    row.insert("score".to_string(), serde_json::json!(""));
                    rows.push(row);
                }
            }
            "INFO" => {
                let section = if args.is_empty() { "default" } else { args[0] };
                let info: String = redis::cmd("INFO").arg(section).query_async(conn).await?;
                columns = vec!["property".to_string(), "value".to_string()];
                for line in info.lines() {
                    if line.contains(':') {
                        let parts: Vec<&str> = line.splitn(2, ':').collect();
                        if parts.len() == 2 {
                            let mut row = HashMap::new();
                            row.insert("property".to_string(), serde_json::json!(parts[0]));
                            row.insert("value".to_string(), serde_json::json!(parts[1]));
                            rows.push(row);
                        }
                    }
                }
            }
            _ => {
                // Generic command execution
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
                    Ok(redis::Value::BulkString(bytes)) => {
                        let value = String::from_utf8_lossy(&bytes);
                        let mut row = HashMap::new();
                        row.insert("key".to_string(), serde_json::json!("result"));
                        row.insert("value".to_string(), serde_json::json!(value));
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

        // Get CONFIG GET databases to find max databases
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

        // Parse database number from "dbN" format
        let db_num = database
            .strip_prefix("db")
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0);

        // Select the database
        redis::cmd("SELECT")
            .arg(db_num)
            .query_async::<String>(conn)
            .await?;
        self.current_db = db_num;

        // Get info about keyspace
        let info: String = redis::cmd("INFO").arg("keyspace").query_async(conn).await?;

        // Parse keyspace info
        let mut tables = Vec::new();
        for line in info.lines() {
            if line.starts_with(&format!("db{}", db_num)) {
                // Parse keys count from the line
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

        // If no info available, create a generic entry
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
        // Redis is schemaless, return a generic schema
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

    async fn get_table_data(
        &mut self,
        database: &str,
        _table: &str,
        limit: u32,
        offset: u32,
    ) -> Result<QueryResult> {
        let conn = self
            .connection
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;

        // Parse and select database
        let db_num = database
            .strip_prefix("db")
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0);

        redis::cmd("SELECT")
            .arg(db_num)
            .query_async::<String>(conn)
            .await?;

        let start = Instant::now();

        // Use SCAN to get keys with pagination
        let cursor = offset;
        let (_, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("COUNT")
            .arg(limit)
            .query_async(conn)
            .await?;

        let mut rows = Vec::new();
        let columns = vec!["key".to_string(), "type".to_string(), "value".to_string()];

        for key in keys {
            let key_type: String = redis::cmd("TYPE")
                .arg(&key)
                .query_async(conn)
                .await
                .unwrap_or_else(|_| "unknown".to_string());

            let value = match key_type.as_str() {
                "string" => {
                    let val: RedisResult<String> = conn.get(&key).await;
                    val.unwrap_or_else(|_| "".to_string())
                }
                "list" => {
                    let len: i32 = conn.llen(&key).await.unwrap_or(0);
                    format!("[list: {} items]", len)
                }
                "set" => {
                    let len: i32 = conn.scard(&key).await.unwrap_or(0);
                    format!("[set: {} members]", len)
                }
                "zset" => {
                    let len: i32 = conn.zcard(&key).await.unwrap_or(0);
                    format!("[sorted set: {} members]", len)
                }
                "hash" => {
                    let len: i32 = conn.hlen(&key).await.unwrap_or(0);
                    format!("[hash: {} fields]", len)
                }
                _ => "[unknown type]".to_string(),
            };

            let mut row = HashMap::new();
            row.insert("key".to_string(), serde_json::json!(key));
            row.insert("type".to_string(), serde_json::json!(key_type));
            row.insert("value".to_string(), serde_json::json!(value));
            rows.push(row);
        }

        let execution_time = start.elapsed().as_millis();

        Ok(QueryResult {
            columns,
            column_types: None,
            rows,
            rows_affected: None,
            execution_time,
            final_query: None,
        })
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
