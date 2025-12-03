use crate::commands::connection::ConnectionStore;
use crate::models::connection::*;
use crate::models::schema::*;
use futures::FutureExt;
use serde_json::json;
use tauri::State;

/// Universal command for fetching database objects
/// Replaces multiple separate commands with a single unified API
#[tauri::command]
pub async fn get_database_object(
    connection_id: String,
    request_type: String,
    database: Option<String>,
    schema: Option<String>,
    object_name: Option<String>,
    state: State<'_, ConnectionStore>,
) -> Result<serde_json::Value, String> {
    tracing::debug!(
        "📊 [SCHEMA] get_database_object - conn_id: {}, type: {}, db: {:?}, schema: {:?}, object: {:?}",
        connection_id,
        request_type,
        database,
        schema,
        object_name
    );

    // Check if connection exists in pool
    if !state.pool.is_connected(&connection_id).await {
        let config = {
            let connections = state.connections.lock().unwrap();
            connections
                .iter()
                .find(|c| c.id == connection_id)
                .ok_or_else(|| format!("Connection '{}' not found", connection_id))?
                .clone()
        }; // MutexGuard dropped here before await

        let result = state.pool.connect(config.clone()).await;

        if result.is_ok() {
            tracing::info!(
                "✅ [SCHEMA] Successfully connected to database: '{}'",
                config.name
            );
        } else {
            tracing::error!(
                "❌ [SCHEMA] Failed to connect to database: '{}'",
                config.name
            );
        }
    }

    match request_type.as_str() {
        "database_list" => {
            // Get list of databases
            let databases = state
                .pool
                .with_connection(&connection_id, |conn| {
                    async move { conn.get_databases().await }.boxed()
                })
                .await?;

            tracing::info!("✅ [SCHEMA] Retrieved {} databases", databases.len());
            Ok(json!({ "databases": databases }))
        }

        "database_info" => {
            // Get database info (full lists of objects)
            let db_name = database.ok_or("Database name is required")?;

            // Get all object types
            let (tables, views, indexes, procedures, triggers, events) = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = db_name.clone();
                    async move {
                        let tables = conn.get_tables(&db).await.unwrap_or_default();
                        let views = conn.get_views(&db, None).await.unwrap_or_default();
                        let indexes = conn.get_indexes(&db, None).await.unwrap_or_default();
                        let procedures = conn.get_procedures(&db, None).await.unwrap_or_default();
                        let triggers = conn.get_triggers(&db, None).await.unwrap_or_default();
                        let events = conn.get_events(&db, None).await.unwrap_or_default();
                        Ok::<_, anyhow::Error>((
                            tables, views, indexes, procedures, triggers, events,
                        ))
                    }
                    .boxed()
                })
                .await?;

            // Format tables without row_count
            let table_list: Vec<_> = tables
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "schema": t.schema,
                        "size": format_size(t.size_bytes)
                    })
                })
                .collect();

            // Format views with schema
            let view_list: Vec<_> = views
                .iter()
                .map(|v| {
                    json!({
                        "name": v.name,
                        "schema": v.schema
                    })
                })
                .collect();

            tracing::info!("✅ [SCHEMA] Retrieved database info for '{}'", db_name);
            Ok(json!({
                "tables": table_list,
                "views": view_list,
                "indexes": indexes,
                "procedures": procedures,
                "triggers": triggers,
                "events": events
            }))
        }

        "schema_list" => {
            // Get list of schemas (PostgreSQL/MSSQL)
            let db_name = database.ok_or("Database name is required")?;

            // Get tables to extract unique schemas
            let tables = state
                .pool
                .with_connection(&connection_id, |conn| {
                    async move { conn.get_tables(&db_name).await }.boxed()
                })
                .await?;

            // Extract unique schemas
            let mut schemas: Vec<String> = tables
                .iter()
                .filter_map(|t| t.schema.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            schemas.sort();

            let schema_list: Vec<_> = schemas.iter().map(|s| json!({ "name": s })).collect();

            tracing::info!("✅ [SCHEMA] Retrieved {} schemas", schema_list.len());
            Ok(json!({ "schemas": schema_list }))
        }

        "schema_info" => {
            // Get schema info (full lists of objects in a schema)
            let db_name = database.ok_or("Database name is required")?;
            let schema_name = schema.ok_or("Schema name is required")?;

            // Get all object types for the schema
            let (tables, views, indexes, procedures, triggers) = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = db_name.clone();
                    let sch = Some(schema_name.clone());
                    async move {
                        let tables = conn
                            .get_tables(&db)
                            .await
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|t| t.schema.as_ref() == sch.as_ref())
                            .collect::<Vec<_>>();
                        let views = conn
                            .get_views(&db, sch.as_deref())
                            .await
                            .unwrap_or_default();
                        let indexes = conn
                            .get_indexes(&db, sch.as_deref())
                            .await
                            .unwrap_or_default();
                        let procedures = conn
                            .get_procedures(&db, sch.as_deref())
                            .await
                            .unwrap_or_default();
                        let triggers = conn
                            .get_triggers(&db, sch.as_deref())
                            .await
                            .unwrap_or_default();
                        Ok::<_, anyhow::Error>((tables, views, indexes, procedures, triggers))
                    }
                    .boxed()
                })
                .await?;

            // Format tables without row_count
            let table_list: Vec<_> = tables
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "size": format_size(t.size_bytes)
                    })
                })
                .collect();

            // Format views without definition
            let view_list: Vec<_> = views
                .iter()
                .map(|v| {
                    json!({
                        "name": v.name
                    })
                })
                .collect();

            tracing::info!("✅ [SCHEMA] Retrieved schema info for '{}'", schema_name);
            Ok(json!({
                "tables": table_list,
                "views": view_list,
                "indexes": indexes,
                "procedures": procedures,
                "triggers": triggers
            }))
        }

        "procedure" | "function" => {
            // Get procedure/function source code
            let db_name = database.ok_or("Database name is required")?;
            let proc_name = object_name.ok_or("object_name is required for procedure/function")?;
            let proc_schema = schema.clone();

            // Get the procedure source code directly using trait method
            let source = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = db_name.clone();
                    let proc_name_clone = proc_name.clone();
                    let proc_schema_clone = proc_schema.clone();
                    let req_type = request_type.to_string();
                    async move {
                        let proc_type = if req_type == "function" {
                            Some("FUNCTION".to_string())
                        } else {
                            Some("PROCEDURE".to_string())
                        };
                        conn.get_procedure_source(
                            &db,
                            &proc_name_clone,
                            proc_type,
                            proc_schema_clone,
                        )
                        .await
                    }
                    .boxed()
                })
                .await?;

            tracing::info!(
                "✅ [SCHEMA] Retrieved {} source for '{}'",
                request_type,
                proc_name
            );
            Ok(json!({
                "name": proc_name,
                "source": source,
                "type": request_type
            }))
        }

        _ => Err(format!("Unknown request_type: {}", request_type)),
    }
}

