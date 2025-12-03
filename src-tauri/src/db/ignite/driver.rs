use crate::db::traits::DatabaseConnection;
use crate::models::{connection::*, query_result::*, schema::*};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(windows)]
use tokio::net::windows::named_pipe::ClientOptions;
#[cfg(unix)]
use tokio::net::UnixStream;

// ============ IPC STRUCTURES ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub action: String,
    pub connection_id: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub query: Option<String>,
    pub cache_name: Option<String>,
    pub table_name: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub success: bool,
    pub message: Option<String>,
    pub result: Option<BasicResult>,
    pub caches: Option<Vec<CacheInfo>>,
    pub tables: Option<Vec<TableInfo>>,
    pub schema: Option<SchemaInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicResult {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub rows_affected: Option<i64>,
    pub final_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: Option<bool>,
    pub default_value: Option<serde_json::Value>,
    pub is_primary_key: Option<bool>,
}

// ============ STATIC STATE ============

lazy_static::lazy_static! {
    static ref PIPE_NAME: String = {
        let mode = if cfg!(debug_assertions) { "dev" } else { "prod" };
        format!("ignite-bridge-{}-{}", mode, std::process::id())
    };

    static ref SIDECAR_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    static ref SIDECAR_STARTED: AtomicBool = AtomicBool::new(false);
}

fn get_pipe_path() -> String {
    #[cfg(windows)]
    {
        format!("\\\\.\\pipe\\{}", *PIPE_NAME)
    }
    #[cfg(unix)]
    {
        format!("/tmp/{}.sock", *PIPE_NAME)
    }
}

pub fn shutdown_bridge() {
    if let Ok(mut guard) = SIDECAR_PROCESS.lock() {
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    SIDECAR_STARTED.store(false, Ordering::Relaxed);
    tracing::info!("🛑 [IGNITE BRIDGE] Bridge shutdown complete");
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

    fn ensure_sidecar_running() -> Result<()> {
        use std::sync::atomic::Ordering;

        if SIDECAR_STARTED.load(Ordering::Relaxed) {
            return Ok(());
        }

        tracing::info!("🚀 [IGNITE BRIDGE] Starting bridge sidecar...");

        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("Cannot get exe directory"))?;

        #[cfg(windows)]
        let sidecar_name = "ignite-x86_64-pc-windows-msvc.exe";
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let sidecar_name = "ignite-x86_64-unknown-linux-gnu";
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        let sidecar_name = "ignite-aarch64-unknown-linux-gnu";
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let sidecar_name = "ignite-aarch64-apple-darwin";
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        let sidecar_name = "ignite-x86_64-apple-darwin";
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        let sidecar_name = "ignite";

        let possible_sidecar_paths = [
            exe_dir.join(sidecar_name),
            exe_dir.join("resources").join(sidecar_name),
        ];

        let sidecar_path = possible_sidecar_paths.iter().find(|p| p.exists()).cloned();

        let (cmd, args): (std::path::PathBuf, Vec<String>) = if let Some(path) = sidecar_path {
            (path, vec![])
        } else {
            let possible_paths = [
                exe_dir.join("../../../src-bridge/ignite.cjs"),
                exe_dir.join("../../src-bridge/ignite.cjs"),
                exe_dir.join("../src-bridge/ignite.cjs"),
                exe_dir.join("src-bridge/ignite.cjs"),
                std::path::PathBuf::from("src-bridge/ignite.cjs"),
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

        let mut command = std::process::Command::new(&cmd);
        for arg in &args {
            command.arg(arg);
        }
        command.env("IGNITE_BRIDGE_PIPE", &*PIPE_NAME);
        command.stdout(std::process::Stdio::inherit());
        command.stderr(std::process::Stdio::inherit());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let child = command
            .spawn()
            .map_err(|e| anyhow!("Failed to start Ignite bridge: {}", e))?;

        if let Ok(mut guard) = SIDECAR_PROCESS.lock() {
            *guard = Some(child);
        }

        SIDECAR_STARTED.store(true, Ordering::Relaxed);
        tracing::info!(
            "✅ [IGNITE BRIDGE] Bridge sidecar started successfully with pipe: {}",
            *PIPE_NAME
        );
        Ok(())
    }

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

        let json = serde_json::to_vec(request)?;

        let len = json.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&json).await?;
        stream.flush().await?;

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let response_len = u32::from_be_bytes(len_buf) as usize;

        let mut response_buf = vec![0u8; response_len];
        stream.read_exact(&mut response_buf).await?;

        let response: IpcResponse = serde_json::from_slice(&response_buf)
            .map_err(|e| anyhow!("Invalid response from bridge: {}", e))?;

        Ok(response)
    }

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

    async fn ensure_bridge_running(&self) -> Result<()> {
        if self.is_bridge_running().await {
            return Ok(());
        }

        tracing::info!("🔄 [IGNITE BRIDGE] Bridge not responding, restarting...");
        SIDECAR_STARTED.store(false, std::sync::atomic::Ordering::Relaxed);

        Self::ensure_sidecar_running()?;

        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if self.is_bridge_running().await {
                tracing::info!("✅ [IGNITE BRIDGE] Bridge restarted successfully");
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
    }
}

#[async_trait]
impl DatabaseConnection for IgniteConnection {
    async fn connect(&mut self, config: &ConnectionConfig) -> Result<()> {
        tracing::info!(
            "🔌 [IGNITE] Connecting to Ignite cluster at {}:{}",
            config.host,
            config.port
        );

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
            let err_msg = result.message.unwrap_or("Connection failed".to_string());
            tracing::error!("❌ [IGNITE] Connection failed: {}", err_msg);
            return Err(anyhow!(err_msg));
        }

        self.config = Some(config.clone());
        self.connection_id = Some(connection_id.clone());

        tracing::info!(
            "✅ [IGNITE] Successfully connected to Ignite cluster (ID: {})",
            connection_id
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(connection_id) = &self.connection_id {
            tracing::info!(
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
                        tracing::info!(
                            "✅ [IGNITE] Bridge confirmed disconnect for connection: {}",
                            connection_id
                        );
                    } else {
                        tracing::warn!(
                            "⚠️ [IGNITE] Bridge disconnect returned failure for connection: {}",
                            connection_id
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [IGNITE] Failed to send disconnect to bridge for connection {}: {}",
                        connection_id,
                        e
                    );
                }
            }
        } else {
            tracing::info!("🔌 [IGNITE] No connection_id to disconnect");
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
        self.ensure_bridge_running().await?;

        let start = Instant::now();
        let connection_id = self
            .connection_id
            .as_ref()
            .ok_or_else(|| anyhow!("Not connected to Ignite"))?;

        let query_upper = query.trim().to_uppercase();

        if query_upper.starts_with("SCAN ") {
            let parts: Vec<&str> = query.split_whitespace().collect();
            let cache_name = parts.get(1).map(|s| s.to_string());

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

            tracing::info!(
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
                column_display_names: None,
                column_types: None,
                rows: br.rows,
                rows_affected: br.rows_affected.map(|v| v as u64),
                execution_time: start.elapsed().as_millis(),
                final_query: Some(query.to_string()),
            });
        }

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
            column_display_names: None,
            column_types: None,
            rows: br.rows,
            rows_affected: br.rows_affected.map(|v| v as u64),
            execution_time: start.elapsed().as_millis(),
            final_query: br.final_query.or_else(|| Some(query.to_string())),
        })
    }

    async fn get_databases(&mut self) -> Result<Vec<Database>> {
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

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
