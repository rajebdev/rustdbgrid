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

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    // Use connection from pool
    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_databases().await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_tables(
    config: ConnectionConfig,
    database: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<Table>, String> {
    let connection_id = config.id.clone();

    // Check if already connected, if not connect first
    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    // Use connection from pool
    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_tables(&database).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_views(
    config: ConnectionConfig,
    database: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<View>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_views(&database).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_indexes(
    config: ConnectionConfig,
    database: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<DbIndex>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_indexes(&database).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_procedures(
    config: ConnectionConfig,
    database: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<Procedure>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_procedures(&database).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_triggers(
    config: ConnectionConfig,
    database: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<Trigger>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_triggers(&database).await }.boxed()
        })
        .await
}

#[tauri::command]
pub async fn get_events(
    config: ConnectionConfig,
    database: String,
    state: State<'_, ConnectionStore>,
) -> Result<Vec<Event>, String> {
    let connection_id = config.id.clone();

    if !state.pool.is_connected(&connection_id).await {
        state.pool.connect(config).await?;
    }

    state
        .pool
        .with_connection(&connection_id, |conn| {
            async move { conn.get_events(&database).await }.boxed()
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