/// Universal command for fetching table properties
/// Handles schema, statistics, relationships, triggers, and PostgreSQL-specific properties
#[tauri::command]
pub async fn get_properties_object(
    connection_id: String,
    request_type: String,
    database: String,
    table: String,
    state: State<'_, ConnectionStore>,
) -> Result<serde_json::Value, String> {
    tracing::debug!(
        "📊 [SCHEMA] get_properties_object - conn_id: {}, type: {}, db: {}, table: {}",
        connection_id,
        request_type,
        database,
        table
    );

    // Check if connection exists in pool
    if !state.pool.is_connected(&connection_id).await {
        let config = {
            let connections = state.connections.lock().unwrap();
            connections
                .iter()
                .find(|c| c.id == connection_id)
                .ok_or_else(|| format!("Connection '{}' not found", connection_id))?
                .clone()
        }; // MutexGuard dropped here before await

        let result = state.pool.connect(config.clone()).await;

        if result.is_ok() {
            tracing::info!(
                "✅ [SCHEMA] Successfully connected to database: '{}'",
                config.name
            );
        } else {
            tracing::error!(
                "❌ [SCHEMA] Failed to connect to database: '{}'",
                config.name
            );
        }
    }

    match request_type.as_str() {
        "schema" => {
            // Get table schema (columns, types, etc.)
            let schema = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = database.clone();
                    let tbl = table.clone();
                    async move { conn.get_table_schema(&db, &tbl).await }.boxed()
                })
                .await?;

            tracing::info!("✅ [SCHEMA] Retrieved schema for table '{}'", table);
            serde_json::to_value(schema).map_err(|e| e.to_string())
        }

        "statistics" => {
            // Get table statistics (row count, size, etc.)
            let stats = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = database.clone();
                    let tbl = table.clone();
                    async move { conn.get_table_statistics(&db, &tbl).await }.boxed()
                })
                .await?;

            tracing::info!("✅ [SCHEMA] Retrieved statistics for table '{}'", table);
            serde_json::to_value(stats).map_err(|e| e.to_string())
        }

        "relationships" => {
            // Get table relationships (foreign keys)
            let relationships = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = database.clone();
                    let tbl = table.clone();
                    async move { conn.get_table_relationships(&db, &tbl).await }.boxed()
                })
                .await?;

            tracing::info!(
                "✅ [SCHEMA] Retrieved {} relationships for table '{}'",
                relationships.len(),
                table
            );
            serde_json::to_value(relationships).map_err(|e| e.to_string())
        }

        "triggers" => {
            // Get all triggers for the database, will be filtered in frontend
            let triggers = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = database.clone();
                    async move { conn.get_triggers(&db, None).await }.boxed()
                })
                .await?;

            tracing::info!(
                "✅ [SCHEMA] Retrieved {} triggers for database '{}'",
                triggers.len(),
                database
            );
            serde_json::to_value(triggers).map_err(|e| e.to_string())
        }

        "pg_constraints" => {
            // PostgreSQL-specific: Get constraints
            let constraints = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = database.clone();
                    let tbl = table.clone();
                    async move {
                        if let Some(pg_conn) = conn
                            .as_any_mut()
                            .downcast_mut::<crate::db::postgres::PostgresConnection>()
                        {
                            pg_conn.get_pg_constraints(&db, &tbl).await
                        } else {
                            Err(anyhow::anyhow!("Not a PostgreSQL connection"))
                        }
                    }
                    .boxed()
                })
                .await?;

            tracing::info!(
                "✅ [SCHEMA] Retrieved {} PostgreSQL constraints for table '{}'",
                constraints.len(),
                table
            );
            serde_json::to_value(constraints).map_err(|e| e.to_string())
        }

        "pg_foreign_keys" => {
            // PostgreSQL-specific: Get foreign keys
            let foreign_keys = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = database.clone();
                    let tbl = table.clone();
                    async move {
                        if let Some(pg_conn) = conn
                            .as_any_mut()
                            .downcast_mut::<crate::db::postgres::PostgresConnection>()
                        {
                            pg_conn.get_pg_foreign_keys(&db, &tbl).await
                        } else {
                            Err(anyhow::anyhow!("Not a PostgreSQL connection"))
                        }
                    }
                    .boxed()
                })
                .await?;

            tracing::info!(
                "✅ [SCHEMA] Retrieved {} PostgreSQL foreign keys for table '{}'",
                foreign_keys.len(),
                table
            );
            serde_json::to_value(foreign_keys).map_err(|e| e.to_string())
        }

        "pg_indexes" => {
            // PostgreSQL-specific: Get indexes
            let indexes = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = database.clone();
                    let tbl = table.clone();
                    async move {
                        if let Some(pg_conn) = conn
                            .as_any_mut()
                            .downcast_mut::<crate::db::postgres::PostgresConnection>()
                        {
                            pg_conn.get_pg_indexes(&db, &tbl).await
                        } else {
                            Err(anyhow::anyhow!("Not a PostgreSQL connection"))
                        }
                    }
                    .boxed()
                })
                .await?;

            tracing::info!(
                "✅ [SCHEMA] Retrieved {} PostgreSQL indexes for table '{}'",
                indexes.len(),
                table
            );
            serde_json::to_value(indexes).map_err(|e| e.to_string())
        }

        "pg_references" => {
            // PostgreSQL-specific: Get references
            let references = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = database.clone();
                    let tbl = table.clone();
                    async move {
                        if let Some(pg_conn) = conn
                            .as_any_mut()
                            .downcast_mut::<crate::db::postgres::PostgresConnection>()
                        {
                            pg_conn.get_pg_references(&db, &tbl).await
                        } else {
                            Err(anyhow::anyhow!("Not a PostgreSQL connection"))
                        }
                    }
                    .boxed()
                })
                .await?;

            tracing::info!(
                "✅ [SCHEMA] Retrieved {} PostgreSQL references for table '{}'",
                references.len(),
                table
            );
            serde_json::to_value(references).map_err(|e| e.to_string())
        }

        "pg_partitions" => {
            // PostgreSQL-specific: Get partitions
            let partitions = state
                .pool
                .with_connection(&connection_id, |conn| {
                    let db = database.clone();
                    let tbl = table.clone();
                    async move {
                        if let Some(pg_conn) = conn
                            .as_any_mut()
                            .downcast_mut::<crate::db::postgres::PostgresConnection>()
                        {
                            pg_conn.get_pg_partitions(&db, &tbl).await
                        } else {
                            Err(anyhow::anyhow!("Not a PostgreSQL connection"))
                        }
                    }
                    .boxed()
                })
                .await?;

            tracing::info!(
                "✅ [SCHEMA] Retrieved {} PostgreSQL partitions for table '{}'",
                partitions.len(),
                table
            );
            serde_json::to_value(partitions).map_err(|e| e.to_string())
        }

        _ => Err(format!("Unknown request_type: {}", request_type)),
    }
}

