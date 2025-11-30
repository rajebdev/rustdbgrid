use crate::commands::connection::ConnectionStore;
use crate::models::connection::*;
use crate::models::query_result::*;
use crate::models::schema::*;
use futures::FutureExt;
use tauri::State;

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

#[tauri::command]
pub async fn get_table_schema(
    config: ConnectionConfig,
    database: String,
    table: String,
    state: State<'_, ConnectionStore>,
) -> Result<TableSchema, String> {
    let connection_id = config.id.clone();

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    // Use connection from pool
    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_table_schema(&database, &table).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_table_relationships(
    config: ConnectionConfig,
    database: String,
    table: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<TableRelationship>, String> {
    let connection_id = config.id.clone();

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    // Use connection from pool
    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_table_relationships(&database, &table).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_table_statistics(
    config: ConnectionConfig,
    database: String,
    table: String,
    state: State<'_, ConnectionStore>,
) -> Result<TableStatistics, String> {
    let connection_id = config.id.clone();

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    // Use connection from pool
    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_table_statistics(&database, &table).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_table_data(
    config: ConnectionConfig,
    database: String,
    table: String,
    limit: u32,
    offset: u32,
    state: State<'_, ConnectionStore>,
) -> Result<QueryResult, String> {
    let connection_id = config.id.clone();

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    // Use connection from pool
    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_table_data(&database, &table, limit, offset).await }.boxed()
        })
        .await
}

// PostgreSQL-specific commands
#[tauri::command]
pub async fn get_pg_constraints(
    config: ConnectionConfig,
    database: String,
    table: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<PgConstraint>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move {
                if let Some(pg_conn) = conn
                    .as_any_mut()
                    .downcast_mut::<crate::db::postgres::PostgresConnection>()
                {
                    pg_conn.get_pg_constraints(&database, &table).await
                } else {
                    Err(anyhow::anyhow!("Not a PostgreSQL connection"))
                }
            }
            .boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_pg_foreign_keys(
    config: ConnectionConfig,
    database: String,
    table: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<PgForeignKey>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move {
                if let Some(pg_conn) = conn
                    .as_any_mut()
                    .downcast_mut::<crate::db::postgres::PostgresConnection>()
                {
                    pg_conn.get_pg_foreign_keys(&database, &table).await
                } else {
                    Err(anyhow::anyhow!("Not a PostgreSQL connection"))
                }
            }
            .boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_pg_indexes(
    config: ConnectionConfig,
    database: String,
    table: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<PgIndex>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move {
                if let Some(pg_conn) = conn
                    .as_any_mut()
                    .downcast_mut::<crate::db::postgres::PostgresConnection>()
                {
                    pg_conn.get_pg_indexes(&database, &table).await
                } else {
                    Err(anyhow::anyhow!("Not a PostgreSQL connection"))
                }
            }
            .boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_pg_references(
    config: ConnectionConfig,
    database: String,
    table: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<PgReference>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move {
                if let Some(pg_conn) = conn
                    .as_any_mut()
                    .downcast_mut::<crate::db::postgres::PostgresConnection>()
                {
                    pg_conn.get_pg_references(&database, &table).await
                } else {
                    Err(anyhow::anyhow!("Not a PostgreSQL connection"))
                }
            }
            .boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_pg_partitions(
    config: ConnectionConfig,
    database: String,
    table: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<PgPartition>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move {
                if let Some(pg_conn) = conn
                    .as_any_mut()
                    .downcast_mut::<crate::db::postgres::PostgresConnection>()
                {
                    pg_conn.get_pg_partitions(&database, &table).await
                } else {
                    Err(anyhow::anyhow!("Not a PostgreSQL connection"))
                }
            }
            .boxed()
        })
        .await
}
