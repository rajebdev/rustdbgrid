use crate::db::traits::DatabaseConnection;
use crate::models::connection::ConnectionConfig;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// Wrapper for connection with metadata
struct PooledConnection {
    connection: Arc<Mutex<Box<dyn DatabaseConnection>>>,
    last_used: Instant,
}

/// Connection pool manager
pub struct ConnectionPool {
    connections: Mutex<HashMap<String, PooledConnection>>,
}

impl ConnectionPool {
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
        }
    }

    /// Connect to database and add to pool
    pub async fn connect(&self, config: ConnectionConfig) -> Result<(), String> {
        let connection_id = config.id.clone();
        let connection_name = config.name.clone();

        tracing::info!(
            "🔌 [CONNECTION POOL] Connecting to '{}' (ID: {})...",
            connection_name,
            connection_id
        );

        // Create new connection
        let mut conn = crate::db::traits::create_connection(&config.db_type);

        // Connect to database
        conn.connect(&config).await.map_err(|e| {
            tracing::error!(
                "❌ [CONNECTION POOL] Failed to connect to '{}': {}",
                connection_name,
                e
            );
            format!("Failed to connect: {}", e)
        })?;

        // Test connection
        conn.test_connection().await.map_err(|e| {
            tracing::error!(
                "❌ [CONNECTION POOL] Connection test failed for '{}': {}",
                connection_name,
                e
            );
            format!("Connection test failed: {}", e)
        })?;

        // Add to pool
        let pooled = PooledConnection {
            connection: Arc::new(Mutex::new(conn)),
            last_used: Instant::now(),
        };

        let mut connections = self.connections.lock().await;
        connections.insert(connection_id.clone(), pooled);

        tracing::info!(
            "✅ [CONNECTION POOL] Successfully connected to '{}'. Total connections: {}",
            connection_name,
            connections.len()
        );

        Ok(())
    }
    /// Disconnect and remove from pool
    pub async fn disconnect(&self, connection_id: &str) -> Result<(), String> {
        tracing::info!(
            "🔌 [CONNECTION POOL] Disconnecting from '{}' ...",
            connection_id
        );

        let mut connections = self.connections.lock().await;

        if let Some(pooled) = connections.remove(connection_id) {
            let remaining = connections.len();
            drop(connections); // Release lock before async operation

            let mut conn = pooled.connection.lock().await;
            conn.disconnect().await.map_err(|e| {
                tracing::error!(
                    "❌ [CONNECTION POOL] Failed to disconnect from '{}': {}",
                    connection_id,
                    e
                );
                format!("Failed to disconnect: {}", e)
            })?;

            tracing::info!("✅ [CONNECTION POOL] Successfully disconnected from '{}'. Remaining connections: {}", connection_id, remaining);
        } else {
            tracing::warn!(
                "⚠️  [CONNECTION POOL] Connection '{}' not found in pool",
                connection_id
            );
        }

        Ok(())
    }
    /// Check if connection exists and is alive
    pub async fn is_connected(&self, connection_id: &str) -> bool {
        let connections = self.connections.lock().await;
        connections.contains_key(connection_id)
    }

    /// Get all connected connection IDs
    pub async fn get_connected_ids(&self) -> Vec<String> {
        let connections = self.connections.lock().await;
        connections.keys().cloned().collect()
    }

    /// Execute operation with connection from pool
    pub async fn with_connection<F, T>(
        &self,
        connection_id: &str,
        operation: F,
    ) -> Result<T, String>
    where
        F: FnOnce(
            &mut Box<dyn DatabaseConnection>,
        ) -> futures::future::BoxFuture<'_, Result<T, anyhow::Error>>,
    {
        tracing::info!(
            "🔄 [CONNECTION POOL] Using connection '{}' from pool...",
            connection_id
        );

        // Get connection Arc from pool (without removing it)
        let connection_arc = {
            let connections = self.connections.lock().await;
            connections
                .get(connection_id)
                .map(|p| p.connection.clone())
                .ok_or_else(|| {
                    tracing::error!(
                        "❌ [CONNECTION POOL] Connection '{}' not found in pool",
                        connection_id
                    );
                    format!("Connection '{}' not found", connection_id)
                })?
        };

        // Lock the connection for this operation
        let mut conn = connection_arc.lock().await;

        // Execute operation
        let result = operation(&mut conn).await;

        // Update last used timestamp
        {
            let mut connections = self.connections.lock().await;
            if let Some(pooled) = connections.get_mut(connection_id) {
                pooled.last_used = Instant::now();
            }
        }

        match &result {
            Ok(_) => tracing::info!(
                "✅ [CONNECTION POOL] Operation completed successfully with '{}'",
                connection_id
            ),
            Err(e) => tracing::error!(
                "❌ [CONNECTION POOL] Operation failed with '{}': {}",
                connection_id,
                e
            ),
        }

        result.map_err(|e| e.to_string())
    }
}

impl Default for ConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}