/// Helper function to format size bytes to human-readable format
fn format_size(size_bytes: Option<u64>) -> String {
    match size_bytes {
        Some(bytes) => {
            if bytes == 0 {
                "".to_string()
            } else if bytes < 1024 {
                format!("{}B", bytes)
            } else if bytes < 1024 * 1024 {
                format!("{:.0}K", bytes as f64 / 1024.0)
            } else if bytes < 1024 * 1024 * 1024 {
                format!("{:.0}M", bytes as f64 / (1024.0 * 1024.0))
            } else if bytes < 1024 * 1024 * 1024 * 1024 {
                format!("{:.0}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
            } else {
                format!("{:.0}T", bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
            }
        }
        None => "".to_string(),
    }
}

#[tauri::command]
pub async fn get_databases(
    config: ConnectionConfig,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<Database>, String> {
    let connection_id = config.id.clone();

    tracing::debug!(
        "📊 [SCHEMA] Fetching databases for connection: {}",
        connection_id
    );

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    // Use connection from pool
    let result = state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_databases().await }.boxed()
        })
        .await;

    if let Ok(ref databases) = result {
        tracing::info!("✅ [SCHEMA] Retrieved {} databases", databases.len());
    }

    result
}

#[tauri::command]
pub async fn get_tables(
    config: ConnectionConfig,
    database: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<Table>, String> {
    let connection_id = config.id.clone();
    let db_name = database.clone();

    tracing::debug!("📊 [SCHEMA] Fetching tables from database: {}", database);

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    // Use connection from pool
    let result = state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_tables(&db_name).await }.boxed()
        })
        .await;

    if let Ok(ref tables) = result {
        tracing::info!(
            "✅ [SCHEMA] Retrieved {} tables from '{}'",
            tables.len(),
            database
        );
    }

    result
}

#[tauri::command]
pub async fn get_views(
    config: ConnectionConfig,
    database: String,
    schema: Option<String>,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<View>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_views(&database, schema.as_deref()).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_indexes(
    config: ConnectionConfig,
    database: String,
    schema: Option<String>,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<DbIndex>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_indexes(&database, schema.as_deref()).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_procedures(
    config: ConnectionConfig,
    database: String,
    schema: Option<String>,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<Procedure>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_procedures(&database, schema.as_deref()).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_triggers(
    config: ConnectionConfig,
    database: String,
    schema: Option<String>,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<Trigger>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_triggers(&database, schema.as_deref()).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_events(
    config: ConnectionConfig,
    database: String,
    schema: Option<String>,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<Event>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_events(&database, schema.as_deref()).await }.boxed()
        })
        .await
}
