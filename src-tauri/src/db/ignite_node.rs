// Ignite Bridge Connection
// Uses Bun-compiled sidecar with Named Pipe for secure IPC
//
// Security: Uses random pipe name per-session so other processes cannot connect

use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(windows)]
use tokio::net::windows::named_pipe::ClientOptions;
#[cfg(unix)]
use tokio::net::UnixStream;

// Generate random pipe name once per process - provides security without overhead
// Use a fixed base name for development to avoid orphan processes on reload
static PIPE_NAME: Lazy<String> = Lazy::new(|| {
    // In development, use a predictable name so HMR/reload can reconnect
    // In production, add PID for security (multiple instances)
    #[cfg(debug_assertions)]
    {
        // Development: fixed name allows reconnection after reload
        "rustdbgrid-ignite-dev".to_string()
    }
    #[cfg(not(debug_assertions))]
    {
        // Production: unique per process for security
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid = std::process::id();
        format!("rustdbgrid-ignite-{}-{}", pid, timestamp % 1_000_000)
    }
});

// Sidecar process handle
static SIDECAR_STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static SIDECAR_PROCESS: Lazy<Mutex<Option<std::process::Child>>> = Lazy::new(|| Mutex::new(None));

#[cfg(windows)]
fn get_pipe_path() -> String {
    format!(r"\\.\pipe\{}", *PIPE_NAME)
}

#[cfg(unix)]
fn get_pipe_path() -> String {
    format!("/tmp/{}.sock", *PIPE_NAME)
}

