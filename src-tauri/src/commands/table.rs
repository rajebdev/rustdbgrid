use crate::commands::connection::ConnectionStore;
use crate::db::traits::get_query_builder;
use crate::models::table_request::*;
use futures::FutureExt;
use std::time::Instant;
use tauri::State;

#[tauri::command]
pub async fn load_table_data(
    request: TableDataRequest,
    state: State<'_, ConnectionStore>,
) -> Result<TableDataResponse, String> {
    let connection_id = request.connection_id.clone();
    let query_req = &request.query;

    tracing::debug!(
        "🔍 [TABLE_DATA] Loading data for connection: {}, table: {}",
        connection_id,
        query_req.table
    );

    // Check if already connected, if not try to get from stored connections and connect
    if !state.pool.is_connected(&connection_id).await {
        tracing::info!(
            "🔌 [TABLE_DATA] Connection not found in pool, looking in stored connections..."
        );

        let config = {
            let connections = state.connections.lock().unwrap();
            connections
                .iter()
                .find(|c| c.id == connection_id)
                .ok_or_else(|| format!("Connection '{}' not found", connection_id))?
                .clone()
        }; // MutexGuard dropped here before await

        state.pool.connect(config).await?;
    }

    // Build SQL query using appropriate query builder from the db_type in request
    let sql_query = {
        let query_builder = get_query_builder(&query_req.db_type);
        query_builder
            .build_select_query(query_req)
            .map_err(|e| format!("Failed to build query: {}", e))?
    }; // query_builder dropped here before await

    tracing::debug!("🔍 [TABLE_DATA] Generated SQL: {}", sql_query);

    // Execute query and measure time
    let start = Instant::now();
    let query_result = state
        .pool
        .with_connection(&connection_id, |conn| {
            let query_clone = sql_query.clone();
            async move { conn.execute_query(&query_clone).await }.boxed()
        })
        .await?;
    let execution_time = start.elapsed();

    // Convert QueryResult to TableDataResponse with Vec<Vec<Value>>
    let columns = convert_columns(&query_result);
    let rows = convert_rows_to_vec(&query_result);

    // Check if there's more data by comparing returned rows with limit
    let has_more_data = query_result.rows.len() >= query_req.limit;

    let response = TableDataResponse::new(columns, rows, sql_query, has_more_data, execution_time);

    tracing::info!(
        "✅ [TABLE_DATA] Data loaded. Rows: {}, Columns: {}, Time: {:?}",
        response.rows.len(),
        response.columns.len(),
        execution_time
    );

    Ok(response)
}

/// Convert QueryResult columns to ColumnInfo with type information
fn convert_columns(result: &crate::models::query_result::QueryResult) -> Vec<ColumnInfo> {
    let column_types = result.column_types.as_ref();

    result
        .columns
        .iter()
        .map(|col_name| {
            let data_type = column_types
                .and_then(|types| types.get(col_name))
                .cloned()
                .unwrap_or_else(|| "UNKNOWN".to_string());

            ColumnInfo {
                name: col_name.clone(),
                data_type,
            }
        })
        .collect()
}

/// Convert QueryResult rows (Vec<HashMap>) to Vec<Vec<Value>>
fn convert_rows_to_vec(
    result: &crate::models::query_result::QueryResult,
) -> Vec<Vec<serde_json::Value>> {
    result
        .rows
        .iter()
        .map(|row| {
            // Maintain column order
            result
                .columns
                .iter()
                .map(|col| row.get(col).cloned().unwrap_or(serde_json::Value::Null))
                .collect()
        })
        .collect()
}
