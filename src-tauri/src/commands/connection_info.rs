use crate::commands::connection::ConnectionStore;
use crate::models::connection::ConnectionConfig;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Minimal connection info for frontend display (no sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub name: String,
    pub db_type: String,
    pub host: String,
    pub port: u16,
}

impl From<&ConnectionConfig> for ConnectionInfo {
    fn from(config: &ConnectionConfig) -> Self {
        Self {
            id: config.id.clone(),
            name: config.name.clone(),
            db_type: format!("{:?}", config.db_type),
            host: config.host.clone(),
            port: config.port,
        }
    }
}

#[tauri::command]
pub async fn get_connections_info(
    state: State<'_, ConnectionStore>,
) -> Result<Vec<ConnectionInfo>, String> {
    let connections = state.connections.lock().unwrap();
    let info: Vec<ConnectionInfo> = connections.iter().map(|c| c.into()).collect();
    Ok(info)
}

#[tauri::command]
pub async fn get_connection_for_edit(
    connection_id: String,
    state: State<'_, ConnectionStore>,
) -> Result<ConnectionConfig, String> {
    tracing::info!(
        "🔐 [SECURITY] Fetching connection details for edit: {}",
        connection_id
    );

    let connections = state.connections.lock().unwrap();
    let config = connections
        .iter()
        .find(|c| c.id == connection_id)
        .ok_or_else(|| format!("Connection '{}' not found", connection_id))?
        .clone();

    Ok(config)
}