#[derive(Debug, Serialize)]
struct IpcRequest {
    action: String,
    #[serde(rename = "connectionId", skip_serializing_if = "Option::is_none")]
    connection_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
    #[serde(rename = "cacheName", skip_serializing_if = "Option::is_none")]
    cache_name: Option<String>,
    #[serde(rename = "tableName", skip_serializing_if = "Option::is_none")]
    table_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct IpcResponse {
    success: bool,
    message: Option<String>,
    // Flattened fields from various responses
    caches: Option<Vec<CacheInfo>>,
    tables: Option<Vec<TableInfo>>,
    result: Option<BridgeQueryResult>,
    schema: Option<BridgeSchema>,
    #[allow(dead_code)]
    connections: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct CacheInfo {
    name: String,
}

#[derive(Debug, Deserialize)]
struct TableInfo {
    name: String,
}

#[derive(Debug, Deserialize)]
struct BridgeQueryResult {
    columns: Vec<String>,
    rows: Vec<HashMap<String, serde_json::Value>>,
    #[serde(rename = "rowsAffected")]
    rows_affected: Option<u64>,
    #[serde(rename = "finalQuery")]
    final_query: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BridgeSchema {
    #[serde(rename = "tableName")]
    table_name: String,
    columns: Vec<BridgeColumn>,
}

#[derive(Debug, Deserialize)]
struct BridgeColumn {
    name: String,
    #[serde(rename = "dataType")]
    data_type: String,
    #[serde(rename = "isNullable")]
    is_nullable: Option<bool>,
    #[serde(rename = "defaultValue")]
    default_value: Option<serde_json::Value>,
    #[serde(rename = "isPrimaryKey")]
    is_primary_key: Option<bool>,
}

/// Shutdown the Ignite bridge sidecar process
/// Call this when the application is closing
pub fn shutdown_bridge() {
    use std::sync::atomic::Ordering;

    log::info!("🛑 [IGNITE BRIDGE] Shutting down bridge...");

    // Try to send shutdown command via IPC first (graceful shutdown)
    if SIDECAR_STARTED.load(Ordering::Relaxed) {
        // Try graceful shutdown via IPC
        let pipe_path = get_pipe_path();

        #[cfg(windows)]
        {
            use std::io::Write;
            if let Ok(mut pipe) = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&pipe_path)
            {
                let request = r#"{"action":"shutdown"}"#;
                let len = request.len() as u32;
                let _ = pipe.write_all(&len.to_be_bytes());
                let _ = pipe.write_all(request.as_bytes());
                let _ = pipe.flush();
                log::info!("🛑 [IGNITE BRIDGE] Sent shutdown command via IPC");
            }
        }

        #[cfg(unix)]
        {
            use std::io::Write;
            if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(&pipe_path) {
                let request = r#"{"action":"shutdown"}"#;
                let len = request.len() as u32;
                let _ = stream.write_all(&len.to_be_bytes());
                let _ = stream.write_all(request.as_bytes());
                let _ = stream.flush();
                log::info!("🛑 [IGNITE BRIDGE] Sent shutdown command via IPC");
            }
        }
    }

    // Kill the process if still running
    if let Ok(mut guard) = SIDECAR_PROCESS.lock() {
        if let Some(ref mut child) = *guard {
            // Give it a moment for graceful shutdown
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Check if still running and kill if necessary
            match child.try_wait() {
                Ok(Some(_)) => {
                    log::info!("🛑 [IGNITE BRIDGE] Process exited gracefully");
                }
                Ok(_) => {
                    // Still running, force kill
                    if let Err(e) = child.kill() {
                        log::warn!("🛑 [IGNITE BRIDGE] Failed to kill process: {}", e);
                    } else {
                        log::info!("🛑 [IGNITE BRIDGE] Process killed");
                    }
                }
                Err(e) => {
                    log::warn!("🛑 [IGNITE BRIDGE] Error checking process status: {}", e);
                }
            }
        }
        *guard = None;
    }

    SIDECAR_STARTED.store(false, Ordering::Relaxed);
    log::info!("🛑 [IGNITE BRIDGE] Bridge shutdown complete");
}

pub struct IgniteConnection {
    config: Option<ConnectionConfig>,
    connection_id: Option<String>,
}

impl IgniteConnection {
    pub fn new() -> Self {
        Self {
            config: None,
            connection_id: None,
        }
    }

    /// Start the sidecar if not already running
    fn ensure_sidecar_running() -> Result<()> {
        use std::sync::atomic::Ordering;

        if SIDECAR_STARTED.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Get path to sidecar binary
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("Cannot get exe directory"))?;

        #[cfg(windows)]
        let sidecar_name = "ignite-x86_64-pc-windows-msvc.exe";
        #[cfg(target_os = "linux")]
        let sidecar_name = "ignite-x86_64-unknown-linux-gnu";
        #[cfg(target_os = "macos")]
        let sidecar_name = "ignite-aarch64-apple-darwin";

        let sidecar_path = exe_dir.join(sidecar_name);

        // Try sidecar first (production), fallback to node (dev mode)
        let (cmd, args): (std::path::PathBuf, Vec<String>) = if sidecar_path.exists() {
            (sidecar_path, vec![])
        } else {
            // Dev mode: try multiple possible locations for the bridge script
            let possible_paths = [
                // From exe directory (dev mode - exe is in target/debug)
                exe_dir.join("../../../src-bridge/ignite.cjs"),
                exe_dir.join("../../src-bridge/ignite.cjs"),
                exe_dir.join("../src-bridge/ignite.cjs"),
                exe_dir.join("src-bridge/ignite.cjs"),
                // From current working directory
                std::path::PathBuf::from("src-bridge/ignite.cjs"),
                // Absolute fallback for development
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../src-bridge/ignite.cjs"),
            ];

            let bridge_script = possible_paths
                .iter()
                .find(|p| p.exists())
                .cloned()
                .ok_or_else(|| {
                    anyhow!(
                        "Ignite bridge not found. Searched paths: {:?}",
                        possible_paths
                    )
                })?;

            (
                std::path::PathBuf::from("node"),
                vec![bridge_script.to_string_lossy().to_string()],
            )
        };

        // Start process with random pipe name
        let mut command = std::process::Command::new(&cmd);
        for arg in &args {
            command.arg(arg);
        }
        let child = command
            .env("IGNITE_BRIDGE_PIPE", &*PIPE_NAME)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|e| anyhow!("Failed to start Ignite bridge: {}", e))?;

        // Store process handle for cleanup
        if let Ok(mut guard) = SIDECAR_PROCESS.lock() {
            *guard = Some(child);
        }

        SIDECAR_STARTED.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Send request to bridge via Named Pipe and get response
    async fn send_request(&self, request: &IpcRequest) -> Result<IpcResponse> {
        let pipe_path = get_pipe_path();

        #[cfg(windows)]
        let mut stream = {
            ClientOptions::new()
                .open(&pipe_path)
                .map_err(|e| anyhow!("Failed to connect to bridge pipe: {}", e))?
        };

        #[cfg(unix)]
        let mut stream = UnixStream::connect(&pipe_path)
            .await
            .map_err(|e| anyhow!("Failed to connect to bridge socket: {}", e))?;

        // Serialize request
        let json = serde_json::to_vec(request)?;

        // Write length prefix (4 bytes, big-endian)
        let len = json.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&json).await?;
        stream.flush().await?;

        // Read response length
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let response_len = u32::from_be_bytes(len_buf) as usize;

        // Read response body
        let mut response_buf = vec![0u8; response_len];
        stream.read_exact(&mut response_buf).await?;

        // Parse response
        let response: IpcResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| anyhow!("Invalid response from bridge: {}", e))?;

