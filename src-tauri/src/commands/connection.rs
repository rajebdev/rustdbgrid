use crate::models::connection::*;
use crate::utils::storage;
use std::sync::Mutex;
use tauri::State;

pub struct ConnectionStore {
    pub connections: Mutex<Vec<ConnectionConfig>>,
}

impl ConnectionStore {
    pub fn new() -> Self {
        // Load connections from file on initialization
        let connections = storage::load_connections().unwrap_or_else(|e| {
            eprintln!("Failed to load connections: {}", e);
            Vec::new()
        });

        ConnectionStore {
            connections: Mutex::new(connections),
        }
    }

    /// Save current connections to file
    fn save_to_file(&self) -> Result<(), String> {
        let connections = self.connections.lock().unwrap();
        storage::save_connections(&connections)
            .map_err(|e| format!("Failed to save connections: {}", e))
    }
}

#[tauri::command]
pub async fn test_connection(config: ConnectionConfig) -> Result<ConnectionStatus, String> {
    let mut conn = crate::db::traits::create_connection(&config.db_type);

    match conn.connect(&config).await {
        Ok(_) => match conn.test_connection().await {
            Ok(true) => Ok(ConnectionStatus {
                success: true,
                message: "Connection successful".to_string(),
            }),
            Ok(false) => Ok(ConnectionStatus {
                success: false,
                message: "Connection failed".to_string(),
            }),
            Err(e) => Ok(ConnectionStatus {
                success: false,
                message: format!("Connection test failed: {}", e),
            }),
        },
        Err(e) => Ok(ConnectionStatus {
            success: false,
            message: format!("Connection failed: {}", e),
        }),
    }
}

#[tauri::command]
pub async fn save_connection(
    config: ConnectionConfig,
    state: State<'_, ConnectionStore>,
) -> Result<(), String> {
    let mut connections = state.connections.lock().unwrap();

    // Remove existing connection with same ID if exists
    connections.retain(|c| c.id != config.id);
    connections.push(config);

    // Release the lock before saving
    drop(connections);

    // Save to file
    state.save_to_file()?;

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
    let mut connections = state.connections.lock().unwrap();
    connections.retain(|c| c.id != id);

    // Release the lock before saving
    drop(connections);

    // Save to file
    state.save_to_file()?;

    Ok(())
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
