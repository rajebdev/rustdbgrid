use crate::commands::connection::ConnectionStore;
use crate::db::traits::{get_crud_query_builder, get_query_builder};
use crate::models::connection::DatabaseType;
use crate::models::save_request::{SaveRequest, SaveResponse};
use crate::models::table_request::*;
use futures::FutureExt;
use std::time::Instant;
use tauri::State;
use tracing::{error, info, warn};

/// Helper function to get database type from connection_id
fn get_db_type_from_connection(
    state: &ConnectionStore,
    connection_id: &str,
) -> Result<DatabaseType, String> {
    let connections = state.connections.lock().unwrap();
    connections
        .iter()
        .find(|c| c.id == connection_id)
        .map(|c| c.db_type.clone())
        .ok_or_else(|| format!("Connection '{}' not found", connection_id))
}

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

    // Get table schema to include auto_increment information
    let table_schema = state
        .pool
        .with_connection(&connection_id, |conn| {
            let db = query_req.database.clone().unwrap_or_default();
            let tbl = query_req.table.clone();
            async move { conn.get_table_schema(&db, &tbl).await }.boxed()
        })
        .await
        .ok();

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
    let columns = convert_columns(&query_result, table_schema.as_ref());
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

/// Convert QueryResult columns to ColumnInfo with type information and auto_increment flag
fn convert_columns(
    result: &crate::models::query_result::QueryResult,
    table_schema: Option<&crate::models::schema::TableSchema>,
) -> Vec<ColumnInfo> {
    let column_types = result.column_types.as_ref();

    result
        .columns
        .iter()
        .map(|col_name| {
            let data_type = column_types
                .and_then(|types| types.get(col_name))
                .cloned()
                .unwrap_or_else(|| "UNKNOWN".to_string());

            // Check if column has auto_increment in schema
            let is_auto_increment = table_schema
                .and_then(|schema| schema.columns.iter().find(|col| &col.name == col_name))
                .map(|col| col.is_auto_increment)
                .unwrap_or(false);

            ColumnInfo {
                name: col_name.clone(),
                data_type,
                is_auto_increment,
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

#[tauri::command]
pub async fn generate_sql(
    connection_id: String,
    database: String,
    table: String,
    schema: Option<String>,
    save_request: SaveRequest,
    state: State<'_, ConnectionStore>,
) -> Result<String, String> {
    let db_type = get_db_type_from_connection(&state, &connection_id)?;
    info!("🔍 Generating SQL for table: {}.{}", database, &table);

    let mut queries = Vec::new();
    let crud_builder = get_crud_query_builder(&db_type);

    // Get table schema to find primary keys and column order
    let database_clone = database.clone();
    let table_clone = table.clone();

    let table_schema = state
        .pool
        .with_connection(&connection_id, |conn| {
            let db = database_clone.clone();
            let tbl = table_clone.clone();
            async move { conn.get_table_schema(&db, &tbl).await }.boxed()
        })
        .await
        .map_err(|e| format!("Failed to get table schema: {}", e))?;

    // Generate INSERT queries for new rows
    for (idx, row) in save_request.new_rows.iter().enumerate() {
        match crud_builder.build_insert_query(&table, schema.as_deref(), row, &table_schema) {
            Ok(query) => {
                queries.push(query);
            }
            Err(e) => {
                error!("Failed to generate INSERT for row {}: {}", idx, e);
                return Err(format!("Failed to generate INSERT for row {}: {}", idx, e));
            }
        }
    }

    let primary_keys: Vec<&String> = table_schema
        .columns
        .iter()
        .filter(|col| col.is_primary_key)
        .map(|col| &col.name)
        .collect();

    // Generate UPDATE queries for edited rows
    for (idx, edited_row) in save_request.edited_rows.iter().enumerate() {
        match crud_builder.build_update_query(
            &table,
            schema.as_deref(),
            edited_row,
            &primary_keys,
            &table_schema,
        ) {
            Ok(query) => {
                queries.push(query);
            }
            Err(e) => {
                error!("Failed to generate UPDATE for row {}: {}", idx, e);
                return Err(format!("Failed to generate UPDATE for row {}: {}", idx, e));
            }
        }
    }

    // Generate DELETE queries for deleted rows
    for (idx, row) in save_request.deleted_rows.iter().enumerate() {
        match crud_builder.build_delete_query(&table, schema.as_deref(), row, &primary_keys) {
            Ok(query) => {
                queries.push(query);
            }
            Err(e) => {
                error!("Failed to generate DELETE for row {}: {}", idx, e);
                return Err(format!("Failed to generate DELETE for row {}: {}", idx, e));
            }
        }
    }

    let combined_sql = queries.join("\n");
    info!("✅ Generated {} SQL statements", queries.len());
    Ok(combined_sql)
}

#[tauri::command]
pub async fn save_data(
    connection_id: String,
    database: String,
    table: String,
    schema: Option<String>,
    save_request: SaveRequest,
    state: State<'_, ConnectionStore>,
) -> Result<SaveResponse, String> {
    let db_type = get_db_type_from_connection(&state, &connection_id)?;

    info!(
        "💾 Saving data to {}.{} - New: {}, Edited: {}, Deleted: {}",
        database,
        table,
        save_request.new_rows.len(),
        save_request.edited_rows.len(),
        save_request.deleted_rows.len()
    );

    let mut executed_queries = Vec::new();
    let mut errors = Vec::new();
    let mut affected_rows: i64 = 0;
    let crud_builder = get_crud_query_builder(&db_type);

    // Get table schema to find primary keys
    let database_clone = database.clone();
    let table_clone = table.clone();

    let table_schema = state
        .pool
        .with_connection(&connection_id, |conn| {
            let db = database_clone.clone();
            let tbl = table_clone.clone();
            async move { conn.get_table_schema(&db, &tbl).await }.boxed()
        })
        .await
        .map_err(|e| format!("Failed to get table schema: {}", e))?;

    let primary_keys: Vec<&String> = table_schema
        .columns
        .iter()
        .filter(|col| col.is_primary_key)
        .map(|col| &col.name)
        .collect();

    // Execute INSERT queries
    for (idx, row) in save_request.new_rows.iter().enumerate() {
        match crud_builder.build_insert_query(&table, schema.as_deref(), row, &table_schema) {
            Ok(query) => {
                let connection_id_clone = connection_id.clone();
                let query_clone = query.clone();
                match state
                    .pool
                    .with_connection(&connection_id_clone, |conn| {
                        async move { conn.execute_update(&query_clone).await }.boxed()
                    })
                    .await
                {
                    Ok(rows_affected) => {
                        executed_queries.push(query);
                        affected_rows += rows_affected as i64;
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to insert row {}: {}", idx, e);
                        warn!("{}", error_msg);
                        errors.push(error_msg);
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to generate INSERT for row {}: {}", idx, e);
                warn!("{}", error_msg);
                errors.push(error_msg);
            }
        }
    }

    // Execute UPDATE queries
    for (idx, edited_row) in save_request.edited_rows.iter().enumerate() {
        match crud_builder.build_update_query(
            &table,
            schema.as_deref(),
            edited_row,
            &primary_keys,
            &table_schema,
        ) {
            Ok(query) => {
                let connection_id_clone = connection_id.clone();
                let query_clone = query.clone();
                match state
                    .pool
                    .with_connection(&connection_id_clone, |conn| {
                        async move { conn.execute_update(&query_clone).await }.boxed()
                    })
                    .await
                {
                    Ok(rows_affected) => {
                        if rows_affected == 0 {
                            let error_msg =
                                "No rows updated - row may have been deleted or modified"
                                    .to_string();
                            warn!("{}", error_msg);
                            errors.push(error_msg);
                        } else {
                            executed_queries.push(query);
                            affected_rows += rows_affected as i64;
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to update row {}: {}", idx, e);
                        warn!("{}", error_msg);
                        errors.push(error_msg);
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to generate UPDATE for row {}: {}", idx, e);
                warn!("{}", error_msg);
                errors.push(error_msg);
            }
        }
    }

    // Execute DELETE queries
    for (idx, row) in save_request.deleted_rows.iter().enumerate() {
        match crud_builder.build_delete_query(&table, schema.as_deref(), row, &primary_keys) {
            Ok(query) => {
                let connection_id_clone = connection_id.clone();
                let query_clone = query.clone();
                match state
                    .pool
                    .with_connection(&connection_id_clone, |conn| {
                        async move { conn.execute_update(&query_clone).await }.boxed()
                    })
                    .await
                {
                    Ok(rows_affected) => {
                        executed_queries.push(query);
                        affected_rows += rows_affected as i64;
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to delete row {}: {}", idx, e);
                        warn!("{}", error_msg);
                        errors.push(error_msg);
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to generate DELETE for row {}: {}", idx, e);
                warn!("{}", error_msg);
                errors.push(error_msg);
            }
        }
    }

    if errors.is_empty() {
        info!(
            "✅ Successfully saved all changes - {} rows affected",
            affected_rows
        );
        Ok(SaveResponse::success(
            "All changes saved successfully".to_string(),
            affected_rows,
            executed_queries,
        ))
    } else if !executed_queries.is_empty() {
        let message = format!(
            "Partial save - {} queries executed, {} errors",
            executed_queries.len(),
            errors.len()
        );
        warn!("{}", message);
        Ok(SaveResponse::partial(
            message,
            affected_rows,
            executed_queries,
            errors,
        ))
    } else {
        let message = format!("All save operations failed - {} errors", errors.len());
        error!("{}", message);
        Ok(SaveResponse::error(message))
    }
}