        Ok(response)
    }

    /// Check if bridge is running
    async fn is_bridge_running(&self) -> bool {
        let request = IpcRequest {
            action: "health".to_string(),
            connection_id: None,
            host: None,
            port: None,
            username: None,
            password: None,
            query: None,
            cache_name: None,
            table_name: None,
            limit: None,
            offset: None,
        };

        match self.send_request(&request).await {
            Ok(resp) => resp.success,
            Err(_) => false,
        }
    }

    /// Start the bridge sidecar if not running
    async fn ensure_bridge_running(&self) -> Result<()> {
        if self.is_bridge_running().await {
            return Ok(());
        }

        // Bridge not running - reset the flag so ensure_sidecar_running will start it
        log::info!("🔄 [IGNITE BRIDGE] Bridge not responding, restarting...");
        SIDECAR_STARTED.store(false, std::sync::atomic::Ordering::Relaxed);

        // Start sidecar
        Self::ensure_sidecar_running()?;

        // Wait for bridge to start
        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if self.is_bridge_running().await {
                log::info!("✅ [IGNITE BRIDGE] Bridge restarted successfully");
                return Ok(());
            }
        }

        Err(anyhow!(
            "Ignite bridge sidecar failed to start within timeout"
        ))
    }
}

impl Default for IgniteConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for IgniteConnection {
    fn drop(&mut self) {
        // Note: We don't kill the bridge process as it might be shared
        // The bridge will clean up connections on disconnect
    }
}

#[async_trait]
impl DatabaseConnection for IgniteConnection {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()> {
        // Ensure bridge is running
        self.ensure_bridge_running().await?;

        let connection_id = config.id.clone();
        let request = IpcRequest {
            action: "connect".to_string(),
            connection_id: Some(connection_id.clone()),
            host: Some(config.host.clone()),
            port: Some(config.port),
            username: config.username.clone(),
            password: config.password.clone(),
            query: None,
            cache_name: None,
            table_name: None,
            limit: None,
            offset: None,
        };

        let result = self.send_request(&request).await?;

        if !result.success {
            return Err(anyhow!(result
                .message
                .unwrap_or("Connection failed".to_string())));
        }

        self.config = Some(config.clone());
        self.connection_id = Some(connection_id);

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(connection_id) = &self.connection_id {
            log::info!(
                "🔌 [IGNITE] Sending disconnect request to bridge for connection: {}",
                connection_id
            );

            let request = IpcRequest {
                action: "disconnect".to_string(),
                connection_id: Some(connection_id.clone()),
                host: None,
                port: None,
                username: None,
                password: None,
                query: None,
                cache_name: None,
                table_name: None,
                limit: None,
                offset: None,
            };

            match self.send_request(&request).await {
                Ok(response) => {
                    if response.success {
                        log::info!(
                            "✅ [IGNITE] Bridge confirmed disconnect for connection: {}",
                            connection_id
                        );
                    } else {
                        log::warn!(
                            "⚠️ [IGNITE] Bridge disconnect returned failure for connection: {}",
                            connection_id
                        );
                    }
                }
                Err(e) => {
                    log::warn!(
                        "⚠️ [IGNITE] Failed to send disconnect to bridge for connection {}: {}",
                        connection_id,
                        e
                    );
                }
            }
        } else {
            log::info!("🔌 [IGNITE] No connection_id to disconnect");
        }

