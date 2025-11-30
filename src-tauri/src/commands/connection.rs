use crate::models::connection::*;
use crate::utils::{connection_pool::ConnectionPool, storage};
use std::sync::Mutex;
use tauri::State;

pub struct ConnectionStore {
    pub connections: Mutex<Vec<ConnectionConfig>>,
    pub pool: ConnectionPool,
}

impl ConnectionStore {
    pub fn new() -> Self {
        // Load connections from file on initialization
        let connections = storage::load_connections().unwrap_or_else(|e| {
            tracing::error!("Failed to load connections: {}", e);
            Vec::new()
        });

        ConnectionStore {
            connections: Mutex::new(connections),
            pool: ConnectionPool::new(),
        }
    }

    /// Save current connections to file
    fn save_to_file(&self) -> Result<(), String> {
        let connections = self.connections.lock().unwrap();
        storage::save_connections(&connections)
            .map_err(|e| format!("Failed to save connections: {}", e))
    }
}

impl Default for ConnectionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub async fn test_connection(config: ConnectionConfig) -> Result<ConnectionStatus, String> {
    tracing::info!(
        "🔌 [COMMAND] Testing connection to '{}' ({:?})",
        config.name,
        config.db_type
    );

    let mut conn = crate::db::traits::create_connection(&config.db_type);

    match conn.connect(&config).await {
        Ok(_) => match conn.test_connection().await {
            Ok(true) => {
                tracing::info!(
                    "✅ [COMMAND] Connection test successful for '{}'",
                    config.name
                );
                Ok(ConnectionStatus {
                    success: true,
                    message: "Connection successful".to_string(),
                })
            }
            Ok(false) => {
                tracing::warn!("⚠️ [COMMAND] Connection test failed for '{}'", config.name);
                Ok(ConnectionStatus {
                    success: false,
                    message: "Connection failed".to_string(),
                })
            }
            Err(e) => {
                tracing::error!(
                    "❌ [COMMAND] Connection test error for '{}': {}",
                    config.name,
                    e
                );
                Ok(ConnectionStatus {
                    success: false,
                    message: format!("Connection test failed: {}", e),
                })
            }
        },
        Err(e) => {
            tracing::error!("❌ [COMMAND] Failed to connect to '{}': {}", config.name, e);
            Ok(ConnectionStatus {
                success: false,
                message: format!("Connection failed: {}", e),
            })
        }
    }
}

#[tauri::command]
pub async fn save_connection(
    config: ConnectionConfig,
    state: State<'_, ConnectionStore>,
) -> Result<(), String> {
    tracing::info!(
        "💾 [COMMAND] Saving connection: '{}' (ID: {})",
        config.name,
        config.id
    );

    let mut connections = state.connections.lock().unwrap();

    // Remove existing connection with same ID if exists
    connections.retain(|c| c.id != config.id);
    connections.push(config.clone());

    // Release the lock before saving
    drop(connections);

    // Save to file
    state.save_to_file()?;

    tracing::info!(
        "✅ [COMMAND] Connection '{}' saved successfully",
        config.name
    );

    Ok(())
}

#[tauri::command]
pub async fn get_connections(
    state: State<'_, ConnectionStore>,
) -> Result<Vec<ConnectionConfig>, String> {
    let connections = state.connections.lock().unwrap();
    Ok(connections.clone())
}

#[tauri::command]
pub async fn delete_connection(
    id: String,
    state: State<'_, ConnectionStore>,
) -> Result<(), String> {
    tracing::info!("🗑️ [COMMAND] Deleting connection: {}", id);

    // Disconnect from pool if connected
    let _ = state.pool.disconnect(&id).await;

    let mut connections = state.connections.lock().unwrap();
    connections.retain(|c| c.id != id);

    // Release the lock before saving
    drop(connections);

    // Save to file
    state.save_to_file()?;

    tracing::info!("✅ [COMMAND] Connection {} deleted successfully", id);

    Ok(())
}

#[tauri::command]
pub async fn connect_to_database(
    config: ConnectionConfig,
    state: State<'_, ConnectionStore>,
) -> Result<(), String> {
    tracing::info!("🔌 [COMMAND] Connecting to database: '{}'", config.name);
    let result = state.pool.connect(config.clone()).await;

    if result.is_ok() {
        tracing::info!(
            "✅ [COMMAND] Successfully connected to database: '{}'",
            config.name
        );
    } else {
        tracing::error!(
            "❌ [COMMAND] Failed to connect to database: '{}'",
            config.name
        );
    }

    result
}

#[tauri::command]
pub async fn disconnect_from_database(
    connection_id: String,
    state: State<'_, ConnectionStore>,
) -> Result<(), String> {
    tracing::info!(
        "🔌 [COMMAND] Disconnecting from database: {}",
        connection_id
    );
    let result = state.pool.disconnect(&connection_id).await;

    if result.is_ok() {
        tracing::info!(
            "✅ [COMMAND] Successfully disconnected from database: {}",
            connection_id
        );
    } else {
        tracing::error!(
            "❌ [COMMAND] Failed to disconnect from database: {}",
            connection_id
        );
    }

    result
}

#[tauri::command]
pub async fn is_database_connected(
    connection_id: String,
    state: State<'_, ConnectionStore>,
) -> Result<bool, String> {
    Ok(state.pool.is_connected(&connection_id).await)
}

#[tauri::command]
pub async fn get_connected_databases(
    state: State<'_, ConnectionStore>,
) -> Result<Vec<String>, String> {
    Ok(state.pool.get_connected_ids().await)
}

#[tauri::command]
pub async fn get_storage_info() -> Result<StorageInfo, String> {
    let path =
        storage::get_storage_path().map_err(|e| format!("Failed to get storage path: {}", e))?;

    let exists = path.exists();
    let size = if exists {
        std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    Ok(StorageInfo {
        path: path.to_string_lossy().to_string(),
        exists,
        size_bytes: size,
    })
}

#[derive(serde::Serialize)]
pub struct StorageInfo {
    pub path: String,
    pub exists: bool,
    pub size_bytes: u64,
}
