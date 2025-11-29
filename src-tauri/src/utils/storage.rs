use crate::models::connection::ConnectionConfig;
use crate::utils::encryption;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const STORAGE_FILE: &str = "connections.json";
const ENCRYPTION_SALT: &str = "rustdbgrid_v1_salt_2025"; // Salt untuk enkripsi

#[derive(Debug, Serialize, Deserialize)]
struct StoredConnection {
    pub id: String,
    pub name: String,
    pub db_type: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password_encrypted: Option<String>, // Password yang sudah dienkripsi
    pub database: Option<String>,
    pub ssl: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConnectionStorage {
    connections: Vec<StoredConnection>,
}

/// Get the storage directory path
pub fn get_storage_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data = dirs::config_dir().ok_or("Could not find config directory")?;

    let app_dir = app_data.join("rustdbgrid");

    // Create directory if it doesn't exist
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }

    Ok(app_dir.join(STORAGE_FILE))
}

/// Generate encryption password from machine-specific data
fn get_encryption_password() -> String {
    // Use machine-specific data as password
    // In production, you might want to use more secure methods
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    format!("rustdbgrid_{}", hostname)
}

/// Convert ConnectionConfig to StoredConnection with encrypted password
fn to_stored_connection(
    config: &ConnectionConfig,
) -> Result<StoredConnection, Box<dyn std::error::Error>> {
    let password_encrypted = if let Some(ref password) = config.password {
        let encryption_password = get_encryption_password();
        Some(encryption::encrypt(
            password,
            &encryption_password,
            ENCRYPTION_SALT,
        )?)
    } else {
        None
    };

    Ok(StoredConnection {
        id: config.id.clone(),
        name: config.name.clone(),
        db_type: format!("{:?}", config.db_type),
        host: config.host.clone(),
        port: config.port,
        username: config.username.clone(),
        password_encrypted,
        database: config.database.clone(),
        ssl: config.ssl,
    })
}

/// Convert StoredConnection to ConnectionConfig with decrypted password
fn from_stored_connection(
    stored: &StoredConnection,
) -> Result<ConnectionConfig, Box<dyn std::error::Error>> {
    use crate::models::connection::DatabaseType;

    let db_type = match stored.db_type.as_str() {
        "MySQL" => DatabaseType::MySQL,
        "PostgreSQL" => DatabaseType::PostgreSQL,
        "MongoDB" => DatabaseType::MongoDB,
        "Redis" => DatabaseType::Redis,
        "Ignite" => DatabaseType::Ignite,
        "MSSQL" => DatabaseType::MSSQL,
        _ => DatabaseType::MySQL,
    };

    let password = if let Some(ref encrypted) = stored.password_encrypted {
        let encryption_password = get_encryption_password();
        Some(encryption::decrypt(
            encrypted,
            &encryption_password,
            ENCRYPTION_SALT,
        )?)
    } else {
        None
    };

    Ok(ConnectionConfig {
        id: stored.id.clone(),
        name: stored.name.clone(),
        db_type,
        host: stored.host.clone(),
        port: stored.port,
        username: stored.username.clone(),
        password,
        database: stored.database.clone(),
        ssl: stored.ssl,
    })
}

/// Load connections from file
pub fn load_connections() -> Result<Vec<ConnectionConfig>, Box<dyn std::error::Error>> {
    let path = get_storage_path()?;
    
    tracing::debug!("💾 [STORAGE] Loading connections from: {}", path.display());

    if !path.exists() {
        tracing::info!("ℹ️ [STORAGE] No connections file found, returning empty list");
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)?;
    let storage: ConnectionStorage = serde_json::from_str(&content)?;

    let mut connections = Vec::new();
    for stored in storage.connections {
        match from_stored_connection(&stored) {
            Ok(config) => connections.push(config),
            Err(e) => {
                tracing::error!("Failed to decrypt connection {}: {}", stored.name, e);
                // Skip connections that fail to decrypt
            }
        }
    }
    
    tracing::info!("✅ [STORAGE] Loaded {} connections", connections.len());

    Ok(connections)
}

/// Save connections to file
pub fn save_connections(
    connections: &[ConnectionConfig],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_storage_path()?;
    
    tracing::debug!("💾 [STORAGE] Saving {} connections to: {}", connections.len(), path.display());

    let mut stored_connections = Vec::new();
    for config in connections {
        stored_connections.push(to_stored_connection(config)?);
    }

    let storage = ConnectionStorage {
        connections: stored_connections,
    };

    let json = serde_json::to_string_pretty(&storage)?;
    fs::write(&path, json)?;
    
    tracing::info!("✅ [STORAGE] Saved {} connections successfully", connections.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::connection::DatabaseType;

    #[test]
    fn test_storage_conversion() {
        let config = ConnectionConfig {
            id: "test-id".to_string(),
            name: "Test Connection".to_string(),
            db_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            username: Some("user".to_string()),
            password: Some("secret_password".to_string()),
            database: Some("testdb".to_string()),
            ssl: false,
        };

        let stored = to_stored_connection(&config).unwrap();
        let recovered = from_stored_connection(&stored).unwrap();

        assert_eq!(config.id, recovered.id);
        assert_eq!(config.name, recovered.name);
        assert_eq!(config.password, recovered.password);
    }
}