        self.config = None;
        self.connection_id = None;

        Ok(())
    }

    async fn test_connection(&mut self) -> Result<bool> {
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| anyhow!("Not configured"))?;

        // Ensure bridge is running
        self.ensure_bridge_running().await?;

        let request = IpcRequest {
            action: "test".to_string(),
            connection_id: None,
            host: Some(config.host.clone()),
            port: Some(config.port),
            username: config.username.clone(),
            password: config.password.clone(),
            query: None,
            cache_name: None,
            table_name: None,
            limit: None,
            offset: None,
        };

        let result = self.send_request(&request).await?;
        Ok(result.success)
    }

    async fn execute_query(&mut self, query: &str) -> Result<QueryResult> {
        // Ensure bridge is running (may have auto-shutdown)
        self.ensure_bridge_running().await?;

        let start = Instant::now();
        let connection_id = self
            .connection_id
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Ignite"))?;

        let query_upper = query.trim().to_uppercase();

        // For SCAN queries, use scan action instead of query
        if query_upper.starts_with("SCAN ") {
            // Parse: SCAN cache_name [LIMIT x] [OFFSET y]
            let parts: Vec<&str> = query.split_whitespace().collect();
            let cache_name = parts.get(1).map(|s| s.to_string());

            // Parse LIMIT and OFFSET
            let mut limit = 200u32;
            let mut offset = 0u32;

            for i in 0..parts.len() {
                if parts[i].to_uppercase() == "LIMIT" {
                    if let Some(val) = parts.get(i + 1) {
                        limit = val.parse().unwrap_or(200);
                    }
                }
                if parts[i].to_uppercase() == "OFFSET" {
                    if let Some(val) = parts.get(i + 1) {
                        offset = val.parse().unwrap_or(0);
                    }
                }
            }

            log::info!(
                "🔥 [IGNITE] Executing SCAN: cache={:?}, limit={}, offset={}",
                cache_name,
                limit,
                offset
            );

            let request = IpcRequest {
                action: "scan".to_string(),
                connection_id: Some(connection_id.clone()),
                host: None,
                port: None,
                username: None,
                password: None,
                query: None,
                cache_name: cache_name.clone(),
                table_name: None,
                limit: Some(limit),
                offset: Some(offset),
            };

            let result = self.send_request(&request).await?;

            if !result.success {
                return Err(anyhow!(result.message.unwrap_or("Scan failed".to_string())));
            }

            let br = result.result.ok_or_else(|| anyhow!("No scan data"))?;

            return Ok(QueryResult {
                columns: br.columns,
                column_types: None,
                rows: br.rows,
                rows_affected: br.rows_affected,
                execution_time: start.elapsed().as_millis(),
                final_query: Some(query.to_string()),
            });
        }

        // For SQL queries, use query action
        let request = IpcRequest {
            action: "query".to_string(),
            connection_id: Some(connection_id.clone()),
            host: None,
            port: None,
            username: None,
            password: None,
            query: Some(query.to_string()),
            cache_name: None,
            table_name: None,
            limit: None,
            offset: None,
        };

        let result = self.send_request(&request).await?;

        if !result.success {
            return Err(anyhow!(result
                .message
                .unwrap_or("Query failed".to_string())));
        }

        let br = result.result.ok_or_else(|| anyhow!("No result data"))?;

        Ok(QueryResult {
            columns: br.columns,
            column_types: None,
            rows: br.rows,
            rows_affected: br.rows_affected,
            execution_time: start.elapsed().as_millis(),
            final_query: br.final_query.or_else(|| Some(query.to_string())),
        })
    }

    async fn get_databases(&mut self) -> Result<Vec<Database>> {
        // Ensure bridge is running (may have auto-shutdown)
        self.ensure_bridge_running().await?;

        let connection_id = self
            .connection_id
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Ignite"))?;

        let request = IpcRequest {
            action: "caches".to_string(),
            connection_id: Some(connection_id.clone()),
            host: None,
            port: None,
            username: None,
            password: None,
            query: None,
            cache_name: None,
            table_name: None,
            limit: None,
            offset: None,
        };

        let result = self.send_request(&request).await?;

        if !result.success {
            return Err(anyhow!(result
                .message
                .unwrap_or("Failed to get caches".to_string())));
        }

        let caches = result.caches.ok_or_else(|| anyhow!("No cache data"))?;

        Ok(caches
            .into_iter()
            .map(|c| Database { name: c.name })
            .collect())
    }

    async fn get_tables(&mut self, database: &str) -> Result<Vec<Table>> {
        // Ensure bridge is running (may have auto-shutdown)
        self.ensure_bridge_running().await?;

        let connection_id = self
            .connection_id
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Ignite"))?;

        let request = IpcRequest {
            action: "tables".to_string(),
            connection_id: Some(connection_id.clone()),
            host: None,
            port: None,
            username: None,
            password: None,
            query: None,
            cache_name: Some(database.to_string()),
            table_name: None,
            limit: None,
            offset: None,
        };

        let result = self.send_request(&request).await?;

        if !result.success {
            return Err(anyhow!(result
                .message
                .unwrap_or("Failed to get tables".to_string())));
        }

        let tables = result.tables.ok_or_else(|| anyhow!("No table data"))?;

        Ok(tables
            .into_iter()
            .map(|t| Table {
                name: t.name,
                schema: None,
                size_bytes: None,
            })
            .collect())
    }

    async fn get_table_schema(&mut self, database: &str, table: &str) -> Result<TableSchema> {
        // Ensure bridge is running (may have auto-shutdown)
        self.ensure_bridge_running().await?;

        let connection_id = self
            .connection_id
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Ignite"))?;

        let request = IpcRequest {
            action: "schema".to_string(),
            connection_id: Some(connection_id.clone()),
            host: None,
            port: None,
            username: None,
            password: None,
            query: None,
            cache_name: Some(database.to_string()),
            table_name: Some(table.to_string()),
            limit: None,
            offset: None,
        };

        let result = self.send_request(&request).await?;

        if !result.success {
            return Err(anyhow!(result
                .message
                .unwrap_or("Failed to get schema".to_string())));
        }

        let schema = result.schema.ok_or_else(|| anyhow!("No schema data"))?;

        Ok(TableSchema {
            table_name: schema.table_name,
            columns: schema
                .columns
                .into_iter()
                .map(|c| Column {
                    name: c.name,
                    data_type: c.data_type,
                    nullable: c.is_nullable.unwrap_or(true),
                    default_value: c.default_value.map(|v| v.to_string()),
                    is_primary_key: c.is_primary_key.unwrap_or(false),
                    is_auto_increment: false,
                })
                .collect(),
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
        // Ensure bridge is running (may have auto-shutdown)
        self.ensure_bridge_running().await?;

        let start = Instant::now();
        let connection_id = self
            .connection_id
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Ignite"))?;

        let cache_name = if database.is_empty() { table } else { database };

        let request = IpcRequest {
            action: "scan".to_string(),
            connection_id: Some(connection_id.clone()),
            host: None,
            port: None,
            username: None,
            password: None,
            query: None,
            cache_name: Some(cache_name.to_string()),
            table_name: None,
            limit: Some(limit),
            offset: Some(offset),
        };

        let result = self.send_request(&request).await?;

        if !result.success {
            return Err(anyhow!(result.message.unwrap_or("Scan failed".to_string())));
        }

        let br = result.result.ok_or_else(|| anyhow!("No scan data"))?;

        Ok(QueryResult {
            columns: br.columns,
            column_types: None,
            rows: br.rows,
            rows_affected: br.rows_affected,
            execution_time: start.elapsed().as_millis(),
            final_query: Some(format!(
                "SCAN {} LIMIT {} OFFSET {}",
                cache_name, limit, offset
            )),
        })
    }
}
